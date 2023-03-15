use super::*;

// pub enum GpuUniformSet {
//     Fast,
//     MidFast,
//     MidSlow,
//     Slow,
// }

// pub enum GpuUniformType {
//     Buffer,
// }

#[derive(Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct ShaderSet<'a>(pub(crate) Vec<(ShaderType, &'a [u8])>);

impl<'a> ShaderSet<'a> {
    pub fn shaders(shaders: &[(ShaderType, &'a [u8])]) -> Self {
        ShaderSet(shaders.to_vec())
    }
}

#[derive(Default)]
pub struct NewProgramExt {}

impl Context {
    pub fn new_program(
        &mut self,
        shaders: &ShaderSet,
        ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        match self {
            Context::Vulkan(vk) => vk.new_program(shaders, ext),
        }
    }
}
