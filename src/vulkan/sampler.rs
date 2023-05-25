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
    mip_filter: MipSamplerFilter,
}

pub struct VkSamplerCache {
    samplers: HashMap<SamplerData, vk::Sampler>,
    sampler_datas: Vec<SamplerData>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkSamplerCache {
    pub fn new(drop_queue_ref: &VkDropQueueRef) -> Self {
        Self {
            samplers: HashMap::new(),
            sampler_datas: Vec::new(),
            drop_queue_ref: Arc::clone(drop_queue_ref),
        }
    }

    pub fn get(&self, sampler_id: SamplerId) -> Option<vk::Sampler> {
        let data = self.sampler_datas.get(sampler_id.id())?;
        self.samplers.get(data).cloned()
    }

    fn get_or_insert(&mut self, dev: &Device, data: SamplerData) -> GResult<usize> {
        if let Some(id) = self
            .sampler_datas
            .iter()
            .position(|&cached_data| cached_data == data)
        {
            Ok(id)
        } else {
            let mut sampler_info = vk::SamplerCreateInfo::builder()
                .min_filter(filter_into_vk(data.min_filter))
                .mag_filter(filter_into_vk(data.mag_filter))
                .mipmap_mode(mip_filter_into_vk(data.mip_filter))
                .address_mode_u(mode_into_vk(data.u_mode))
                .address_mode_v(mode_into_vk(data.v_mode))
                .build();

            if let Some(lod) = data.min_lod {
                sampler_info.min_lod = lod.get_val()
            }
            if let Some(lod) = data.max_lod {
                sampler_info.max_lod = lod.get_val()
            }

            let sampler = unsafe { dev.create_sampler(&sampler_info, None) }
                .map_err(|e| gpu_api_err!("vulkan sampler {}", e))?;

            let id = self.sampler_datas.len();
            self.sampler_datas.push(data);
            self.samplers.insert(data, sampler);

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

fn mip_filter_into_vk(filter: MipSamplerFilter) -> vk::SamplerMipmapMode {
    match filter {
        MipSamplerFilter::Linear => vk::SamplerMipmapMode::LINEAR,
        MipSamplerFilter::Nearest => vk::SamplerMipmapMode::NEAREST,
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
            .get_or_insert(&self.core.dev, data)
            .map(SamplerId::from_id)
    }
}

impl Drop for VkSamplerCache {
    fn drop(&mut self) {
        let samplers = self.samplers.values().cloned().collect::<Vec<_>>();
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                for sampler in samplers {
                    dev.destroy_sampler(sampler, None);
                }
            }))
    }
}
