use super::*;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ShaderUniformFrequencyHint {
    High = 0,
    Mid = 1,
    Low = 2,
    Static = 3,
}

#[derive(Clone, Copy)]
pub enum ShaderUniformType {
    Texture(TextureId),
    UniformBuffer(UniformBufferId),
}

#[derive(Clone)]
pub struct ShaderUniform {
    pub ty: ShaderUniformType,
    pub binding: usize,
    pub frequency: ShaderUniformFrequencyHint,
}

#[derive(Clone)]
pub enum ShaderType {
    Vertex(VertexBufferInput),
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
        uniforms: &[ShaderUniform],
        ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        match self {
            Context::Vulkan(vk) => vk.new_program(shaders, uniforms, ext),
        }
    }
}
