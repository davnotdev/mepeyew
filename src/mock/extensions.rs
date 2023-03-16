use super::*;

#[allow(dead_code)]
impl MockContext {
    pub fn extension_is_enabled(&self, _ty: ExtensionType) -> bool {
        unimplemented!("No backend chosen")
    }

    pub fn memory_flush_extension_flush_memory(&mut self) {
        unimplemented!("No backend chosen")
    }

    #[cfg(feature = "surface_extension")]
    pub fn surface_extension_set_surface_size(
        &mut self,
        _width: usize,
        _height: usize,
    ) -> GResult<()> {
        unimplemented!("No backend chosen")
    }

    #[cfg(feature = "shader_reflection_extension")]
    pub fn shader_reflection_extension_reflect(
        &self,
        _code: &[u8],
        _hint: context_extensions::shader_reflection::ReflectionShaderTypeHint,
    ) -> GResult<ShaderType> {
        unimplemented!("No backend chosen")
    }
}
