use super::*;

pub struct VkSwapchain {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub swapchain_ext: vk_extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
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
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .clipped(true)
            .build();
        let swapchain = unsafe { swapchain_ext.create_swapchain(&swapchain_create, None) }
            .map_err(|e| gpu_api_err!("vulkan swapchain init {}", e))?;
        let swapchain_images = unsafe { swapchain_ext.get_swapchain_images(swapchain) }
            .map_err(|e| gpu_api_err!("vulkan swapchain images obtain {}", e))?;

        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .into_iter()
            .map(|img| {
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

        Ok(VkSwapchain {
            format: format.format,
            extent,
            swapchain_ext,
            swapchain,
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
