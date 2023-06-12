use super::*;

/// Used in [`NewProgramExt`] for depth and stencil operations.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

/// Used in [`NewProgramExt`] for stencil operations.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

/// Used in [`NewProgramExt`] for blending operations.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShaderBlendOperation {
    #[default]
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

/// Used in [`NewProgramExt`] for blending operations.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

/// Used in [`NewProgramExt`] for culling operations.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShaderCullMode {
    Front,
    #[default]
    Back,
}

/// Used in [`NewProgramExt`] for culling operations.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShaderCullFrontFace {
    Clockwise,
    #[default]
    CounterClockwise,
}

/// Used in [`NewProgramExt`] for primitive topology configuration.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShaderPrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShaderUniformType {
    Sampler(SamplerId),
    Texture(TextureId),
    UniformBuffer(UniformBufferId),
    DynamicUniformBuffer(DynamicUniformBufferId),
    InputAttachment(AttachmentImageId),
    ShaderStorageBuffer(extensions::ShaderStorageBufferId),
    ShaderStorageBufferReadOnly(extensions::ShaderStorageBufferId),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderUniform {
    /// Although most gpus support up to 32 descriptor sets, WebGpu only allows 4.
    /// Therefore, that is currently the max supported set count.
    pub set: usize,
    pub binding: usize,
    pub ty: ShaderUniformType,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ShaderType {
    Vertex(VertexBufferInput),
    Fragment,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderSet<'a>(pub(crate) Vec<(ShaderType, &'a [u8])>);

impl<'a> ShaderSet<'a> {
    pub fn shaders(shaders: &[(ShaderType, &'a [u8])]) -> Self {
        ShaderSet(shaders.to_vec())
    }
}

/// Allows the configuration of:
/// - Depth testing
/// - Stencil testing
/// - Blending
/// - Culling
/// - Primitive Topology
#[derive(Default, Debug, Clone)]
pub struct NewProgramExt {
    pub enable_depth_test: Option<()>,
    pub enable_depth_write: Option<()>,
    pub depth_compare_op: Option<ShaderCompareOp>,

    pub enable_stencil_test: Option<()>,
    pub stencil_compare_op: Option<ShaderCompareOp>,
    pub stencil_fail: Option<ShaderStencilOp>,
    pub stencil_pass: Option<ShaderStencilOp>,
    pub stencil_depth_fail: Option<ShaderStencilOp>,
    pub stencil_reference: Option<u32>,
    pub stencil_compare_mask: Option<u32>,
    pub stencil_write_mask: Option<u32>,

    pub enable_blend: Option<()>,
    pub blend_color_operation: Option<ShaderBlendOperation>,
    pub blend_color_src_factor: Option<ShaderBlendFactor>,
    pub blend_color_dst_factor: Option<ShaderBlendFactor>,
    pub blend_alpha_operation: Option<ShaderBlendOperation>,
    pub blend_alpha_src_factor: Option<ShaderBlendFactor>,
    pub blend_alpha_dst_factor: Option<ShaderBlendFactor>,

    pub enable_culling: Option<()>,
    pub cull_mode: Option<ShaderCullMode>,
    pub cull_front_face: Option<ShaderCullFrontFace>,

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
