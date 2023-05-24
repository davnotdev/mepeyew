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
#[derive(Debug, Clone, Copy)]
pub enum ShaderUniformFrequencyHint {
    High = 0,
    Mid = 1,
    Low = 2,
    Static = 3,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderCompareOp {
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

//  TODO docs.
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderStencilOp {
    #[default]
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderUniformType {
    Sampler(SamplerId),
    Texture(TextureId),
    UniformBuffer(UniformBufferId),
    InputAttachment(AttachmentImageId),
}

#[derive(Debug, Clone)]
pub struct ShaderUniform {
    pub ty: ShaderUniformType,
    pub binding: usize,
    /// This value also denotes descriptor set indices for apis that use them.
    /// See [`ShaderUniformFrequencyHint`].
    pub frequency: ShaderUniformFrequencyHint,
}

#[derive(Debug, Clone)]
pub enum ShaderType {
    Vertex(VertexBufferInput),
    Fragment,
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
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
pub struct NewProgramExt {
    /// Enable depth testing.
    /// [learnopengl.com](https://learnopengl.com/Advanced-OpenGL/Depth-testing) has a nice
    /// article on this concept.
    pub enable_depth_test: Option<()>,
    /// Enable depth writing.
    pub enable_depth_write: Option<()>,
    /// See [`ShaderDepthCompareOp`].
    pub depth_compare_op: Option<ShaderCompareOp>,

    //  TODO docs. Perhaps we may even want this to become one struct.
    pub enable_stencil_test: Option<()>,
    pub stencil_compare_op: Option<ShaderCompareOp>,
    pub stencil_fail: Option<ShaderStencilOp>,
    pub stencil_pass: Option<ShaderStencilOp>,
    pub stencil_depth_fail: Option<ShaderStencilOp>,
    pub stencil_reference: Option<u32>,
    pub stencil_compare_mask: Option<u32>,
    pub stencil_write_mask: Option<u32>,
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
            Context::WebGpu(wgpu) => wgpu.new_program(shaders, uniforms, ext),
        }
    }
}
