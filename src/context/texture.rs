use super::*;

#[derive(Clone, Copy)]
pub enum TextureFormat {
    //  80% of gpus with vulkan don't support Rgb.
    //  Rgb,
    Rgba,
}

#[derive(Clone, Copy)]
pub enum TextureAttachmentUsage {
    ColorAttachment,
    DepthAttachment,
}

#[derive(Default, Clone, Copy)]
pub struct NewTextureExt {
    pub attachment_usage: Option<TextureAttachmentUsage>,
    pub depends_on_surface_size: bool,
}
#[derive(Default)]
pub struct ResizeTextureExt {}
#[derive(Default)]
pub struct UploadTextureExt {}

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
}
