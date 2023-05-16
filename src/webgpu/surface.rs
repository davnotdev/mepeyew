use super::*;

pub struct WebGpuSurface {
    pub context: GpuCanvasContext,
    pub present_format: GpuTextureFormat,
}
