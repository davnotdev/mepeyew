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
    pub min_filter: SamplerFilter,
    pub mag_filter: SamplerFilter,
    pub u_mode: SamplerMode,
    pub v_mode: SamplerMode,
    pub min_lod: Option<f32>,
    pub max_lod: Option<f32>,
}

impl Context {
    pub fn get_sampler(&mut self, ext: Option<GetSamplerExt>) -> GResult<SamplerId> {
        match self {
            Self::Vulkan(vk) => vk.get_sampler(ext),
        }
    }
}
