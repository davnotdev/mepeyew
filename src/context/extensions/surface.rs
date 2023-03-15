use super::*;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Clone, Copy)]
pub struct SurfaceConfiguration {
    pub width: usize,
    pub height: usize,
    pub display: RawDisplayHandle,
    pub window: RawWindowHandle,
}

impl Context {
    pub fn surface_extension_set_surface_size(
        &mut self,
        width: usize,
        height: usize,
    ) -> GResult<()> {
        self.assert_extension_enabled(ExtensionType::Surface);
        match self {
            Self::Vulkan(vk) => vk.surface_extension_set_surface_size(width, height),
        }
    }
}
