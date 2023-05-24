#[derive(Debug, Clone)]
pub struct WebGpuInitFromWindow {
    /// Created using these lines in javascript land.
    /// ```javascript
    /// let adapter = await navigator.gpu.requestAdapter();
    /// window.mepeyewAdapter = adapter;
    /// ```
    /// In this case "mepeyewAdapter" would be the value.
    pub adapter: String,
    /// Created using these lines in javascript land.
    /// ```javascript
    /// let device = await adapter.requestDevice();
    /// window.mepeyewDevice = device;
    /// ```
    /// In this case "mepeyewDevice" would be the value.
    pub device: String,
    /// Used if you indend on rendering to a canvas.
    /// ```html
    /// <canvas id="renderhere"></canvas>
    ///             ^canvas_id
    /// ```
    pub canvas_id: Option<String>,
}
