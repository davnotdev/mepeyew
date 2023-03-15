use super::context::{
    self, BufferStorageType, CompiledPassId, Extension, ExtensionType, ImageId, ImageUsage,
    IndexBufferElement, IndexBufferId, Pass, PassInputLoadOpColorType,
    PassInputLoadOpDepthStencilType, PassInputType, PassStep, ProgramId, ShaderSet, ShaderType,
    Submit, VertexBufferElement, VertexBufferId,
};
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

use buffer::{VkIndexBuffer, VkVertexBuffer};
use drop::VkDropQueue;
use frame::{VkFrame, VkFrameDependent};
use framebuffer::VkFramebuffer;
use image::{new_image_view, VkImage, VK_COLOR_ATTACHMENT_FORMAT, VK_DEPTH_ATTACHMENT_FORMAT};
use pass::VkCompiledPass;
use program::VkProgram;
use shader::VkShader;
use submit::VkSubmitData;
use vkcore::{new_fence, new_semaphore, VkCore, VkCoreConfiguration, VkCoreGpuPreference};

mod buffer;
mod debug;
mod drop;
mod extensions;
mod frame;
mod framebuffer;
mod image;
mod pass;
mod program;
mod shader;
mod submit;
mod vkcore;

pub type VkDropQueueRef = Arc<Mutex<VkDropQueue>>;

pub struct VkContext {
    frame: VkFrame,

    programs: ManuallyDrop<Vec<VkProgram>>,
    vbos: ManuallyDrop<Vec<VkVertexBuffer>>,
    ibos: ManuallyDrop<Vec<VkIndexBuffer>>,
    images: ManuallyDrop<Vec<VkImage>>,
    compiled_passes: ManuallyDrop<Vec<VkCompiledPass>>,
    submit: ManuallyDrop<VkSubmitData>,
    alloc: ManuallyDrop<Allocator>,

    enabled_extensions: Vec<ExtensionType>,
    surface_ext: ManuallyDrop<Option<extensions::VkSurfaceExt>>,

    drop_queue: ManuallyDrop<VkDropQueueRef>,

    core: VkCore,
}

impl VkContext {
    pub fn new(extensions: &[Extension]) -> GResult<Self> {
        let supported_extensions = extensions::supported_extensions();
        let (enabled_extensions, unsupported_extensions): (Vec<_>, Vec<_>) = extensions
            .iter()
            .map(|ext| ext.get_type())
            .partition(|ty| supported_extensions.contains(ty));
        if !unsupported_extensions.is_empty() {
            Err(gpu_api_err!(
                "vulkan these extensions not supported: {:?}",
                unsupported_extensions
            ))?;
        }

        //  Core Config from Extensions
        let gpu_preference = extensions
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
            .iter()
            .find_map(|ext| {
                if let Extension::NativeDebug = ext {
                    Some(true)
                } else {
                    None
                }
            })
            .unwrap_or(false);
        let use_surface = extensions
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
        let surface_ext = extensions.iter().find_map(|ext| {
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

        let shaders = ManuallyDrop::new(vec![]);
        let vbos = ManuallyDrop::new(vec![]);
        let ibos = ManuallyDrop::new(vec![]);
        let images = ManuallyDrop::new(vec![]);
        let compiled_passes = ManuallyDrop::new(vec![]);

        Ok(VkContext {
            core,
            frame,

            alloc: ManuallyDrop::new(alloc),
            submit: ManuallyDrop::new(submit),

            drop_queue: ManuallyDrop::new(drop_queue),

            programs: shaders,
            vbos,
            ibos,
            images,
            compiled_passes,

            enabled_extensions,
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

            let _programs = ManuallyDrop::take(&mut self.programs);
            let _vbos = ManuallyDrop::take(&mut self.vbos);
            let _ibos = ManuallyDrop::take(&mut self.ibos);
            let _images = ManuallyDrop::take(&mut self.images);
            let _compiled_passes = ManuallyDrop::take(&mut self.compiled_passes);
        }

        Arc::get_mut(&mut self.drop_queue)
            .expect("vulkan resources are out whilst VkContext is being dropped");
        self.flush_memory();

        unsafe {
            let _alloc = ManuallyDrop::take(&mut self.alloc);
        }
    }
}
