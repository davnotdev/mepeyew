use super::*;
use context::extensions::WebGpuInitFromWindow;

impl WebGpuContext {
    pub fn init_from_window(
        init: WebGpuInitFromWindow,
    ) -> GResult<(GpuAdapter, GpuDevice, Option<String>)> {
        let window = window().unwrap();
        let window_flabby: &JsValue = &window;

        let adapter_key = JsValue::from_str(&init.adapter);
        let device_key = JsValue::from_str(&init.device);

        let adapter = Reflect::get(window_flabby, &adapter_key)
            .map_err(|e| gpu_api_err!("webgpu window.{} does not exist: {:?}", init.adapter, e))?
            .dyn_into::<GpuAdapter>()
            .map_err(|e| {
                gpu_api_err!(
                    "webgpu window.{} is not a GPUAdapter: {:?}",
                    init.adapter,
                    e
                )
            })?;
        let device = Reflect::get(window_flabby, &device_key)
            .map_err(|e| gpu_api_err!("webgpu window.{} does not exist: {:?}", init.device, e))?
            .dyn_into::<GpuDevice>()
            .map_err(|e| {
                gpu_api_err!("webgpu window.{} is not a GPUDevice: {:?}", init.device, e)
            })?;

        Ok((adapter, device, init.canvas_id))
    }
}
