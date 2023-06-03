mod memory_flush;
mod surface;

use super::*;

pub use surface::VkSurfaceExt;

pub fn check_extensions(extensions: &Extensions) -> GResult<()> {
    extensions
        .extensions
        .iter()
        .try_for_each(|extension| match extension {
            Extension::FlightFramesCount(_) => Ok(()),
            Extension::GpuPowerLevel(_) => Ok(()),
            Extension::NativeDebug(_) => Ok(()),
            Extension::MemoryFlush => Ok(()),
            Extension::NagaTranslation => Ok(()),
            Extension::WebGpuInitFromWindow(_) => Ok(()),
            Extension::Surface(_) => Ok(()),
        })
}
