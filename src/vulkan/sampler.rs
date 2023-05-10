use super::*;
use std::collections::HashMap;

//  We don't care about the f32 hashing edge cases.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct HashableF32(u32);

impl HashableF32 {
    fn from_val(val: f32) -> Self {
        HashableF32(val.to_bits())
    }

    fn get_val(&self) -> f32 {
        f32::from_bits(self.0)
    }
}

#[derive(Clone, Copy, Hash, Default, PartialEq, Eq)]
struct SamplerData {
    min_filter: SamplerFilter,
    mag_filter: SamplerFilter,
    u_mode: SamplerMode,
    v_mode: SamplerMode,
    min_lod: Option<HashableF32>,
    max_lod: Option<HashableF32>,
}

#[derive(Default)]
pub struct VkSamplerCache {
    current_id: usize,
    samplers: HashMap<(usize, SamplerData), vk::Sampler>,
}

impl VkSamplerCache {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_or_insert(&mut self, dev: &Device, data: SamplerData) -> GResult<usize> {
        if let Some(&(id, _)) = self
            .samplers
            .keys()
            .find(|(id, cached_data)| *cached_data == data)
        {
            Ok(id)
        } else {
            let mut sampler_info = vk::SamplerCreateInfo::builder()
                .min_filter(filter_into_vk(data.min_filter))
                .mag_filter(filter_into_vk(data.mag_filter))
                .address_mode_u(mode_into_vk(data.u_mode))
                .address_mode_v(mode_into_vk(data.v_mode))
                .build();

            data.min_lod.map(|lod| sampler_info.min_lod = lod.get_val());
            data.max_lod.map(|lod| sampler_info.max_lod = lod.get_val());

            let sampler = unsafe { dev.create_sampler(&sampler_info, None) }
                .map_err(|e| gpu_api_err!("vulkan sampler {}", e))?;

            let id = self.current_id;
            self.current_id += 1;

            self.samplers.insert((id, data), sampler);

            Ok(id)
        }
    }
}

fn filter_into_vk(filter: SamplerFilter) -> vk::Filter {
    match filter {
        SamplerFilter::Linear => vk::Filter::LINEAR,
        SamplerFilter::Nearest => vk::Filter::NEAREST,
    }
}

fn mode_into_vk(mode: SamplerMode) -> vk::SamplerAddressMode {
    match mode {
        SamplerMode::Repeat => vk::SamplerAddressMode::REPEAT,
        SamplerMode::Mirror => vk::SamplerAddressMode::MIRRORED_REPEAT,
        SamplerMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        SamplerMode::ClampToBorder => vk::SamplerAddressMode::CLAMP_TO_BORDER,
    }
}

impl VkContext {
    pub fn get_sampler(&mut self, ext: Option<GetSamplerExt>) -> GResult<SamplerId> {
        let GetSamplerExt {
            min_filter,
            mag_filter,
            u_mode,
            v_mode,
            min_lod,
            max_lod,
        } = ext.unwrap_or_default();
        let data = SamplerData {
            min_filter,
            mag_filter,
            u_mode,
            v_mode,
            min_lod: min_lod.map(|lod| HashableF32::from_val(lod)),
            max_lod: max_lod.map(|lod| HashableF32::from_val(lod)),
        };
        self.sampler_cache
            .get_or_insert(&self.core.dev, data)
            .map(|id| SamplerId::from_id(id))
    }
}
