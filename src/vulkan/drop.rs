use super::*;

pub type VkDropQueueCallback = Box<dyn FnOnce(&Device, &mut Allocator)>;

#[derive(Default)]
pub struct VkDropQueue {
    rms: Vec<VkDropQueueCallback>,
}

impl VkDropQueue {
    pub fn new() -> Self {
        VkDropQueue { rms: Vec::new() }
    }

    pub fn idle_flush(&mut self, dev: &Device, alloc: &mut Allocator) {
        self.rms
            .drain(0..self.rms.len())
            .for_each(|rm| (rm)(dev, alloc));
    }

    pub fn push(&mut self, callback: VkDropQueueCallback) {
        self.rms.push(callback);
    }
}
