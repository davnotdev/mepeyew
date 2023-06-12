use super::*;

pub struct VkShader {
    pub module: vk::ShaderModule,
    pub shader_ty: ShaderType,

    drop_queue_ref: VkDropQueueRef,
}

impl VkShader {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        shader_ty: &ShaderType,
        code: &[u8],
    ) -> GResult<Self> {
        let shader_create = vk::ShaderModuleCreateInfo::builder()
            .code(unsafe {
                std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / (32 / 8))
            })
            .build();
        let module = unsafe { dev.create_shader_module(&shader_create, None) }
            .map_err(|e| gpu_api_err!("vulkan shader init {}", e))?;

        Ok(VkShader {
            module,
            shader_ty: shader_ty.clone(),

            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }

    pub fn get_vertex_inputs(
        vertex_inputs: &VertexBufferInput,
    ) -> (
        Vec<vk::VertexInputAttributeDescription>,
        vk::VertexInputBindingDescription,
    ) {
        let mut current_offset = 0;
        (
            vertex_inputs
                .args
                .iter()
                .enumerate()
                .map(|(location, count)| {
                    let ret = vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location(location as u32)
                        .format(stride_count_to_vulkan_format(*count))
                        .offset(current_offset as u32)
                        .build();
                    current_offset += count * std::mem::size_of::<VertexBufferElement>();
                    ret
                })
                .collect::<Vec<_>>(),
            vk::VertexInputBindingDescription::builder()
                .binding(0)
                .stride(current_offset as u32)
                .input_rate(vk::VertexInputRate::VERTEX)
                .build(),
        )
    }
}

impl Drop for VkShader {
    fn drop(&mut self) {
        unsafe {
            let module = self.module;
            self.drop_queue_ref
                .lock()
                .unwrap()
                .push(Box::new(move |dev, _| {
                    dev.destroy_shader_module(module, None);
                }))
        }
    }
}

fn stride_count_to_vulkan_format(count: usize) -> vk::Format {
    match count {
        1 => vk::Format::R32_SFLOAT,
        2 => vk::Format::R32G32_SFLOAT,
        3 => vk::Format::R32G32B32_SFLOAT,
        4 => vk::Format::R32G32B32A32_SFLOAT,
        _ => unimplemented!("vulkan shader exceeded count limit"),
    }
}
