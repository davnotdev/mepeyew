use super::*;
use std::collections::HashMap;

impl WebGpuContext {
    pub fn get_sampler(&mut self, ext: Option<GetSamplerExt>) -> GResult<SamplerId> {
        let GetSamplerExt {
            min_filter,
            mag_filter,
            mip_filter,
            u_mode,
            v_mode,
            min_lod,
            max_lod,
        } = ext.unwrap_or_default();
        let data = SamplerData {
            min_filter,
            mag_filter,
            mip_filter,
            u_mode,
            v_mode,
            min_lod: min_lod.map(HashableF32::from_val),
            max_lod: max_lod.map(HashableF32::from_val),
        };
        self.sampler_cache
            .get_or_insert(&self.device, data)
            .map(SamplerId::from_id)
    }
}

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
    mip_filter: MipSamplerFilter,
}

pub struct WebGpuSamplerCache {
    samplers: HashMap<SamplerData, GpuSampler>,
    sampler_datas: Vec<SamplerData>,
}

fn filter_into_webgpu(filter: SamplerFilter) -> GpuFilterMode {
    match filter {
        SamplerFilter::Nearest => GpuFilterMode::Nearest,
        SamplerFilter::Linear => GpuFilterMode::Linear,
    }
}

fn mode_into_webgpu(mode: SamplerMode) -> GpuAddressMode {
    match mode {
        SamplerMode::Repeat => GpuAddressMode::Repeat,
        //  TODO FIX: This variant does not exist yet!
        SamplerMode::ClampToBorder => GpuAddressMode::ClampToEdge,
        SamplerMode::ClampToEdge => GpuAddressMode::ClampToEdge,
        SamplerMode::Mirror => GpuAddressMode::MirrorRepeat,
    }
}

impl WebGpuSamplerCache {
    pub fn new() -> Self {
        Self {
            samplers: HashMap::new(),
            sampler_datas: Vec::new(),
        }
    }

    pub fn get(&self, sampler_id: SamplerId) -> Option<GpuSampler> {
        let data = self.sampler_datas.get(sampler_id.id())?;
        self.samplers.get(data).cloned()
    }

    fn get_or_insert(&mut self, device: &GpuDevice, data: SamplerData) -> GResult<usize> {
        if let Some(id) = self
            .sampler_datas
            .iter()
            .position(|&cached_data| cached_data == data)
        {
            Ok(id)
        } else {
            let mut sampler_info = GpuSamplerDescriptor::new();

            sampler_info
                .mipmap_filter(match data.mip_filter {
                    MipSamplerFilter::Nearest => GpuMipmapFilterMode::Nearest,
                    MipSamplerFilter::Linear => GpuMipmapFilterMode::Linear,
                })
                .min_filter(filter_into_webgpu(data.min_filter))
                .mag_filter(filter_into_webgpu(data.mag_filter))
                .address_mode_u(mode_into_webgpu(data.u_mode))
                .address_mode_v(mode_into_webgpu(data.v_mode));

            if let Some(lod) = data.min_lod {
                sampler_info.lod_min_clamp(lod.get_val());
            }
            if let Some(lod) = data.max_lod {
                sampler_info.lod_max_clamp(lod.get_val());
            }

            let sampler = device.create_sampler_with_descriptor(&sampler_info);

            let id = self.sampler_datas.len();
            self.sampler_datas.push(data);
            self.samplers.insert(data, sampler);

            Ok(id)
        }
    }
}
