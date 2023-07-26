use super::*;
use context::extensions::WebGpuInit;

impl WebGpuContext {
    pub async fn init(init: WebGpuInit) -> GResult<(GpuAdapter, GpuDevice, Option<String>)> {
        let navigator = window().unwrap().navigator();
        let adapter: GpuAdapter = JsFuture::from(navigator.gpu().request_adapter())
            .await
            .map_err(|e| {
                gpu_api_err!(
                    "webgpu: cannot request adapter (webgpu may not be supported): {:?}",
                    e
                )
            })?
            .into();
        let device: GpuDevice = JsFuture::from(adapter.request_device())
            .await
            .map_err(|e| {
                gpu_api_err!(
                    "webgpu: cannot request device (webgpu may not be supported):{:?}",
                    e
                )
            })?
            .into();
        Ok((adapter, device, init.canvas_id))
    }
}
