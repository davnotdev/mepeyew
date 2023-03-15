use super::*;

impl Context {
    pub fn memory_flush_extension_flush_memory(&mut self) {
        self.assert_extension_enabled(ExtensionType::MemoryFlush);
        match self {
            Self::Vulkan(vk) => vk.memory_flush_extension_flush_memory(),
        }
    }
}
