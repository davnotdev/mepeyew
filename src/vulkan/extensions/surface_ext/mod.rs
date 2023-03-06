mod surface;
mod swapchain;

use surface::VkSurface;
use swapchain::VkSwapchain;

use super::*;
use context::extensions::surface::SurfaceConfiguration;

pub struct VkSurfaceExt {
    pub swapchain: VkSwapchain,
    pub surface: VkSurface,
}

impl VkSurfaceExt {
    pub fn new(
        core: &VkCore,
        drop_queue: &VkDropQueueRef,
        config: &SurfaceConfiguration,
    ) -> GResult<Self> {
        let surface = VkSurface::new(
            &core.entry,
            &core.instance,
            &config.display,
            &config.window,
            drop_queue,
        )?;
        let swapchain = VkSwapchain::new(
            core,
            &surface,
            config.width as u32,
            config.height as u32,
            drop_queue,
        )?;

        Ok(VkSurfaceExt { swapchain, surface })
    }

    pub fn get_additional_instance_extensions() -> &'static [&'static str] {
        &[
            #[cfg(target_os = "macos")]
            "VK_EXT_metal_surface",
            #[cfg(all(target_family = "unix", not(target_os = "macos")))]
            "VK_KHR_xlib_surface",
            #[cfg(all(target_family = "unix", not(target_os = "macos")))]
            "VK_KHR_wayland_surface",
            #[cfg(target_family = "windows")]
            "VK_KHR_win32_surface",
            "VK_KHR_surface",
        ]
    }

    pub fn get_additional_device_extension() -> &'static std::ffi::CStr {
        vk_extensions::khr::Swapchain::name()
    }
}

//  TODO FIX: Add Drop
