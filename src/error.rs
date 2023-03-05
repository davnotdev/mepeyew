pub type GResult<T> = std::result::Result<T, GpuError>;

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
