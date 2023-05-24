use super::*;

impl WebGpuContext {
    pub fn new_attachment_image(
        &mut self,
        initial_width: usize,
        initial_height: usize,
        attachment_usage: AttachmentImageUsage,
        ext: Option<NewAttachmentImageExt>,
    ) -> GResult<AttachmentImageId> {
        let attachment_image = WebGpuAttachmentImage::new(
            &self.device,
            initial_width,
            initial_height,
            attachment_usage,
            ext,
        );
        self.attachment_images.push(attachment_image);

        Ok(AttachmentImageId::from_id(self.attachment_images.len() - 1))
    }
}

pub struct WebGpuAttachmentImage {
    attachment_usage: AttachmentImageUsage,
    ext: NewAttachmentImageExt,

    texture: GpuTexture,
    pub texture_view: GpuTextureView,
}

impl WebGpuAttachmentImage {
    pub fn new(
        device: &GpuDevice,
        initial_width: usize,
        initial_height: usize,
        attachment_usage: AttachmentImageUsage,
        ext: Option<NewAttachmentImageExt>,
    ) -> Self {
        let ext = ext.unwrap_or_default();

        let format = match attachment_usage {
            AttachmentImageUsage::ColorAttachment => WEBGPU_COLOR_ATTACHMENT_FORMAT,
            AttachmentImageUsage::DepthAttachment => WEBGPU_DEPTH_ATTACHMENT_FORMAT,
        };

        let size = Array::new();
        size.push(&JsValue::from(initial_width));
        size.push(&JsValue::from(initial_height));

        let usage = GpuTextureUsageFlags::RenderAttachment as u32
            | GpuTextureUsageFlags::TextureBinding as u32;

        let mut texture_info = GpuTextureDescriptor::new(format, &size, usage);
        texture_info.sample_count(match ext.msaa_samples.unwrap_or_default() {
            MsaaSampleCount::Sample1 => 1,
            MsaaSampleCount::Sample2 => 2,
            MsaaSampleCount::Sample4 => 4,
            MsaaSampleCount::Sample8 => 8,
            MsaaSampleCount::Sample16 => 16,
            MsaaSampleCount::Sample32 => 32,
            MsaaSampleCount::Sample64 => 64,
        });

        let texture = device.create_texture(&texture_info);
        let texture_view = texture.create_view();

        WebGpuAttachmentImage {
            ext,
            attachment_usage,
            texture,
            texture_view,
        }
    }

    pub fn recreate_with_new_size(&mut self, device: &GpuDevice, width: usize, height: usize) {
        let new_attachment_image = WebGpuAttachmentImage::new(
            device,
            width,
            height,
            self.attachment_usage,
            Some(self.ext),
        );
        *self = new_attachment_image;
    }
}
