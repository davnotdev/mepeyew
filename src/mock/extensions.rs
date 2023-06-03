use super::*;

#[allow(dead_code)]
impl MockContext {
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

    pub fn new_shader_storage_buffer<T: Copy>(
        &mut self,
        _data: &T,
        _ext: Option<context::extensions::NewShaderStorageBufferExt>,
    ) -> GResult<context::extensions::ShaderStorageBufferId> {
        unimplemented!("No backend chosen")
    }

    pub fn new_compute_program(
        &mut self,
        _code: &[u8],
        _uniforms: &[ShaderUniform],
        _ext: Option<context::extensions::NewComputeProgramExt>,
    ) -> GResult<ComputeProgramId> {
        unimplemented!("No backend chosen")
    }

    pub fn compile_compute_pass(
        &mut self,
        _compute_pass: context::extensions::ComputePass,
        _ext: Option<context::extensions::CompileComputePassExt>,
    ) -> GResult<CompiledComputePassId> {
        unimplemented!("No backend chosen")
    }

    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        _ssbo: context::extensions::ShaderStorageBufferId,
        _ext: Option<context::extensions::ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        unimplemented!("No backend chosen")
    }
}
