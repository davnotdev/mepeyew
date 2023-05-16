#[cfg(feature = "webgpu")]
use wasm_bindgen::JsValue;

#[cfg(not(feature = "webgpu"))]
type JsValue = ();

#[derive(Clone)]
pub struct WebGpuInit {
    /// Created using this line from javascript land.
    /// ```javascript
    /// let adapter = await navigator.gpu.requestAdapter();
    /// ```
    pub adapter: JsValue,
    /// Created using this line from javascript land.
    /// ```javascript
    /// let device = await adapter.requestDevice();
    /// ```
    pub device: JsValue,
    /// Used if you indend on rendering to a canvas.
    /// ```html
    /// <canvas id="renderhere"></canvas>
    ///             ^canvas_id
    /// ```
    pub canvas_id: Option<String>,
}
