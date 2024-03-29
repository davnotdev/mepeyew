use super::*;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Debug, Clone, Copy)]
pub struct SurfaceConfiguration {
    pub width: usize,
    pub height: usize,
    pub display: RawDisplayHandle,
    pub window: RawWindowHandle,
}

impl Context {
    /// Call when the surface resizes.
    pub fn set_surface_size(&mut self, width: usize, height: usize) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.surface_extension_set_surface_size(width, height),
            Self::WebGpu(wgpu) => wgpu.surface_extension_set_surface_size(width, height),
        }
    }
}
