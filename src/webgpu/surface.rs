use super::*;

pub struct WebGpuSurface {
    pub canvas: HtmlCanvasElement,
    pub context: GpuCanvasContext,
    pub present_format: GpuTextureFormat,
}
