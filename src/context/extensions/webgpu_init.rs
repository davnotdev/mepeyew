/// This requires async to be present.
/// Use [`super::WebGpuInitFromWindow`] otherwise.
#[derive(Debug, Clone)]
pub struct WebGpuInit {
    /// Used if you indend on rendering to a canvas.
    /// ```html
    /// <canvas id="renderhere"></canvas>
    ///             ^canvas_id
    /// ```
    pub canvas_id: Option<String>,
}
