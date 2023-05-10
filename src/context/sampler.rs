use super::*;

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum SamplerMode {
    #[default]
    Repeat,
    ClampToBorder,
    ClampToEdge,
    Mirror,
}

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
pub enum SamplerFilter {
    #[default]
    Nearest,
    Linear,
}

#[derive(Default)]
pub struct GetSamplerExt {
    min_filter: SamplerFilter,
    mag_filter: SamplerFilter,
    u_mode: SamplerMode,
    v_mode: SamplerMode,
    min_lod: Option<f32>,
    max_lod: Option<f32>,
}

impl Context {
    pub fn get_sampler(&mut self, ext: Option<GetSamplerExt>) -> GResult<SamplerId> {
        match self {
            Self::Vulkan(vk) => vk.get_sampler(ext),
        }
    }
}
