use super::*;

/// Not all graphics apis use the concept of descriptor sets.
/// When using an api that does, this value also notes the descriptor set index.
/// The values are shown below.
/// ```
/// #[repr(u8)]
/// #[derive(Clone, Copy)]
/// pub enum ShaderUniformFrequencyHint {
///     High = 0,
///     Mid = 1,
///     Low = 2,
///     Static = 3,
/// }
/// ```
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ShaderUniformFrequencyHint {
    High = 0,
    Mid = 1,
    Low = 2,
    Static = 3,
}

/// Used in [`NewProgramExt`] to configure depth testing.
/// [learnopengl.com](https://learnopengl.com/Advanced-OpenGL/Depth-testing) has a nice article
/// about this topic.
#[derive(Default, Clone, Copy)]
pub enum ShaderDepthCompareOp {
    Never,
    #[default]
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

#[derive(Clone, Copy)]
pub enum ShaderUniformType {
    Texture(TextureId),
    UniformBuffer(UniformBufferId),
    InputAttachment(AttachmentImageId),
}

#[derive(Clone)]
pub struct ShaderUniform {
    pub ty: ShaderUniformType,
    pub binding: usize,
    /// This value also denotes descriptor set indices for apis that use them.
    /// See [`ShaderUniformFrequencyHint`].
    pub frequency: ShaderUniformFrequencyHint,
}

/// This value can be inferred using the shader reflection extension.
/// See [`Extension`].
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
pub struct NewProgramExt {
    /// Enable depth testing.
    /// [learnopengl.com](https://learnopengl.com/Advanced-OpenGL/Depth-testing) has a nice article
    pub enable_depth_test: Option<()>,
    /// See [`ShaderDepthCompareOp`].
    pub depth_compare_op: Option<ShaderDepthCompareOp>,
}

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
