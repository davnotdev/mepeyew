mod surface;
mod swapchain;

use surface::VkSurface;
use swapchain::VkSwapchain;

use super::*;
use context::extensions::surface::SurfaceConfiguration;

pub struct VkSurfaceExt {
    pub swapchain: VkSwapchain,
    pub surface: VkSurface,

    original_surface_config: SurfaceConfiguration,
}

impl VkSurfaceExt {
    pub fn new(
        core: &VkCore,
        drop_queue: &VkDropQueueRef,
        config: SurfaceConfiguration,
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

        Ok(VkSurfaceExt {
            swapchain,
            surface,
            original_surface_config: config,
        })
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

impl VkContext {
    pub fn surface_extension_set_surface_size(
        &mut self,
        width: usize,
        height: usize,
    ) -> GResult<()> {
        //  Resize Surface.
        let old_surface = unsafe { ManuallyDrop::take(&mut self.surface_ext) }.ok_or(gpu_api_err!(
            "vulkan tried to resize surface without surface extension"
        ))?;
        let mut config = old_surface.original_surface_config;
        config.width = width;
        config.height = height;
        self.surface_ext = ManuallyDrop::new(Some(VkSurfaceExt::new(
            &self.core,
            &self.drop_queue,
            config,
        )?));

        //  Resize Dependent Passes
        let patches = self
            .compiled_passes
            .iter()
            .enumerate()
            .filter(|(_, compiled_pass)| compiled_pass.original_pass.depends_on_surface_size)
            .map(|(idx, compiled_pass)| {
                let mut original_pass = compiled_pass.original_pass.clone();
                original_pass.render_width = width;
                original_pass.render_height = height;
                let new_pass = VkCompiledPass::new(self, &original_pass)?;
                Ok((idx, new_pass))
            })
            .collect::<GResult<Vec<_>>>()?;
        patches
            .into_iter()
            .for_each(|(patch_idx, patch_new)| self.compiled_passes[patch_idx] = patch_new);
        Ok(())
    }
}
