use super::*;

/// Used in [`GetSamplerExt`].
#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum SamplerMode {
    #[default]
    Repeat,
    ClampToBorder,
    ClampToEdge,
    Mirror,
}

/// Used in [`GetSamplerExt`].
#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum SamplerFilter {
    Nearest,
    #[default]
    Linear,
}

/// Used in [`GetSamplerExt`].
#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum MipSamplerFilter {
    Nearest,
    #[default]
    Linear,
}

/// Allows the configuration of:
/// - Min filter
/// - Mag filter
/// - Mip filter
/// - uv overflow behavior
/// - LOD
#[derive(Default, Debug)]
pub struct GetSamplerExt {
    /// The minification filter to use.
    pub min_filter: SamplerFilter,
    /// The maxification filter to use.
    pub mag_filter: SamplerFilter,
    pub mip_filter: MipSamplerFilter,

    /// Overflow behavior of the texture on the u (x) axis.
    pub u_mode: SamplerMode,
    /// Overflow behavior of the texture on the v (y) axis.
    pub v_mode: SamplerMode,

    /// Specify the min lod.
    pub min_lod: Option<f32>,
    /// Specify the max lod.
    /// This value can be obtained for textures using [`Context::get_texture_max_lod`]
    pub max_lod: Option<f32>,
}

impl Context {
    pub fn get_sampler(&mut self, ext: Option<GetSamplerExt>) -> GResult<SamplerId> {
        match self {
            Self::Vulkan(vk) => vk.get_sampler(ext),
            Self::WebGpu(wgpu) => wgpu.get_sampler(ext),
        }
    }
}
