use super::*;

pub enum ImageUsage {
    ColorAttachment,
    DepthAttachment,
}

impl Context {
    pub fn new_image(
        &mut self,
        width: usize,
        height: usize,
        usage: ImageUsage,
    ) -> GResult<ImageId> {
        match self {
            Self::Vulkan(vk) => vk.new_image(width, height, usage),
        }
    }
}
