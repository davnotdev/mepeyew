/// All returned errors are wrapped in [`GResult`].
pub type GResult<T> = std::result::Result<T, GpuError>;

/// Errors in mepeyew are always strings since each platform can fail in its own way.
/// Note that most errors are NOT validated meaning that misuse of anything can cause undefined
/// behavior, crashes, and graphical glitches.
#[derive(Debug)]
pub struct GpuError {
    pub error: String,
    pub line: u32,
    pub file: &'static str,
}

#[macro_export]
macro_rules! gpu_api_err {
    ($($arg:tt)*) => {{
        GpuError {
            error: format!($($arg)*),
            line: line!(),
            file: file!(),
        }
    }};
}

pub use crate::gpu_api_err;
