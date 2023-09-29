//! Since not all platforms are created equal, extensions exist to use special features or eak out
//! more performance.
//! See [`Extensions`] for details of each extension.
//!
//! ## Extension Support
//!
//! ✅ = Fully Supported
//! 🆗  = Has No Effect
//! ❌ = Unsupported
//!
//!  
//! | Extension                         | Vulkan    | WebGpu    |
//! | ---                               | ---       | ---       |
//! | `flight_frames_count`             | ✅        | 🆗        |
//! | `gpu_power_level`                 | ✅        | 🆗        |
//! | `native_debug`                    | ✅        | 🆗        |
//! | `memory_flush`                    | ✅        | 🆗        |
//! | `naga_translation`                | ✅        | ✅        |
//! | `webgpu_init_from_window`         | 🆗        | ✅        |
//! | `webgpu_init`                     | 🆗        | ✅        |
//! | `surface_extension`               | ✅        | ✅        |
//! | `compute`                         | ✅        | ✅        |
//! | `shader_storage_buffer_object`    | ✅        | ✅        |
//!

pub mod compute;
pub mod gpu_power_level;
pub mod memory_flush;
pub mod native_debug;
pub mod shader_storage_buffer_object;
pub mod webgpu_init;
pub mod webgpu_init_from_window;

#[cfg(feature = "surface_extension")]
pub mod surface;

#[cfg(feature = "naga_translation")]
pub mod naga_translation;

pub use compute::{
    CompileComputePassExt, ComputePass, ComputePassSubmitData, Dispatch, DispatchType,
    NewComputeProgramExt,
};
pub use gpu_power_level::GpuPowerLevel;
pub use native_debug::NativeDebugConfiguration;
pub use shader_storage_buffer_object::{
    NewShaderStorageBufferExt, ReadSyncedShaderStorageBufferExt, ShaderStorageBufferId,
};
pub use webgpu_init::WebGpuInit;
pub use webgpu_init_from_window::WebGpuInitFromWindow;

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
    /// 2-3 frames should suffice.
    /// Also, this typically does not apply for compute workflows.
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
    /// Invoke using [`Context::flush_memory`].
    pub fn memory_flush(&mut self) -> &mut Self {
        self.extensions.push(Extension::MemoryFlush);
        self
    }

    /// Translate from one shader language to another via [`naga`](https://github.com/gfx-rs/naga).
    /// Invoke using [`Context::naga_translate_shader_code`].
    /// Requires that the `naga_translation` feature is enabled for you project.
    #[cfg(feature = "naga_translation")]
    pub fn naga_translation(&mut self) -> &mut Self {
        self.extensions.push(Extension::NagaTranslation);
        self
    }

    /// Initialize the WebGpu Context.
    /// You will need to use [`Context::async_new`] to use this.
    pub fn webgpu_init(&mut self, init: WebGpuInit) -> &mut Self {
        self.extensions.push(Extension::WebGpuInit(init));
        self
    }

    /// Workaround required to initialize the WebGpu Context without async.
    /// This extension is not compatible to [`Context::new`].
    pub fn webgpu_init_from_window(&mut self, init: WebGpuInitFromWindow) -> &mut Self {
        self.extensions.push(Extension::WebGpuInitFromWindow(init));
        self
    }

    /// Rendering to the screen.
    /// Enable this unless you plan to run headlessly.
    /// Be sure to invoke [`Context::set_surface_size`] properly.
    /// Requires that the `surface_extension` feature is enabled for you project.
    #[cfg(feature = "surface_extension")]
    pub fn surface_extension(&mut self, cfg: SurfaceConfiguration) -> &mut Self {
        self.extensions.push(Extension::Surface(cfg));
        self
    }

    /// Enable compute support.
    /// Note that using feature without this extension may or may not work.
    pub fn compute(&mut self) -> &mut Self {
        self.extensions.push(Extension::Compute);
        self
    }

    /// Enable shader storage buffer objects.
    /// Note that using this feature without this extension may or may not work.
    pub fn shader_storage_buffer_object(&mut self) -> &mut Self {
        self.extensions.push(Extension::ShaderStorageBufferObject);
        self
    }
}

#[derive(Debug, Clone)]
pub enum Extension {
    FlightFramesCount(usize),
    GpuPowerLevel(GpuPowerLevel),
    NativeDebug(NativeDebugConfiguration),
    MemoryFlush,
    NagaTranslation,
    WebGpuInitFromWindow(WebGpuInitFromWindow),
    WebGpuInit(WebGpuInit),
    Surface(SurfaceConfiguration),
    Compute,
    ShaderStorageBufferObject,
}
