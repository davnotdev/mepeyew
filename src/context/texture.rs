use super::*;

#[derive(Clone, Copy)]
pub enum TextureFormat {
    //  80% of gpus with vulkan don't support Rgb.
    //  Rgb,
    Rgba,
}

#[derive(Clone, Copy)]
pub enum AttachmentImageUsage {
    ColorAttachment,
    DepthAttachment,
}

#[derive(Default, Clone, Copy)]
pub struct NewTextureExt {}
#[derive(Default, Clone, Copy)]
pub struct ResizeTextureExt {}
#[derive(Default, Clone, Copy)]
pub struct UploadTextureExt {}
#[derive(Default, Clone, Copy)]
pub struct NewAttachmentImageExt {}

impl Context {
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        sampler: SamplerId,
        format: TextureFormat,
        ext: NewTextureExt,
    ) -> GResult<TextureId> {
        match self {
            Self::Vulkan(vk) => vk.new_texture(width, height, sampler, format, ext),
        }
    }

    pub fn resize_texture(
        &mut self,
        texture: TextureId,
        width: usize,
        height: usize,
        ext: ResizeTextureExt,
    ) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.resize_texture(texture, width, height, ext),
        }
    }

    pub fn upload_texture(
        &mut self,
        texture: TextureId,
        data: &[u8],
        ext: UploadTextureExt,
    ) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.upload_texture(texture, data, ext),
        }
    }

    pub fn new_attachment_image(
        &mut self,
        initial_width: usize,
        initial_height: usize,
        attachment_usage: AttachmentImageUsage,
        ext: NewAttachmentImageExt,
    ) -> GResult<AttachmentImageId> {
        match self {
            Self::Vulkan(vk) => {
                vk.new_attachment_image(initial_width, initial_height, attachment_usage, ext)
            }
        }
    }
}