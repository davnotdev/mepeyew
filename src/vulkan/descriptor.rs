use super::*;

const DESCRIPTOR_SET_COUNT: usize = 4;

pub struct VkDescriptors {
    descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: [vk::DescriptorSet; DESCRIPTOR_SET_COUNT],
    pub descriptor_set_layouts: [vk::DescriptorSetLayout; DESCRIPTOR_SET_COUNT],

    drop_queue_ref: VkDropQueueRef,
}

impl VkDescriptors {
    pub fn new(context: &VkContext, uniforms: &[ShaderUniform]) -> GResult<Self> {
        //  Descriptor Pool
        let supported_descriptor_types = [
            vk::DescriptorType::UNIFORM_BUFFER,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::DescriptorType::INPUT_ATTACHMENT,
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
                ShaderUniformType::UniformBuffer(_) => vk::DescriptorSetLayoutBinding::builder()
                    .binding(uniform.binding as u32)
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .build(),
            };
            layouts_bindings[0].push(binding_info);
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

        //  Update Descriptor Sets
        let mut buffer_infos = vec![];
        let writes = uniforms
            .iter()
            .map(|uniform| match uniform.ty {
                ShaderUniformType::UniformBuffer(ubo_id) => {
                    let set_idx = uniform.frequency as usize;
                    let ubo = context.ubos.get(ubo_id.id()).ok_or(gpu_api_err!(
                        "vulkan uniform buffer id {:?} does not exist",
                        ubo_id
                    ))?;
                    let buffer_info = vk::DescriptorBufferInfo::builder()
                        .buffer(ubo.buffer.buffer)
                        .range(ubo.buffer.size as u64)
                        .offset(0)
                        .build();
                    buffer_infos.push(buffer_info);
                    Ok(vk::WriteDescriptorSet::builder()
                        .dst_set(descriptor_sets[set_idx])
                        .dst_binding(uniform.binding as u32)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&[buffer_info])
                        .build())
                }
            })
            .collect::<GResult<Vec<_>>>()?;

        unsafe { context.core.dev.update_descriptor_sets(&writes, &[]) };

        Ok(VkDescriptors {
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layouts,

            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
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
