pub mod gpu_power_level;
pub mod memory_flush;

#[cfg(feature = "surface_extension")]
pub mod surface;

use super::*;

//  TODO EXT: List of future extensions:
//  - Named Buffers
//  - ForceSPIRV
//  - Raytracing

pub enum Extension {
    FlightFramesCount(usize),
    GpuPowerLevel(gpu_power_level::GpuPowerLevel),
    NativeDebug,
    MemoryFlush,
    #[cfg(feature = "surface_extension")]
    Surface(surface::SurfaceConfiguration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    FlightFramesCount,
    GpuPowerLevel,
    NativeDebug,
    MemoryFlush,
    #[cfg(feature = "surface_extension")]
    Surface,
}

impl Extension {
    pub fn get_type(&self) -> ExtensionType {
        match self {
            Self::FlightFramesCount(_) => ExtensionType::FlightFramesCount,
            Self::GpuPowerLevel(_) => ExtensionType::GpuPowerLevel,
            Self::NativeDebug => ExtensionType::NativeDebug,
            Self::MemoryFlush => ExtensionType::MemoryFlush,
            #[cfg(feature = "surface_extension")]
            Self::Surface(_) => ExtensionType::Surface,
        }
    }
}

impl Context {
    pub fn extension_is_enabled(&self, ty: ExtensionType) -> bool {
        match self {
            Self::Vulkan(vk) => vk.extension_is_enabled(ty),
        }
    }

    #[cfg(feature = "assert_extensions")]
    fn assert_extension_enabled(&self, ty: ExtensionType) {
        assert!(self.extension_is_enabled(ty))
    }
    #[cfg(not(feature = "assert_extensions"))]
    fn assert_extension_enabled(&self, ty: ExtensionType) {
        assert!(self.extension_is_enabled(ty))
    }
}
