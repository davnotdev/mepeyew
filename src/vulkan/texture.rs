use super::*;

impl VkContext {
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        format: TextureFormat,
        ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        let texture = VkTexture::new(self, width, height, format, ext.unwrap_or_default())?;
        self.textures.push(texture);
        Ok(TextureId::from_id(self.textures.len() - 1))
    }

    pub fn upload_texture(
        &mut self,
        texture: TextureId,
        data: &[u8],
        ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        let ext = ext.unwrap_or_default();
        let texture = self.textures.get_mut(texture.id()).ok_or(gpu_api_err!(
            "vulkan upload texture {:?} doesn't exist",
            texture
        ))?;
        let texture_width = texture.width;
        let texture_height = texture.height;
        let texture_image = texture.image.image;
        let texture_mip_level = texture.mip_levels;

        texture.upload(&self.core, data, ext.clone())?;

        if ext.generate_mipmaps.is_some() {
            unsafe {
                generate_mipmaps(
                    self,
                    texture_width,
                    texture_height,
                    texture_image,
                    texture_mip_level,
                )?;
            }
        }

        Ok(())
    }

    pub fn get_texture_max_lod(&self, texture: TextureId) -> GResult<f32> {
        let texture = self.textures.get(texture.id()).ok_or(gpu_api_err!(
            "vulkan get texture max lod: {:?} does not exist",
            texture
        ))?;
        Ok(texture.mip_levels as f32)
    }
}

pub struct VkTexture {
    width: usize,
    height: usize,
    mip_levels: u32,

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
        let mip_levels = if ext.enable_mipmaps.is_some() {
            ext.mip_levels
                .unwrap_or(std::cmp::max(width, height).ilog2())
        } else {
            1
        };

        let vkformat = match format {
            // TextureFormat::Rgb => vk::Format::R8G8B8_UNORM,
            // TODO FIX: This should be R8G8B8A8_SRGB, shouldn't it?
            TextureFormat::Rgba => vk::Format::R8G8B8A8_UNORM,
        };

        let vkusages = vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::SAMPLED;
        let aspect = vk::ImageAspectFlags::COLOR;

        let image = VkImage::new(
            &context.core.dev,
            &context.drop_queue,
            &mut context.alloc,
            vkformat,
            vkusages,
            aspect,
            vk::SampleCountFlags::TYPE_1,
            mip_levels,
            vk::Extent3D {
                width: width as u32,
                height: height as u32,
                depth: 1,
            },
        )?;

        let image_view =
            new_image_view(&context.core.dev, image.image, vkformat, aspect, mip_levels)?;

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
            mip_levels,

            image,
            staging,
            image_view,
            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
    }

    fn upload(&mut self, core: &VkCore, data: &[u8], ext: UploadTextureExt) -> GResult<()> {
        self.staging
            .map_copy_data(data.as_ptr() as *const u8, data.len(), 0)?;

        let _misc_command = core.misc_command();

        let range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
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
            .aspect_mask(vk::ImageAspectFlags::COLOR)
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

        let mut image_use_barrier = image_transfer_barrier;
        image_use_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_use_barrier.new_layout = if ext.generate_mipmaps.is_some() {
            vk::ImageLayout::TRANSFER_DST_OPTIMAL
        } else {
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        };
        image_use_barrier.src_access_mask = if ext.generate_mipmaps.is_some() {
            vk::AccessFlags::empty()
        } else {
            vk::AccessFlags::TRANSFER_WRITE
        };
        image_use_barrier.dst_access_mask = if ext.generate_mipmaps.is_some() {
            vk::AccessFlags::TRANSFER_WRITE
        } else {
            vk::AccessFlags::SHADER_READ
        };

        unsafe {
            core.dev.cmd_pipeline_barrier(
                core.misc_command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                if ext.generate_mipmaps.is_some() {
                    vk::PipelineStageFlags::TRANSFER
                } else {
                    vk::PipelineStageFlags::FRAGMENT_SHADER
                },
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_use_barrier],
            );
        }
        Ok(())
    }
}

//  Derived from https://github.com/SaschaWillems/Vulkan/blob/master/examples/texturemipmapgen/texturemipmapgen.cpp
//  Thank you Sascha Willems! <3
unsafe fn generate_mipmaps(
    context: &VkContext,
    width: usize,
    height: usize,
    image: vk::Image,
    mip_levels: u32,
) -> GResult<()> {
    let _misc = context.core.misc_command()?;

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .layer_count(1)
        .build();

    let image_transition_barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .subresource_range(subresource_range)
        .build();

    context.core.dev.cmd_pipeline_barrier(
        context.core.misc_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[image_transition_barrier],
    );

    for i in 1..mip_levels {
        let blit = vk::ImageBlit::builder()
            .src_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .mip_level(i - 1)
                    .build(),
            )
            .dst_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .mip_level(i)
                    .build(),
            )
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D {
                    x: (width as u32 >> (i - 1)) as i32,
                    y: (height as u32 >> (i - 1)) as i32,
                    z: 1,
                },
            ])
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D {
                    x: (width as u32 >> i) as i32,
                    y: (height as u32 >> i) as i32,
                    z: 1,
                },
            ])
            .build();

        let mip_subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(i)
            .level_count(1)
            .layer_count(1)
            .build();

        let image_transition_barrier = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .subresource_range(mip_subresource_range)
            .build();

        context.core.dev.cmd_pipeline_barrier(
            context.core.misc_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_transition_barrier],
        );

        context.core.dev.cmd_blit_image(
            context.core.misc_command_buffer,
            image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[blit],
            vk::Filter::LINEAR,
        );

        let image_transition_barrier = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .subresource_range(mip_subresource_range)
            .build();

        context.core.dev.cmd_pipeline_barrier(
            context.core.misc_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_transition_barrier],
        );
    }
    let range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1)
        .build();

    let image_transition_barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .subresource_range(range)
        .build();

    context.core.dev.cmd_pipeline_barrier(
        context.core.misc_command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[image_transition_barrier],
    );

    Ok(())
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
