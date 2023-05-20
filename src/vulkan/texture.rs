use super::*;

impl VkContext {
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        format: TextureFormat,
        ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        let texture = VkTexture::new(
            self,
            width,
            height,
            format,
            ext.unwrap_or_default(),
        )?;
        self.textures.push(texture);
        Ok(TextureId::from_id(self.textures.len() - 1))
    }

    pub fn upload_texture(
        &mut self,
        texture: TextureId,
        data: &[u8],
        ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        let texture = self.textures.get_mut(texture.id()).ok_or(gpu_api_err!(
            "vulkan upload texture {:?} doesn't exist",
            texture
        ))?;
        texture.upload(&self.core, data, ext.unwrap_or_default())
    }
}

pub struct VkTexture {
    width: usize,
    height: usize,
    format: TextureFormat,
    ext: NewTextureExt,
    aspect: vk::ImageAspectFlags,

    pub image: VkImage,
    staging: VkBuffer,
    pub image_view: vk::ImageView,

    drop_queue_ref: VkDropQueueRef,
}

impl VkTexture {
    fn new(
        context: &mut VkContext,
        width: usize,
        height: usize,
        format: TextureFormat,
        ext: NewTextureExt,
    ) -> GResult<Self> {
        let vkformat = match format {
            // TextureFormat::Rgb => vk::Format::R8G8B8_UNORM,
            // TODO FIX: This should be R8G8B8A8_SRGB, shouldn't it?
            TextureFormat::Rgba => vk::Format::R8G8B8A8_UNORM,
        };

        let vkusages = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
        let aspect = vk::ImageAspectFlags::COLOR;

        let image = VkImage::new(
            &context.core.dev,
            &context.drop_queue,
            &mut context.alloc,
            vkformat,
            vkusages,
            aspect,
            vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
        )?;

        let image_view = new_image_view(&context.core.dev, image.image, vkformat, aspect)?;

        let per_pixel_byte_size = match format {
            // TextureFormat::Rgb => 3,
            TextureFormat::Rgba => 4,
        };

        let staging = VkBuffer::new(
            &context.core.dev,
            &context.drop_queue,
            &mut context.alloc,
            per_pixel_byte_size * width * height,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;

        Ok(VkTexture {
            width,
            height,
            aspect,

            image,
            staging,
            image_view,
            format,
            ext,
            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
    }

    fn upload(&mut self, core: &VkCore, data: &[u8], _ext: UploadTextureExt) -> GResult<()> {
        self.staging
            .map_copy_data(data.as_ptr() as *const u8, data.len())?;

        let _misc_command = core.misc_command();

        let range = vk::ImageSubresourceRange::builder()
            .aspect_mask(self.aspect)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let image_transfer_barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(self.image.image)
            .subresource_range(range)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .build();

        unsafe {
            core.dev.cmd_pipeline_barrier(
                core.misc_command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_transfer_barrier],
            );
        }

        let image_subresource_layers = vk::ImageSubresourceLayers::builder()
            .aspect_mask(self.aspect)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();
        let copy_region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_extent(vk::Extent3D {
                width: self.width as u32,
                height: self.height as u32,
                depth: 1,
            })
            .image_subresource(image_subresource_layers)
            .build();

        unsafe {
            core.dev.cmd_copy_buffer_to_image(
                core.misc_command_buffer,
                self.staging.buffer,
                self.image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy_region],
            );
        }

        //  TODO FIX: Support depth as well.
        let mut image_use_barrier = image_transfer_barrier;
        image_use_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_use_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_use_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_use_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            core.dev.cmd_pipeline_barrier(
                core.misc_command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_use_barrier],
            );
        }

        Ok(())
    }
}

impl Drop for VkTexture {
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
