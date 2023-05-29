use super::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    //  80% of gpus with vulkan don't support Rgb.
    //  Rgb,
    Rgba,
}

#[derive(Debug, Clone, Copy)]
pub enum AttachmentImageColorFormat {
    R8UNorm,
    R8G8UNorm,
    //  93% of gpus with vulkan don't support Rgb.
    //  R8G8B8UNorm,
    R8G8B8A8UNorm,

    R32SFloat,
    R32G32SFloat,
    //  85% of gpus with vulkan don't support Rgb.
    //  R32G32B32SFloat,
    R32G32B32A32SFloat,
}

#[derive(Debug, Clone, Copy)]
pub enum AttachmentImageUsage {
    ColorAttachment,
    DepthAttachment,
}

#[derive(Default, Clone)]
pub struct NewTextureExt {
    pub mip_levels: Option<u32>,
}
#[derive(Default, Clone)]
pub struct UploadTextureExt {
    /// Generate mipmips.
    /// The mipmap count can be contained with [`Context::get_texture_max_lod`]
    pub generate_mipmaps: Option<()>,
}
#[derive(Default, Clone)]
pub struct NewAttachmentImageExt {
    /// Should match the mssa samples used in [`CompilePassExt`].
    /// Or, the value can be obtained
    pub msaa_samples: Option<MsaaSampleCount>,
    /// Optionaly specify the format used by the attachment image if the image is used as a color
    /// attachment.
    pub color_format: Option<AttachmentImageColorFormat>,
}

impl Context {
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        format: TextureFormat,
        ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        match self {
            Self::Vulkan(vk) => vk.new_texture(width, height, format, ext),
            Self::WebGpu(wgpu) => wgpu.new_texture(width, height, format, ext),
        }
    }

    pub fn upload_texture(
        &mut self,
        texture: TextureId,
        data: &[u8],
        ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.upload_texture(texture, data, ext),
            Self::WebGpu(wgpu) => wgpu.upload_texture(texture, data, ext),
        }
    }

    pub fn get_texture_max_lod(&self, texture: TextureId) -> GResult<f32> {
        match self {
            Self::Vulkan(vk) => vk.get_texture_max_lod(texture),
            Self::WebGpu(wgpu) => wgpu.get_texture_max_lod(texture),
        }
    }

    pub fn new_attachment_image(
        &mut self,
        initial_width: usize,
        initial_height: usize,
        attachment_usage: AttachmentImageUsage,
        ext: Option<NewAttachmentImageExt>,
    ) -> GResult<AttachmentImageId> {
        match self {
            Self::Vulkan(vk) => {
                vk.new_attachment_image(initial_width, initial_height, attachment_usage, ext)
            }
            Self::WebGpu(wgpu) => {
                wgpu.new_attachment_image(initial_width, initial_height, attachment_usage, ext)
            }
        }
    }
}
