use super::*;
use std::collections::{HashMap, HashSet};

const DESCRIPTOR_SET_COUNT: usize = 8;

//  TODO OPT: Aren't descriptor sets frame dependent?

pub struct VkDescriptors {
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: [vk::DescriptorSet; DESCRIPTOR_SET_COUNT],
    pub descriptor_set_layouts: [vk::DescriptorSetLayout; DESCRIPTOR_SET_COUNT],

    //  This gets searched when we bind, so order doesn't matter.
    dynamic_indices: HashSet<DynamicGenericBufferId>,
    shader_uniforms: Vec<ShaderUniform>,
    initialized_uniforms: Vec<bool>,

    //  Stores update data and index into `shader_uniforms`.
    uniform_datas: Vec<(ShaderUniformUpdateData, usize)>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkDescriptors {
    pub fn new(context: &VkContext, uniforms: &[ShaderUniform]) -> GResult<Self> {
        //  Descriptor Pool
        let supported_descriptor_types = [
            vk::DescriptorType::UNIFORM_BUFFER,
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
            vk::DescriptorType::STORAGE_BUFFER,
            vk::DescriptorType::STORAGE_BUFFER_DYNAMIC,
            vk::DescriptorType::INPUT_ATTACHMENT,
            vk::DescriptorType::SAMPLED_IMAGE,
            vk::DescriptorType::SAMPLER,
        ];

        let descriptor_pool_sizes = supported_descriptor_types
            .into_iter()
            .map(|ty| {
                vk::DescriptorPoolSize::builder()
                    .descriptor_count(DESCRIPTOR_SET_COUNT as u32)
                    .ty(ty)
                    .build()
            })
            .collect::<Vec<_>>();

        let descriptor_pool_create = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&descriptor_pool_sizes)
            .max_sets(DESCRIPTOR_SET_COUNT as u32)
            .build();

        let descriptor_pool = unsafe {
            context
                .core
                .dev
                .create_descriptor_pool(&descriptor_pool_create, None)
        }
        .map_err(|e| gpu_api_err!("vulkan descriptor pool create {}", e))?;

        let mut layouts_bindings = (0..DESCRIPTOR_SET_COUNT)
            .map(|_| vec![])
            .collect::<Vec<_>>();

        //  Descriptor Layouts
        uniforms.iter().for_each(|uniform| {
            let binding_info = match uniform.ty {
                ShaderUniformType::UniformBuffer => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(
                        vk::ShaderStageFlags::VERTEX
                            | vk::ShaderStageFlags::FRAGMENT
                            | vk::ShaderStageFlags::COMPUTE,
                    )
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::DynamicUniformBuffer => {
                    vk::DescriptorSetLayoutBinding::builder()
                        .binding(uniform.binding as u32)
                        .stage_flags(
                            vk::ShaderStageFlags::VERTEX
                                | vk::ShaderStageFlags::FRAGMENT
                                | vk::ShaderStageFlags::COMPUTE,
                        )
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                        .descriptor_count(1)
                        .build()
                }
                ShaderUniformType::ShaderStorageBuffer
                | ShaderUniformType::ShaderStorageBufferReadOnly => {
                    vk::DescriptorSetLayoutBinding::builder()
                        .binding(uniform.binding as u32)
                        .stage_flags(
                            vk::ShaderStageFlags::VERTEX
                                | vk::ShaderStageFlags::FRAGMENT
                                | vk::ShaderStageFlags::COMPUTE,
                        )
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .descriptor_count(1)
                        .build()
                }
                ShaderUniformType::Texture => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::CubemapTexture => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::Sampler => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::InputAttachment => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                    .descriptor_count(1)
                    .build(),
            };
            layouts_bindings[uniform.set].push(binding_info);
        });

        let descriptor_set_layouts: [vk::DescriptorSetLayout; DESCRIPTOR_SET_COUNT] =
            layouts_bindings
                .iter()
                .map(|bindings| {
                    let info = vk::DescriptorSetLayoutCreateInfo::builder()
                        .bindings(bindings)
                        .build();
                    unsafe { context.core.dev.create_descriptor_set_layout(&info, None) }
                        .map_err(|e| gpu_api_err!("vulkan descriptor set create {}", e))
                })
                .collect::<GResult<Vec<_>>>()?
                .try_into()
                .unwrap();

        //  Descriptor Sets
        let descriptor_sets_info = vk::DescriptorSetAllocateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .descriptor_pool(descriptor_pool)
            .build();

        let descriptor_sets: [vk::DescriptorSet; DESCRIPTOR_SET_COUNT] = unsafe {
            context
                .core
                .dev
                .allocate_descriptor_sets(&descriptor_sets_info)
        }
        .map_err(|e| gpu_api_err!("vulkan descriptor set create {}", e))?
        .into_iter()
        .collect::<Vec<vk::DescriptorSet>>()
        .try_into()
        .unwrap();

        let descriptors = VkDescriptors {
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layouts,

            dynamic_indices: HashSet::new(),
            uniform_datas: vec![],
            shader_uniforms: uniforms.to_vec(),

            initialized_uniforms: vec![false; uniforms.len()],

            drop_queue_ref: Arc::clone(&context.drop_queue),
        };

        Ok(descriptors)
    }

    pub fn set_partial_uniform_datas(&mut self, partials: &[Option<ShaderUniformUpdateData>]) {
        for (idx, data) in partials.iter().enumerate() {
            if let Some(data) = data {
                self.uniform_datas.push((*data, idx));
                self.initialized_uniforms[idx] = true;
            }
        }
    }

    pub fn update_all(&mut self, context: &VkContext) -> GResult<()> {
        //  Update Descriptor Sets
        let mut lifetime_buffer_infos = vec![];
        let mut lifetime_image_infos = vec![];

        let mut dynamic_indices = self.dynamic_indices.clone();

        let writes = self
            .uniform_datas
            .iter()
            .map(|(uniform_data, uniform_idx)| {
                let uniform = self.shader_uniforms.get(*uniform_idx).unwrap();
                match uniform_data {
                    ShaderUniformUpdateData::UniformBuffer(ubo_id) => {
                        let ubo = context.ubos.get(ubo_id.id()).ok_or(gpu_api_err!(
                            "vulkan uniform buffer id {:?} does not exist",
                            ubo_id
                        ))?;
                        let buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(ubo.buffer.buffer)
                            .range(ubo.buffer.size as u64)
                            .offset(0)
                            .build();

                        let buffer_info_list = vec![buffer_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                            .buffer_info(&buffer_info_list)
                            .build();

                        lifetime_buffer_infos.push(buffer_info_list);

                        Ok(ret)
                    }
                    ShaderUniformUpdateData::DynamicUniformBuffer(ubo_id) => {
                        dynamic_indices.insert(DynamicGenericBufferId::Uniform(*ubo_id));
                        let ubo = context.dyn_ubos.get(ubo_id.id()).ok_or(gpu_api_err!(
                            "vulkan dynamic uniform buffer id {:?} does not exist",
                            ubo_id
                        ))?;
                        let buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(ubo.buffer.buffer)
                            .range(ubo.per_index_offset as u64)
                            .offset(0)
                            .build();

                        let buffer_info_list = vec![buffer_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                            .buffer_info(&buffer_info_list)
                            .build();

                        lifetime_buffer_infos.push(buffer_info_list);

                        Ok(ret)
                    }
                    ShaderUniformUpdateData::ShaderStorageBuffer(ssbo_id)
                    | ShaderUniformUpdateData::ShaderStorageBufferReadOnly(ssbo_id) => {
                        let ssbo = context.ssbos.get(ssbo_id.id()).ok_or(gpu_api_err!(
                            "vulkan shader storage buffer id {:?} does not exist",
                            ssbo_id
                        ))?;
                        let buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(ssbo.buffer.buffer)
                            .range(ssbo.buffer.size as u64)
                            .offset(0)
                            .build();

                        let buffer_info_list = vec![buffer_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                            .buffer_info(&buffer_info_list)
                            .build();

                        lifetime_buffer_infos.push(buffer_info_list);

                        Ok(ret)
                    }
                    ShaderUniformUpdateData::Texture(idx, texture_id) => {
                        let texture = context.textures.get(texture_id.id()).ok_or(gpu_api_err!(
                            "vulkan uniform texture id {:?} does not exist",
                            texture_id
                        ))?;
                        let image_info = vk::DescriptorImageInfo::builder()
                            .image_view(texture.image_view)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .build();

                        let image_info_list = vec![image_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(*idx as u32)
                            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                            .image_info(&image_info_list)
                            .build();

                        lifetime_image_infos.push(image_info_list);

                        Ok(ret)
                    }
                    ShaderUniformUpdateData::CubemapTexture(texture_id) => {
                        let texture = context.textures.get(texture_id.id()).ok_or(gpu_api_err!(
                            "vulkan uniform cubemap texture id {:?} does not exist",
                            texture_id
                        ))?;
                        let image_info = vk::DescriptorImageInfo::builder()
                            .image_view(texture.image_view)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .build();

                        let image_info_list = vec![image_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                            .image_info(&image_info_list)
                            .build();

                        lifetime_image_infos.push(image_info_list);

                        Ok(ret)
                    }
                    ShaderUniformUpdateData::Sampler(idx, sampler_id) => {
                        let sampler =
                            context.sampler_cache.get(*sampler_id).ok_or(gpu_api_err!(
                                "vulkan uniform sampler id {:?} does not exist",
                                sampler_id
                            ))?;
                        let image_info =
                            vk::DescriptorImageInfo::builder().sampler(sampler).build();

                        let image_info_list = vec![image_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(*idx as u32)
                            .descriptor_type(vk::DescriptorType::SAMPLER)
                            .image_info(&image_info_list)
                            .build();

                        lifetime_image_infos.push(image_info_list);

                        Ok(ret)
                    }
                    ShaderUniformUpdateData::InputAttachment(attachment_image_id) => {
                        let attachment_image = context
                            .attachment_images
                            .get(attachment_image_id.id())
                            .ok_or(gpu_api_err!(
                                "vulkan uniform attachment image id {:?} does not exist",
                                attachment_image_id
                            ))?;
                        let image_info = vk::DescriptorImageInfo::builder()
                            .image_view(attachment_image.image_view)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .build();

                        let image_info_list = vec![image_info];

                        let ret = vk::WriteDescriptorSet::builder()
                            .dst_set(self.descriptor_sets[uniform.set])
                            .dst_binding(uniform.binding as u32)
                            .dst_array_element(0)
                            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                            .image_info(&image_info_list)
                            .build();

                        lifetime_image_infos.push(image_info_list);

                        Ok(ret)
                    }
                }
            })
            .collect::<GResult<Vec<_>>>()?;

        self.dynamic_indices = dynamic_indices;

        unsafe { context.core.dev.update_descriptor_sets(&writes, &[]) };
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized_uniforms.iter().all(|i| *i)
    }

    pub unsafe fn cmd_bind(
        &self,
        ctx: &VkContext,
        cmd_buf: vk::CommandBuffer,
        bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        dynamic_indices: &HashMap<DynamicGenericBufferId, usize>,
    ) -> GResult<()> {
        if !self.is_initialized() {
            Err(gpu_api_err!("vulkan not all uniforms updated"))?
        }

        let mut offsets = vec![0; self.dynamic_indices.len()];

        (self.dynamic_indices.len() == dynamic_indices.len())
            .then_some(())
            .ok_or(gpu_api_err!(
                "vulkan not all dynamic indices provided for draw"
            ))?;

        dynamic_indices
            .iter()
            .try_for_each(|(id, index)| match id {
                DynamicGenericBufferId::Uniform(id) => {
                    let offset_index = self
                        .dynamic_indices
                        .iter()
                        .position(|p| match p {
                            DynamicGenericBufferId::Uniform(p) => p == id,
                        })
                        .ok_or(gpu_api_err!(
                            "vulkan dynamic uniform buffer (for indexing) {:?} does not exist",
                            id
                        ))?;
                    offsets[offset_index] =
                        (*index * ctx.dyn_ubos.get(id.id()).unwrap().per_index_offset) as u32;
                    Ok(())
                }
            })?;

        ctx.core.dev.cmd_bind_descriptor_sets(
            cmd_buf,
            bind_point,
            layout,
            0,
            &self.descriptor_sets,
            &offsets,
        );

        Ok(())
    }
}

impl Drop for VkDescriptors {
    fn drop(&mut self) {
        let descriptor_pool = self.descriptor_pool;
        let descriptor_set_layouts = self.descriptor_set_layouts;

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _alloc| unsafe {
                dev.destroy_descriptor_pool(descriptor_pool, None);
                for descriptor_set_layout in descriptor_set_layouts {
                    dev.destroy_descriptor_set_layout(descriptor_set_layout, None)
                }
            }));
    }
}

impl VkContext {
    pub fn update_all_descriptors(&mut self) -> GResult<()> {
        let p = unsafe { &*(self as *const Self) };
        for program in self.programs.iter_mut() {
            //  TODO FIX: I pinky promise that this won't break.
            program.descriptors.update_all(p)?;
        }
        Ok(())
    }
}
