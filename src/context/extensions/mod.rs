//! Since not all platforms are created equal, extensions exist to use special features or eak out
//! more performance.
//! See [`Extension`] for details of each extension.

pub mod gpu_power_level;
pub mod memory_flush;

#[cfg(feature = "shader_reflection_extension")]
pub mod shader_reflection;
#[cfg(feature = "surface_extension")]
pub mod surface;

use super::*;

//  TODO EXT: List of future extensions:
//  - Named Buffers
//  - ForceSPIRV
//  - Raytracing

#[derive(Clone)]
pub enum Extension {
    /// Configure how many frames ahead the gpu runs ahead.
    /// 2-3 should suffice.
    FlightFramesCount(usize),
    /// Prefer Integrated vs Discrete?
    GpuPowerLevel(gpu_power_level::GpuPowerLevel),
    /// Api dependent debug logs.
    NativeDebug,
    /// Explicitly clear out unused gpu memory.
    MemoryFlush,
    /// Rendering to the screen.
    /// Enable this unless you plan to run headlessly.
    #[cfg(feature = "surface_extension")]
    Surface(surface::SurfaceConfiguration),
    /// Auto-infer out a [`ShaderType`].
    #[cfg(feature = "shader_reflection_extension")]
    ShaderReflection,
}

/// Look at [`Extension`] for details on each extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionType {
    FlightFramesCount,
    GpuPowerLevel,
    NativeDebug,
    MemoryFlush,
    #[cfg(feature = "surface_extension")]
    Surface,
    #[cfg(feature = "shader_reflection_extension")]
    ShaderReflection,
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
            #[cfg(feature = "shader_reflection_extension")]
            Self::ShaderReflection => ExtensionType::ShaderReflection,
        }
    }
}

impl Context {
    pub fn extension_is_enabled(&self, ty: ExtensionType) -> bool {
        match self {
            Self::Vulkan(vk) => vk.extension_is_enabled(ty),
            Self::WebGpu(wgpu) => wgpu.extension_is_enabled(ty),
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
