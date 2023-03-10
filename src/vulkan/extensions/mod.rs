mod memory_flush;
mod surface_ext;

use super::*;

pub use surface_ext::VkSurfaceExt;

pub fn supported_extensions() -> &'static [ExtensionType] {
    &[
        ExtensionType::Surface,
        ExtensionType::FlightFramesCount,
        ExtensionType::GpuPowerLevel,
        ExtensionType::NativeDebug,
        ExtensionType::MemoryFlush,
    ]
}

impl VkContext {
    pub fn extension_is_enabled(&self, ty: ExtensionType) -> bool {
        self.enabled_extensions.contains(&ty)
    }
}
