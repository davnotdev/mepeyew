use super::context::{self, *};
use super::error::{gpu_api_err, GResult, GpuError};
use std::collections::HashSet;

mod attachment_image;
mod buffer;
mod extensions;
mod pass;
mod program;
mod sampler;
mod submit;
mod texture;

pub struct WebGpuContext {
    enabled_extensions: HashSet<ExtensionType>,
}

impl WebGpuContext {
    pub fn new(extensions: &[Extension]) -> GResult<Self> {
        let supported_extensions = extensions::supported_extensions();
        let (enabled_extensions, unsupported_extensions): (Vec<_>, Vec<_>) = extensions
            .iter()
            .map(|ext| ext.get_type())
            .partition(|ty| supported_extensions.contains(ty));
        let enabled_extensions = enabled_extensions.into_iter().collect::<HashSet<_>>();
        if !unsupported_extensions.is_empty() {
            Err(gpu_api_err!(
                "vulkan these extensions not supported: {:?}",
                unsupported_extensions
            ))?;
        }

        todo!()
    }
}
