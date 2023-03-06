use super::*;

impl VkContext {
    pub fn memory_flush_extension_flush_memory(&mut self) {
        self.flush_memory();
    }
}
