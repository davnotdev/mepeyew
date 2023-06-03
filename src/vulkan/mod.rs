use super::context::{self, *};
use super::error::{gpu_api_err, GResult, GpuError};
use ash::{extensions as vk_extensions, vk, Entry, *};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, Allocator, AllocatorCreateDesc},
    MemoryLocation,
};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::{
    mem::ManuallyDrop,
    sync::{Arc, Mutex},
};

use attachment_image::VkAttachmentImage;
use buffer::{
    VkBuffer, VkDynamicUniformBuffer, VkIndexBuffer, VkShaderStorageBuffer, VkUniformBuffer,
    VkVertexBuffer,
};
use compute::{VkCompiledComputePass, VkComputeProgram};
use descriptor::VkDescriptors;
use drop::VkDropQueue;
use frame::{VkFrame, VkFrameDependent};
use framebuffer::VkFramebuffer;
use image::{new_image_view, VkImage, VK_COLOR_ATTACHMENT_FORMAT, VK_DEPTH_ATTACHMENT_FORMAT};
use pass::VkCompiledPass;
use program::{new_pipeline_layout, VkProgram};
use sampler::VkSamplerCache;
use shader::VkShader;
use submit::VkSubmitData;
use texture::VkTexture;
use vkcore::{new_fence, new_semaphore, VkCore, VkCoreConfiguration, VkCoreGpuPreference};

mod attachment_image;
mod buffer;
mod compute;
mod debug;
mod descriptor;
mod drop;
mod extensions;
mod frame;
mod framebuffer;
mod image;
mod pass;
mod program;
mod sampler;
mod shader;
mod submit;
mod texture;
mod vkcore;

pub type VkDropQueueRef = Arc<Mutex<VkDropQueue>>;

pub struct VkContext {
    frame: VkFrame,

    programs: ManuallyDrop<Vec<VkProgram>>,
    compute_programs: ManuallyDrop<Vec<VkComputeProgram>>,
    vbos: ManuallyDrop<Vec<VkVertexBuffer>>,
    ibos: ManuallyDrop<Vec<VkIndexBuffer>>,
    ubos: ManuallyDrop<Vec<VkUniformBuffer>>,
    dyn_ubos: ManuallyDrop<Vec<VkDynamicUniformBuffer>>,
    ssbos: ManuallyDrop<Vec<VkShaderStorageBuffer>>,
    textures: ManuallyDrop<Vec<VkTexture>>,
    attachment_images: ManuallyDrop<Vec<VkAttachmentImage>>,
    compiled_passes: ManuallyDrop<Vec<VkCompiledPass>>,
    compiled_compute_passes: ManuallyDrop<Vec<VkCompiledComputePass>>,
    submit: ManuallyDrop<VkSubmitData>,
    sampler_cache: ManuallyDrop<VkSamplerCache>,
    alloc: ManuallyDrop<Allocator>,

    surface_ext: ManuallyDrop<Option<extensions::VkSurfaceExt>>,

    drop_queue: ManuallyDrop<VkDropQueueRef>,

    core: VkCore,
}

impl VkContext {
    pub fn new(extensions: Extensions) -> GResult<Self> {
        extensions::check_extensions(&extensions)?;

        //  Core Config from Extensions
        let gpu_preference = extensions
            .extensions
            .iter()
            .find_map(|ext| {
                if let &Extension::GpuPowerLevel(power_level) = &ext {
                    Some(match power_level {
                        context::extensions::gpu_power_level::GpuPowerLevel::PreferDiscrete => {
                            VkCoreGpuPreference::Discrete
                        }
                        context::extensions::gpu_power_level::GpuPowerLevel::PreferIntegrated => {
                            VkCoreGpuPreference::Integrated
                        }
                    })
                } else {
                    None
                }
            })
            .unwrap_or(VkCoreGpuPreference::Discrete);
        let use_debug = extensions
            .extensions
            .iter()
            .find_map(|ext| {
                if let Extension::NativeDebug(_) = ext {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        let use_surface = extensions
            .extensions
            .iter()
            .find_map(|ext| {
                if let Extension::Surface(_) = ext {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        let core_config = VkCoreConfiguration {
            gpu_preference,
            use_debug,
            use_surface,
        };

        let core = VkCore::new(core_config)?;
        let drop_queue = Arc::new(Mutex::new(VkDropQueue::new()));

        let alloc = Allocator::new(&AllocatorCreateDesc {
            instance: core.instance.clone(),
            device: core.dev.clone(),
            physical_device: core.physical_dev,
            debug_settings: Default::default(),
            //  TODO OPT: We should enable this perhaps?
            buffer_device_address: false,
        })
        .map_err(|e| gpu_api_err!("vulkan gpu_allocator {}", e))?;

        //  Surface Extension
        let surface_ext = extensions.extensions.iter().find_map(|ext| {
            if let Extension::Surface(surface) = &ext {
                Some(extensions::VkSurfaceExt::new(&core, &drop_queue, *surface))
            } else {
                None
            }
        });
        let surface_ext = if let Some(r) = surface_ext {
            Some(r?)
        } else {
            None
        };

        //  Frames in Flight Extension
        let inflight_frame_count = extensions
            .extensions
            .iter()
            .find_map(|ext| {
                if let &Extension::FlightFramesCount(count) = ext {
                    Some(count)
                } else {
                    None
                }
            })
            .unwrap_or(2);
        let frame = VkFrame::new(inflight_frame_count);

        //  Context State
        let submit = VkSubmitData::new(&core.dev, &frame, core.graphics_command_pool, &drop_queue)?;

        //  Sampler Cache
        let sampler_cache = VkSamplerCache::new(&drop_queue);

        let programs = ManuallyDrop::new(vec![]);
        let compute_programs = ManuallyDrop::new(vec![]);
        let vbos = ManuallyDrop::new(vec![]);
        let ibos = ManuallyDrop::new(vec![]);
        let ubos = ManuallyDrop::new(vec![]);
        let dyn_ubos = ManuallyDrop::new(vec![]);
        let ssbos = ManuallyDrop::new(vec![]);
        let textures = ManuallyDrop::new(vec![]);
        let attachment_images = ManuallyDrop::new(vec![]);
        let compiled_passes = ManuallyDrop::new(vec![]);
        let compiled_compute_passes = ManuallyDrop::new(vec![]);

        Ok(VkContext {
            core,
            frame,

            alloc: ManuallyDrop::new(alloc),
            submit: ManuallyDrop::new(submit),
            sampler_cache: ManuallyDrop::new(sampler_cache),

            drop_queue: ManuallyDrop::new(drop_queue),

            programs,
            compute_programs,
            vbos,
            ibos,
            ubos,
            dyn_ubos,
            ssbos,
            textures,
            attachment_images,
            compiled_passes,
            compiled_compute_passes,

            surface_ext: ManuallyDrop::new(surface_ext),
        })
    }

    pub fn flush_memory(&mut self) {
        unsafe {
            self.core.dev.device_wait_idle().unwrap();
        };
        self.drop_queue
            .lock()
            .unwrap()
            .idle_flush(&self.core.dev, &mut self.alloc);
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.core.dev.device_wait_idle().unwrap();

            let _submit = ManuallyDrop::take(&mut self.submit);
            let _surface_ext = ManuallyDrop::take(&mut self.surface_ext);
            let _sampler_cache = ManuallyDrop::take(&mut self.sampler_cache);

            let _programs = ManuallyDrop::take(&mut self.programs);
            let _compute_programs = ManuallyDrop::take(&mut self.compute_programs);
            let _vbos = ManuallyDrop::take(&mut self.vbos);
            let _ibos = ManuallyDrop::take(&mut self.ibos);
            let _ubos = ManuallyDrop::take(&mut self.ubos);
            let _dyn_ubos = ManuallyDrop::take(&mut self.dyn_ubos);
            let _ssbos = ManuallyDrop::take(&mut self.ssbos);
            let _textures = ManuallyDrop::take(&mut self.textures);
            let _attachment_images = ManuallyDrop::take(&mut self.attachment_images);
            let _compiled_passes = ManuallyDrop::take(&mut self.compiled_passes);
            let _compiled_compute_passes = ManuallyDrop::take(&mut self.compiled_compute_passes);
        }

        Arc::get_mut(&mut self.drop_queue)
            .expect("vulkan resources are out whilst VkContext is being dropped");
        self.flush_memory();

        unsafe {
            let _alloc = ManuallyDrop::take(&mut self.alloc);
        }
    }
}
