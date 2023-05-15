use super::*;

pub enum ReflectionShaderTypeHint {
    Vertex,
    Fragment,
}

impl Context {
    pub fn shader_reflection_extension_reflect(
        &self,
        code: &[u8],
        hint: ReflectionShaderTypeHint,
    ) -> GResult<ShaderType> {
        self.assert_extension_enabled(ExtensionType::ShaderReflection);
        match self {
            Self::Vulkan(vk) => vk.shader_reflection_extension_reflect(code, hint),
            Self::WebGpu(_) => todo!(),
        }
    }
}
