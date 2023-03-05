use super::context::{
    BufferStorageType, CompiledPassId, ImageId, ImageUsage, IndexBufferElement, IndexBufferId,
    Pass, PassInputLoadOpColorType, PassInputLoadOpDepthStencilType, PassInputType, PassStep,
    ProgramId, ShaderSet, ShaderType, Submit, VertexBufferElement, VertexBufferId,
};
use super::error::{gpu_api_err, GResult, GpuError};
use ash::{vk, Entry, *};
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
use swapchain::VkSwapchain;
use vkcore::{new_fence, new_semaphore, VkCore};

mod buffer;
mod debug;
mod drop;
mod frame;
mod framebuffer;
mod image;
mod pass;
mod program;
mod shader;
mod submit;
mod surface;
mod swapchain;
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
    swapchain: ManuallyDrop<VkSwapchain>,

    drop_queue: ManuallyDrop<VkDropQueueRef>,

    core: VkCore,
}

impl VkContext {
    pub fn new(
        display: &RawDisplayHandle,
        window: &RawWindowHandle,
        w: u32,
        h: u32,
    ) -> GResult<Self> {
        let core = VkCore::new(display, window)?;

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

        let swapchain = VkSwapchain::new(&core, w, h, &drop_queue)?;

        //  TODO EXT: Configure frames in flight.
        let frame = VkFrame::new(2);

        let submit = VkSubmitData::new(&core.dev, &frame, core.graphics_command_pool, &drop_queue)?;

        let shaders = ManuallyDrop::new(vec![]);
        let vbos = ManuallyDrop::new(vec![]);
        let ibos = ManuallyDrop::new(vec![]);
        let images = ManuallyDrop::new(vec![]);
        let compiled_passes = ManuallyDrop::new(vec![]);

        let context = VkContext {
            core,
            frame,

            alloc: ManuallyDrop::new(alloc),
            swapchain: ManuallyDrop::new(swapchain),
            submit: ManuallyDrop::new(submit),

            drop_queue: ManuallyDrop::new(drop_queue),

            programs: shaders,
            vbos,
            ibos,
            images,
            compiled_passes,
        };
        Ok(context)
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.core.dev.device_wait_idle().unwrap();

            let _submit = ManuallyDrop::take(&mut self.submit);
            let _swapchain = ManuallyDrop::take(&mut self.swapchain);

            let _programs = ManuallyDrop::take(&mut self.programs);
            let _vbos = ManuallyDrop::take(&mut self.vbos);
            let _ibos = ManuallyDrop::take(&mut self.ibos);
            let _images = ManuallyDrop::take(&mut self.images);
            let _compiled_passes = ManuallyDrop::take(&mut self.compiled_passes);
        }

        let drop_queue = Arc::get_mut(&mut self.drop_queue)
            .expect("vulkan resources are out whilst VkContext is being dropped");
        drop_queue
            .get_mut()
            .unwrap()
            .idle_flush(&self.core.dev, &mut self.alloc);

        unsafe {
            let _alloc = ManuallyDrop::take(&mut self.alloc);
        }
    }
}
