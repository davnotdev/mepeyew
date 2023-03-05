use super::*;

pub struct VkSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_ext: ash::extensions::khr::Surface,
}

impl VkSurface {
    pub fn new(
        entry: &Entry,
        instance: &ash::Instance,
        display: &RawDisplayHandle,
        window: &RawWindowHandle,
    ) -> GResult<Self> {
        let surface_ext = ash::extensions::khr::Surface::new(entry, instance);
        let surface = if cfg!(target_os = "macos") {
            new_unix_macos_surface(entry, instance, display, window)
        } else if cfg!(target_os = "windows") {
            new_windows_surface(entry, instance, display, window)
        } else {
            //  Try using wayland first.
            if let Some(ok) = new_unix_wayland_surface(entry, instance, display, window) {
                Some(ok)
            } else {
                new_unix_xlib_surface(entry, instance, display, window)
            }
        }
        .ok_or(gpu_api_err!("vulkan cannot load any platform surface"))?;
        Ok(VkSurface {
            surface,
            surface_ext,
        })
    }

    pub fn get_additional_extensions() -> &'static [&'static str] {
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

    pub unsafe fn destroy(&mut self) {
        self.surface_ext.destroy_surface(self.surface, None);
    }
}

fn new_windows_surface(
    entry: &Entry,
    instance: &ash::Instance,
    _display: &RawDisplayHandle,
    window: &RawWindowHandle,
) -> Option<vk::SurfaceKHR> {
    let RawWindowHandle::Win32(native_window) = window else {
        None?
    };
    let native_surface_create = vk::Win32SurfaceCreateInfoKHR::builder()
        .hinstance(native_window.hinstance)
        .hwnd(native_window.hwnd)
        .build();
    let native_surface = ash::extensions::khr::Win32Surface::new(entry, instance);
    unsafe { native_surface.create_win32_surface(&native_surface_create, None) }.ok()
}

fn new_unix_xlib_surface(
    entry: &Entry,
    instance: &ash::Instance,
    display: &RawDisplayHandle,
    window: &RawWindowHandle,
) -> Option<vk::SurfaceKHR> {
    let RawDisplayHandle::Xlib(native_display) = display else {
        None?
    };
    let RawWindowHandle::Xlib(native_window) = window else {
        None?
    };
    let native_surface_create = vk::XlibSurfaceCreateInfoKHR::builder()
        .dpy(native_display.display as *mut *const std::ffi::c_void)
        .window(native_window.window)
        .build();
    let native_surface = ash::extensions::khr::XlibSurface::new(entry, instance);
    unsafe { native_surface.create_xlib_surface(&native_surface_create, None) }.ok()
}

fn new_unix_wayland_surface(
    entry: &Entry,
    instance: &ash::Instance,
    display: &RawDisplayHandle,
    window: &RawWindowHandle,
) -> Option<vk::SurfaceKHR> {
    let RawDisplayHandle::Wayland(native_display) = display else {
        None?
    };
    let RawWindowHandle::Wayland(native_window) = window else {
        None?
    };
    let native_surface_create = vk::WaylandSurfaceCreateInfoKHR::builder()
        .display(native_display.display)
        .surface(native_window.surface)
        .build();
    let native_surface = ash::extensions::khr::WaylandSurface::new(entry, instance);
    unsafe { native_surface.create_wayland_surface(&native_surface_create, None) }.ok()
}

#[allow(unused_variables)]
fn new_unix_macos_surface(
    entry: &Entry,
    instance: &ash::Instance,
    display: &RawDisplayHandle,
    window: &RawWindowHandle,
) -> Option<vk::SurfaceKHR> {
    #[cfg(target_os = "macos")]
    {
        use raw_window_metal::{appkit, Layer};

        let RawDisplayHandle::AppKit(_native_display) = display else {
            None?
        };
        let RawWindowHandle::AppKit(native_window) = window else {
            None?
        };
        let layer = match unsafe { appkit::metal_layer_from_handle(native_window.clone()) } {
            Layer::Existing(layer) | Layer::Allocated(layer) => layer.cast(),
            Layer::None => None?,
        };

        let native_surface_create =
            vk::MetalSurfaceCreateInfoEXT::builder().layer(unsafe { &*layer });
        let native_surface = extensions::ext::MetalSurface::new(entry, instance);
        return unsafe { native_surface.create_metal_surface(&native_surface_create, None) }.ok();
    }
    unreachable!()
}
