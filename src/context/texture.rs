use super::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TextureFormat {
    //  80% of gpus with vulkan don't support Rgb.
    //  Rgb,
    Rgba,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum AttachmentImageUsage {
    ColorAttachment,
    DepthAttachment,
}

pub struct CubemapTextureUpload<'a> {
    pub posx: &'a [u8],
    pub negx: &'a [u8],
    pub posy: &'a [u8],
    pub negy: &'a [u8],
    pub posz: &'a [u8],
    pub negz: &'a [u8],
}

/// Allows for the configuration of:
/// - Mipmaps
/// - Cubemap
#[derive(Default, Debug)]
pub struct NewTextureExt {
    pub enable_mipmaps: Option<()>,
    pub mip_levels: Option<u32>,
    pub enable_cubemap: Option<()>,
}

/// Allows for the configuration of:
/// - Whether to generate mipmaps
#[derive(Default, Debug, Clone)]
pub struct UploadTextureExt {
    /// Generate mipmips.
    /// The mipmap count can be obtained with [`Context::get_texture_max_lod`].
    pub generate_mipmaps: Option<()>,
}

/// Allows for the configuration of:
/// - Whether to generate mipmaps
#[derive(Default, Debug, Clone)]
pub struct UploadCubemapTextureExt {
    /// Generate mipmips.
    /// The mipmap count can be obtained with [`Context::get_texture_max_lod`].
    pub generate_mipmaps: Option<()>,
}

/// Allows for the configuration of:
/// - MSAA samples
/// - (Color) attachment image format.
#[derive(Default, Debug, Clone)]
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

    pub fn upload_cubemap_texture(
        &mut self,
        texture: TextureId,
        upload: CubemapTextureUpload,
        ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.upload_cubemap_texture(texture, upload, ext),
            Self::WebGpu(wgpu) => wgpu.upload_cubemap_texture(texture, upload, ext),
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
