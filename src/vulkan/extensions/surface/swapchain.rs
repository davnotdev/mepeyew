use super::*;

pub struct VkSwapchain {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub swapchain_ext: vk_extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkSwapchain {
    pub fn new(
        core: &VkCore,
        surface: &VkSurface,
        w: u32,
        h: u32,
        drop_queue_ref: &VkDropQueueRef,
    ) -> GResult<Self> {
        let (Ok(surface_capablitites), Ok(surface_formats), Ok(surface_present_modes)) = (unsafe {(
                surface.surface_ext.get_physical_device_surface_capabilities(core.physical_dev, surface.surface),
                surface.surface_ext.get_physical_device_surface_formats(core.physical_dev, surface.surface),
                surface.surface_ext.get_physical_device_surface_present_modes(core.physical_dev, surface.surface),
                )}) else {
                Err(gpu_api_err!("vulkan gpu query"))?
            };
        let extent = surface_capablitites.current_extent;
        let extent = if extent.width >= surface_capablitites.min_image_extent.width
            && extent.width <= surface_capablitites.max_image_extent.width
            && extent.height >= surface_capablitites.min_image_extent.height
            && extent.height <= surface_capablitites.max_image_extent.height
        {
            extent
        } else {
            vk::Extent2D {
                width: w.clamp(
                    surface_capablitites.min_image_extent.width,
                    surface_capablitites.max_image_extent.width,
                ),
                height: h.clamp(
                    surface_capablitites.min_image_extent.height,
                    surface_capablitites.max_image_extent.height,
                ),
            }
        };

        //  Choose Swapchain Format
        let format = surface_formats.into_iter().next().unwrap();

        //  Choose Swapchain Present
        let present = [
            vk::PresentModeKHR::FIFO_RELAXED,
            vk::PresentModeKHR::FIFO,
            vk::PresentModeKHR::MAILBOX,
            vk::PresentModeKHR::IMMEDIATE,
        ]
        .into_iter()
        .find(|mode| surface_present_modes.iter().any(|p| p == mode))
        .unwrap();

        //  Create the Swapchain
        let swapchain_ext = vk_extensions::khr::Swapchain::new(&core.instance, &core.dev);
        let swapchain_create = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .image_extent(extent)
            .present_mode(present)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .min_image_count(surface_capablitites.min_image_count)
            .pre_transform(surface_capablitites.current_transform)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .build();
        let swapchain = unsafe { swapchain_ext.create_swapchain(&swapchain_create, None) }
            .map_err(|e| gpu_api_err!("vulkan swapchain init {}", e))?;
        let swapchain_images = unsafe { swapchain_ext.get_swapchain_images(swapchain) }
            .map_err(|e| gpu_api_err!("vulkan swapchain images obtain {}", e))?;

        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&img| {
                new_image_view(
                    &core.dev,
                    img,
                    format.format,
                    vk::ImageAspectFlags::COLOR,
                    1,
                    false,
                )
            })
            .collect::<GResult<_>>()?;

        //  Transition swapchain images from UNDEFINED to PRESENT_SRC_KHR
        {
            let _misc = core.misc_command();
            for &image in swapchain_images.iter() {
                let range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .layer_count(1)
                    .build();
                let src_image_transfer_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .image(image)
                    .subresource_range(range)
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .build();

                unsafe {
                    core.dev.cmd_pipeline_barrier(
                        core.misc_command_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[src_image_transfer_barrier],
                    );
                }
            }
        }

        Ok(VkSwapchain {
            format: format.format,
            extent,
            swapchain_ext,
            swapchain,
            swapchain_images,
            swapchain_image_views,

            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        let swapchain_image_views = self.swapchain_image_views.clone();
        let swapchain_ext = self.swapchain_ext.clone();
        let swapchain = self.swapchain;

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                swapchain_image_views
                    .iter()
                    .for_each(|view| dev.destroy_image_view(*view, None));
                swapchain_ext.destroy_swapchain(swapchain, None);
            }))
    }
}
