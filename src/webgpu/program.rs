use super::*;

impl WebGpuContext {
    pub fn new_program(
        &mut self,
        _shaders: &ShaderSet,
        _uniforms: &[ShaderUniform],
        _ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        todo!()
    }
}
