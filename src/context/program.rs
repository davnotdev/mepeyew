use super::*;

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

///  Used in [`NewProgramExt`] for stencil operations.
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

//  TODO docs.
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderBlendOperation {
    #[default]
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

//  TODO docs.
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderBlendFactor {
    #[default]
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstColor,
    OneMinusDstColor,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturated,
    ConstantColor,
    ConstantAlpha,
    OneMinusConstantColor,
    OneMinusConstantAlpha,
}

//  TODO docs.
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderCullMode {
    Front,
    #[default]
    Back,
}

//  TODO docs.
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderCullFrontFace {
    Clockwise,
    #[default]
    CounterClockwise,
}

//  TODO docs.
#[derive(Default, Debug, Clone, Copy)]
pub enum ShaderPrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderUniformType {
    Sampler(SamplerId),
    Texture(TextureId),
    UniformBuffer(UniformBufferId),
    DynamicUniformBuffer(DynamicUniformBufferId),
    InputAttachment(AttachmentImageId),
    ShaderStorageBuffer(ShaderStorageBufferId),
    ShaderStorageBufferReadOnly(ShaderStorageBufferId),
}

#[derive(Debug, Clone)]
pub struct ShaderUniform {
    /// Although most gpus support up to 32 descriptor sets, WebGpu only allows 4.
    pub set: usize,
    pub binding: usize,
    pub ty: ShaderUniformType,
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

#[derive(Default, Clone)]
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

    //  TODO docs.
    pub enable_blend: Option<()>,
    pub blend_color_operation: Option<ShaderBlendOperation>,
    pub blend_color_src_factor: Option<ShaderBlendFactor>,
    pub blend_color_dst_factor: Option<ShaderBlendFactor>,
    pub blend_alpha_operation: Option<ShaderBlendOperation>,
    pub blend_alpha_src_factor: Option<ShaderBlendFactor>,
    pub blend_alpha_dst_factor: Option<ShaderBlendFactor>,

    //  TODO docs.
    pub enable_culling: Option<()>,
    pub cull_mode: Option<ShaderCullMode>,
    pub cull_front_face: Option<ShaderCullFrontFace>,

    //  TODO docs.
    pub primitive_topology: Option<ShaderPrimitiveTopology>,
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
