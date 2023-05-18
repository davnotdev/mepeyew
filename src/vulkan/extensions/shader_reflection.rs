use super::*;
use context::extensions::shader_reflection::ReflectionShaderTypeHint;
use spirv_reflect::{
    types::{ReflectFormat, ReflectShaderStageFlags},
    ShaderModule,
};

impl VkContext {
    pub fn shader_reflection_extension_reflect(
        &self,
        code: &[u8],
        _hint: ReflectionShaderTypeHint,
    ) -> GResult<ShaderType> {
        let reflect = ShaderModule::load_u8_data(code)
            .map_err(|_| gpu_api_err!("vulkan shader reflection bad code"))?;
        Ok(match reflect.get_shader_stage() {
            ReflectShaderStageFlags::VERTEX => ShaderType::Vertex(vertex_data(&reflect)),
            ReflectShaderStageFlags::FRAGMENT => ShaderType::Fragment,
            _ => unimplemented!("vulkan shader reflection stage unimplemented"),
        })
    }
}

fn vertex_data(reflect: &ShaderModule) -> VertexBufferInput {
    let mut inputs = reflect.enumerate_input_variables(None).unwrap();
    inputs.sort_by_key(|input| input.location);
    VertexBufferInput {
        args: inputs
            .into_iter()
            .map(|input| {
                VertexInputArgCount(match input.format {
                    ReflectFormat::R32_SFLOAT => 1,
                    ReflectFormat::R32G32_SFLOAT => 2,
                    ReflectFormat::R32G32B32_SFLOAT => 3,
                    ReflectFormat::R32G32B32A32_SFLOAT => 4,
                    _ => unimplemented!("vulkan format type size unimplemented"),
                })
            })
            .collect(),
    }
}
