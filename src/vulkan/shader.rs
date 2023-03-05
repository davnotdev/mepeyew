use super::*;
use spirv_reflect::{types::ReflectFormat, ShaderModule};

pub struct VkShader {
    pub module: vk::ShaderModule,
    pub shader_stage: vk::ShaderStageFlags,
    reflect: ShaderModule,

    drop_queue_ref: VkDropQueueRef,
}

impl VkShader {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        shader_stage: vk::ShaderStageFlags,
        code: &[u8],
    ) -> GResult<Self> {
        //  Make Vulkan Shader Module.
        let shader_create = vk::ShaderModuleCreateInfo::builder()
            .code(unsafe {
                std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / (32 / 8))
            })
            .build();
        let module = unsafe { dev.create_shader_module(&shader_create, None) }
            .map_err(|e| gpu_api_err!("vulkan shader init {}", e))?;

        //  Make SPIRV Reflection Shader Module
        let reflect = ShaderModule::load_u8_data(code).unwrap();

        Ok(VkShader {
            module,
            shader_stage,
            reflect,

            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }

    // pub fn get_uniforms(
    //     &self,
    // ) -> GResult<Vec<(vk::DescriptorSetLayoutBinding, GpuProgramUniformSet)>> {
    //     self.reflect
    //         .enumerate_descriptor_bindings(None)
    //         .unwrap()
    //         .into_iter()
    //         .map(|binding| {
    //             Ok((
    //                 vk::DescriptorSetLayoutBinding::builder()
    //                     .binding(binding.binding)
    //                     .stage_flags(self.shader_stage)
    //                     .descriptor_type(reflect_descriptor_type_into_vulkan(
    //                         binding.descriptor_type,
    //                     ))
    //                     .immutable_samplers(&[])
    //                     .descriptor_count(1)
    //                     .build(),
    //                 match binding.set {
    //                     0 => GpuProgramUniformSet::Fast,
    //                     1 => GpuProgramUniformSet::MidFast,
    //                     2 => GpuProgramUniformSet::MidSlow,
    //                     3 => GpuProgramUniformSet::Slow,
    //                     set => Err(gpu_api_err!(
    //                         "vulkan invalid shader set #{}, see GpuProgramUniformSet",
    //                         set
    //                     ))?,
    //                 },
    //             ))
    //         })
    //         .collect::<GResult<Vec<_>>>()
    // }

    pub fn get_inputs(
        &self,
    ) -> (
        Vec<vk::VertexInputAttributeDescription>,
        vk::VertexInputBindingDescription,
    ) {
        let mut current_offset = 0;
        let mut inputs = self.reflect.enumerate_input_variables(None).unwrap();
        inputs.sort_by_key(|input| input.location);
        (
            inputs
                .into_iter()
                .map(|input| {
                    let format = reflect_format_into_vulkan(input.format);
                    let format_size = vulkan_format_size(format);
                    let ret = vk::VertexInputAttributeDescription::builder()
                        .binding(0)
                        .location(input.location)
                        .format(format)
                        .offset(current_offset as u32)
                        .build();
                    current_offset += format_size;
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

// fn reflect_descriptor_type_into_vulkan(ty: ReflectDescriptorType) -> vk::DescriptorType {
//
//     match ty {
//         ReflectDescriptorType::Sampler => vk::DescriptorType::SAMPLER,
//         ReflectDescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
//         ReflectDescriptorType::SampledImage => vk::DescriptorType::SAMPLED_IMAGE,
//         ReflectDescriptorType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
//         ReflectDescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
//         _ => unimplemented!("vulkan descriptor type unimplemented"),
//     }
// }

fn reflect_format_into_vulkan(format: ReflectFormat) -> vk::Format {
    match format {
        ReflectFormat::Undefined => vk::Format::UNDEFINED,
        ReflectFormat::R32_UINT => vk::Format::R32_UINT,
        ReflectFormat::R32_SINT => vk::Format::R32_SINT,
        ReflectFormat::R32_SFLOAT => vk::Format::R32_SFLOAT,
        ReflectFormat::R32G32_UINT => vk::Format::R32G32_UINT,
        ReflectFormat::R32G32_SINT => vk::Format::R32G32_SINT,
        ReflectFormat::R32G32_SFLOAT => vk::Format::R32G32_SFLOAT,
        ReflectFormat::R32G32B32_UINT => vk::Format::R32G32B32_UINT,
        ReflectFormat::R32G32B32_SINT => vk::Format::R32G32B32_SINT,
        ReflectFormat::R32G32B32_SFLOAT => vk::Format::R32G32B32_SFLOAT,
        ReflectFormat::R32G32B32A32_UINT => vk::Format::R32G32B32A32_UINT,
        ReflectFormat::R32G32B32A32_SINT => vk::Format::R32G32B32A32_SINT,
        ReflectFormat::R32G32B32A32_SFLOAT => vk::Format::R32G32B32A32_SFLOAT,
    }
}

fn vulkan_format_size(format: vk::Format) -> usize {
    match format {
        vk::Format::R32_UINT => 32 / 8,
        vk::Format::R32_SINT => 32 / 8,
        vk::Format::R32_SFLOAT => 32 / 8,
        vk::Format::R32G32_UINT => 32 * 2 / 8,
        vk::Format::R32G32_SINT => 32 * 2 / 8,
        vk::Format::R32G32_SFLOAT => 32 * 2 / 8,
        vk::Format::R32G32B32_UINT => 32 * 3 / 8,
        vk::Format::R32G32B32_SINT => 32 * 3 / 8,
        vk::Format::R32G32B32_SFLOAT => 32 * 3 / 8,
        vk::Format::R32G32B32A32_UINT => 32 / 2,
        vk::Format::R32G32B32A32_SINT => 32 / 2,
        vk::Format::R32G32B32A32_SFLOAT => 32 / 2,
        _ => unimplemented!("vulkan format type size unimplemented"),
    }
}
