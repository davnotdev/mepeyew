use super::*;

pub enum ImageUsage {
    ColorAttachment,
    DepthAttachment,
}

#[derive(Default)]
pub struct NewImageExt {}

impl Context {
    pub fn new_image(
        &mut self,
        width: usize,
        height: usize,
        usage: ImageUsage,
        ext: NewImageExt,
    ) -> GResult<ImageId> {
        match self {
            Self::Vulkan(vk) => vk.new_image(width, height, usage, ext),
        }
    }
}
