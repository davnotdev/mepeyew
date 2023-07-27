use super::*;
use std::collections::HashMap;

const DESCRIPTOR_SET_COUNT: usize = 8;

//  TODO OPT: Aren't descriptor sets frame dependent?

pub struct VkDescriptors {
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: [vk::DescriptorSet; DESCRIPTOR_SET_COUNT],
    pub descriptor_set_layouts: [vk::DescriptorSetLayout; DESCRIPTOR_SET_COUNT],

    dynamic_indices: Vec<DynamicGenericBufferId>,

    shader_uniforms: Vec<ShaderUniform>,

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

        //  For Binding Dynamic Offsets Layer
        let mut dynamic_indices = vec![];

        //  Descriptor Layouts
        uniforms.iter().for_each(|uniform| {
            let binding_info = match uniform.ty {
                ShaderUniformType::UniformBuffer(_) => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(
                        vk::ShaderStageFlags::VERTEX
                            | vk::ShaderStageFlags::FRAGMENT
                            | vk::ShaderStageFlags::COMPUTE,
                    )
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::DynamicUniformBuffer(id) => {
                    dynamic_indices.push(DynamicGenericBufferId::Uniform(id));
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
                ShaderUniformType::ShaderStorageBuffer(_)
                | ShaderUniformType::ShaderStorageBufferReadOnly(_) => {
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
                ShaderUniformType::Texture(_) => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::CubemapTexture(_) => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::Sampler(_) => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE)
                    .descriptor_type(vk::DescriptorType::SAMPLER)
                    .descriptor_count(1)
                    .build(),
                ShaderUniformType::InputAttachment(_) => vk::DescriptorSetLayoutBinding::builder()
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

        let mut descriptors = VkDescriptors {
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layouts,

            shader_uniforms: uniforms.to_vec(),

            dynamic_indices,

            drop_queue_ref: Arc::clone(&context.drop_queue),
        };

        descriptors.update(context)?;

        Ok(descriptors)
    }

    pub fn update(&mut self, context: &VkContext) -> GResult<()> {
        //  Update Descriptor Sets
        let mut buffer_infos = vec![];
        let mut image_infos = vec![];
        let writes = self
            .shader_uniforms
            .iter()
            .map(|uniform| match uniform.ty {
                ShaderUniformType::UniformBuffer(ubo_id) => {
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

                    buffer_infos.push(buffer_info_list);

                    Ok(ret)
                }
                ShaderUniformType::DynamicUniformBuffer(ubo_id) => {
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

                    buffer_infos.push(buffer_info_list);

                    Ok(ret)
                }
                ShaderUniformType::ShaderStorageBuffer(ssbo_id)
                | ShaderUniformType::ShaderStorageBufferReadOnly(ssbo_id) => {
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

                    buffer_infos.push(buffer_info_list);

                    Ok(ret)
                }
                ShaderUniformType::Texture(texture_id) => {
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
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                        .image_info(&image_info_list)
                        .build();

                    image_infos.push(image_info_list);

                    Ok(ret)
                }
                ShaderUniformType::CubemapTexture(texture_id) => {
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

                    image_infos.push(image_info_list);

                    Ok(ret)
                }
                ShaderUniformType::Sampler(sampler_id) => {
                    let sampler = context.sampler_cache.get(sampler_id).ok_or(gpu_api_err!(
                        "vulkan uniform sampler id {:?} does not exist",
                        sampler_id
                    ))?;
                    let image_info = vk::DescriptorImageInfo::builder().sampler(sampler).build();

                    let image_info_list = vec![image_info];

                    let ret = vk::WriteDescriptorSet::builder()
                        .dst_set(self.descriptor_sets[uniform.set])
                        .dst_binding(uniform.binding as u32)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::SAMPLER)
                        .image_info(&image_info_list)
                        .build();

                    image_infos.push(image_info_list);

                    Ok(ret)
                }
                ShaderUniformType::InputAttachment(attachment_image_id) => {
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

                    image_infos.push(image_info_list);

                    Ok(ret)
                }
            })
            .collect::<GResult<Vec<_>>>()?;

        unsafe { context.core.dev.update_descriptor_sets(&writes, &[]) };
        Ok(())
    }

    pub unsafe fn cmd_bind(
        &self,
        ctx: &VkContext,
        cmd_buf: vk::CommandBuffer,
        bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        dynamic_indices: &HashMap<DynamicGenericBufferId, usize>,
    ) -> GResult<()> {
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
    pub fn update_descriptors(&mut self) -> GResult<()> {
        let p = unsafe { &*(self as *const Self) };
        for program in self.programs.iter_mut() {
            //  TODO FIX: I pinky promise that this won't break.
            program.descriptors.update(p)?;
        }
        Ok(())
    }
}
