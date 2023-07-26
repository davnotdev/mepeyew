pub mod compute;
mod shader_storage_buffer_object;
mod surface;
mod webgpu_init;
mod webgpu_init_from_window;

use super::*;

pub fn check_extensions(extensions: &Extensions, is_async: bool) -> GResult<()> {
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
            Extension::WebGpuInit(_) =>
                if is_async {
                    Ok(())
                } else {
                    Err(gpu_api_err!(
                        "webgpu: WebGpuInit is only supported in an async context, use WebGpuInitFromWindow if async cannot be used")
                    )
                },
            Extension::Surface(_) => Ok(()),
            Extension::Compute => Ok(()),
            Extension::ShaderStorageBufferObject => Err(gpu_api_err!(
                "webgpu: shader storage buffer objects are not fully supported"
            )),
        })
}
