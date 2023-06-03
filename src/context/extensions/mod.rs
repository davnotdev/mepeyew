//! Since not all platforms are created equal, extensions exist to use special features or eak out
//! more performance.
//! See [`Extension`] for details of each extension.

pub mod gpu_power_level;
pub mod memory_flush;
pub mod native_debug;
pub mod webgpu_init;

#[cfg(feature = "surface_extension")]
pub mod surface;

#[cfg(feature = "naga_translation")]
pub mod naga_translation;

pub use gpu_power_level::*;
pub use memory_flush::*;
pub use native_debug::*;
pub use webgpu_init::*;

#[cfg(feature = "surface_extension")]
pub use surface::*;

#[cfg(feature = "naga_translation")]
pub use naga_translation::*;

use super::*;

#[derive(Default, Debug, Clone)]
pub struct Extensions {
    pub(crate) extensions: Vec<Extension>,
}

impl Extensions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure how many frames ahead the gpu runs ahead.
    /// 2-3 should suffice.
    pub fn flight_frames_count(&mut self, count: usize) -> &mut Self {
        self.extensions.push(Extension::FlightFramesCount(count));
        self
    }

    /// Prefer Integrated vs Discrete?
    pub fn gpu_power_level(&mut self, power_level: GpuPowerLevel) -> &mut Self {
        self.extensions.push(Extension::GpuPowerLevel(power_level));
        self
    }

    /// Api dependent debug logs.
    pub fn native_debug(&mut self, cfg: NativeDebugConfiguration) -> &mut Self {
        self.extensions.push(Extension::NativeDebug(cfg));
        self
    }

    /// Explicitly clear out unused gpu memory.
    /// Invoke using [`Context::memory_flush_extension_flush_memory`].
    pub fn memory_flush(&mut self) -> &mut Self {
        self.extensions.push(Extension::MemoryFlush);
        self
    }

    /// Translate from one shader language to another via [`naga`](https://github.com/gfx-rs/naga).
    /// Invoke using [`Context::naga_translation_extension_translate_shader_code`].
    #[cfg(feature = "naga_translation")]
    pub fn naga_translation(&mut self) -> &mut Self {
        self.extensions.push(Extension::NagaTranslation);
        self
    }

    /// Currently required to initialize the WebGpu Context.
    pub fn webgpu_init_from_window(&mut self, init: WebGpuInitFromWindow) -> &mut Self {
        self.extensions.push(Extension::WebGpuInitFromWindow(init));
        self
    }

    /// Rendering to the screen.
    /// Enable this unless you plan to run headlessly.
    /// Be sure to invoke [Context::surface_extension_set_surface_size] properly.
    pub fn surface_extension(&mut self, cfg: SurfaceConfiguration) -> &mut Self {
        self.extensions.push(Extension::Surface(cfg));
        self
    }
}

#[derive(Debug, Clone)]
pub enum Extension {
    FlightFramesCount(usize),
    GpuPowerLevel(GpuPowerLevel),
    NativeDebug(NativeDebugConfiguration),
    MemoryFlush,
    #[cfg(feature = "naga_translation")]
    NagaTranslation,
    WebGpuInitFromWindow(WebGpuInitFromWindow),
    #[cfg(feature = "surface_extension")]
    Surface(SurfaceConfiguration),
}
