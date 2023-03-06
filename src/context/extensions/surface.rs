use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct SurfaceConfiguration {
    pub width: usize,
    pub height: usize,
    pub display: RawDisplayHandle,
    pub window: RawWindowHandle,
}
