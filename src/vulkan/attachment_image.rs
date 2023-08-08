use super::*;

impl VkContext {
    pub fn new_attachment_image(
        &mut self,
        initial_width: usize,
        initial_height: usize,
        attachment_usage: AttachmentImageUsage,
        ext: Option<NewAttachmentImageExt>,
    ) -> GResult<AttachmentImageId> {
        let attachment_image = VkAttachmentImage::new(
            &self.core.dev,
            &self.drop_queue,
            &mut self.alloc,
            initial_width,
            initial_height,
            attachment_usage,
            ext.unwrap_or_default(),
        )?;
        self.attachment_images.push(attachment_image);

        Ok(AttachmentImageId::from_id(self.attachment_images.len() - 1))
    }
}

pub struct VkAttachmentImage {
    pub ext: NewAttachmentImageExt,

    pub image: VkImage,
    pub format: vk::Format,
    pub image_view: vk::ImageView,
    pub attachment_usage: AttachmentImageUsage,

    drop_queue_ref: VkDropQueueRef,
}

impl VkAttachmentImage {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        width: usize,
        height: usize,
        attachment_usage: AttachmentImageUsage,
        ext: NewAttachmentImageExt,
    ) -> GResult<Self> {
        let format = match attachment_usage {
            AttachmentImageUsage::ColorAttachment => ext
                .color_format
                .map(|color_format| match color_format {
                    AttachmentImageColorFormat::R8UNorm => vk::Format::R8G8_UNORM,
                    AttachmentImageColorFormat::R8G8UNorm => vk::Format::R8G8_UNORM,
                    AttachmentImageColorFormat::R8G8B8A8UNorm => vk::Format::R8G8B8A8_UNORM,
                    AttachmentImageColorFormat::R32SFloat => vk::Format::R32_SFLOAT,
                    AttachmentImageColorFormat::R32G32SFloat => vk::Format::R32G32_SFLOAT,
                    AttachmentImageColorFormat::R32G32B32A32SFloat => {
                        vk::Format::R32G32B32A32_SFLOAT
                    }
                })
                .unwrap_or(VK_COLOR_ATTACHMENT_FORMAT),
            AttachmentImageUsage::DepthAttachment => VK_DEPTH_ATTACHMENT_FORMAT,
        };
        let usages = vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC
            | match attachment_usage {
                AttachmentImageUsage::ColorAttachment => vk::ImageUsageFlags::COLOR_ATTACHMENT,
                AttachmentImageUsage::DepthAttachment => {
                    vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                }
            };
        let aspect = match attachment_usage {
            AttachmentImageUsage::ColorAttachment => vk::ImageAspectFlags::COLOR,
            AttachmentImageUsage::DepthAttachment => vk::ImageAspectFlags::DEPTH,
        };

        let image = VkImage::new(
            dev,
            drop_queue_ref,
            alloc,
            format,
            usages,
            aspect,
            match ext.msaa_samples.unwrap_or_default() {
                MsaaSampleCount::Sample1 => vk::SampleCountFlags::TYPE_1,
                MsaaSampleCount::Sample2 => vk::SampleCountFlags::TYPE_2,
                MsaaSampleCount::Sample4 => vk::SampleCountFlags::TYPE_4,
                MsaaSampleCount::Sample8 => vk::SampleCountFlags::TYPE_8,
                MsaaSampleCount::Sample16 => vk::SampleCountFlags::TYPE_16,
                MsaaSampleCount::Sample32 => vk::SampleCountFlags::TYPE_32,
                MsaaSampleCount::Sample64 => vk::SampleCountFlags::TYPE_64,
            },
            1,
            false,
            vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
        )?;

        let image_view = new_image_view(dev, image.image, format, aspect, 1, false)?;

        Ok(VkAttachmentImage {
            ext,
            attachment_usage,
            format,
            image,
            image_view,
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }

    pub fn resize(
        &mut self,
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        width: usize,
        height: usize,
    ) -> GResult<()> {
        let new_attachment_image = VkAttachmentImage::new(
            dev,
            drop_queue_ref,
            alloc,
            width,
            height,
            self.attachment_usage,
            self.ext.clone(),
        )?;

        let _drop_old = std::mem::replace(self, new_attachment_image);
        Ok(())
    }
}

impl Drop for VkAttachmentImage {
    fn drop(&mut self) {
        let image_view = self.image_view;

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_image_view(image_view, None);
            }))
    }
}
