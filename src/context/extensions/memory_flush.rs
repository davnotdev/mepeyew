use super::*;

impl Context {
    pub fn flush_memory(&mut self) {
        match self {
            Self::Vulkan(vk) => vk.memory_flush_extension_flush_memory(),
            Self::WebGpu(_) => (),
        }
    }
}
