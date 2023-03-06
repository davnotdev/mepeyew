pub mod gpu_power_level;
pub mod memory_flush;
pub mod surface;

use super::*;

//  TODO EXT: List of future extensions:
//  - Named Buffers
//  - ForceSPIRV
//  - Raytracing

pub enum Extension {
    Surface(surface::SurfaceConfiguration),
    FlightFramesCount(usize),
    GpuPowerLevel(gpu_power_level::GpuPowerLevel),
    NativeDebug,
    MemoryFlush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionType {
    Surface,
    FlightFramesCount,
    GpuPowerLevel,
    NativeDebug,
    MemoryFlush,
}

impl Extension {
    pub fn get_type(&self) -> ExtensionType {
        match self {
            Self::Surface(_) => ExtensionType::Surface,
            Self::FlightFramesCount(_) => ExtensionType::FlightFramesCount,
            Self::GpuPowerLevel(_) => ExtensionType::GpuPowerLevel,
            Self::NativeDebug => ExtensionType::NativeDebug,
            Self::MemoryFlush => ExtensionType::MemoryFlush,
        }
    }
}
