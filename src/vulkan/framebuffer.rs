use super::*;

pub enum VkFramebuffer {
    Single(VkSingleFramebuffer),
    Swapchain(Vec<VkSingleFramebuffer>),
}

impl VkFramebuffer {
    pub fn new(
        context: &VkContext,
        render_pass: vk::RenderPass,
        images: &[AttachmentImageId],
        resolve_images: &[vk::ImageView],
        width: usize,
        height: usize,
        use_swapchain: bool,
    ) -> GResult<VkFramebuffer> {
        Ok(if use_swapchain {
            let framebuffers = context
                .surface_ext
                .as_ref()
                .ok_or(gpu_api_err!(
                    "vulkan tried to create surface framebuffer without surface extension"
                ))?
                .swapchain
                .swapchain_image_views
                .iter()
                .copied()
                .map(|image_view| {
                    VkSingleFramebuffer::new(
                        context,
                        render_pass,
                        images,
                        resolve_images,
                        width,
                        height,
                        Some(image_view),
                    )
                })
                .collect::<GResult<Vec<_>>>()?;
            VkFramebuffer::Swapchain(framebuffers)
        } else {
            VkFramebuffer::Single(VkSingleFramebuffer::new(
                context,
                render_pass,
                images,
                resolve_images,
                width,
                height,
                None,
            )?)
        })
    }

    pub fn get_current_framebuffer(&self, idx: u32) -> vk::Framebuffer {
        match self {
            Self::Single(fb) => fb.framebuffer,
            Self::Swapchain(fbs) => fbs[idx as usize].framebuffer,
        }
    }
}

pub struct VkSingleFramebuffer {
    pub framebuffer: vk::Framebuffer,

    drop_queue_ref: VkDropQueueRef,
}

//  TODO FIX: Assert that image dims >= fb dims.
impl VkSingleFramebuffer {
    fn new(
        context: &VkContext,
        render_pass: vk::RenderPass,
        images: &[AttachmentImageId],
        resolve_images: &[vk::ImageView],
        width: usize,
        height: usize,
        swapchain_image_view: Option<vk::ImageView>,
    ) -> GResult<Self> {
        let mut image_views = vec![];
        if let Some(swapchain_image_view) = swapchain_image_view {
            image_views.push(swapchain_image_view);
        }

        images.iter().for_each(|attachment_image_id| {
            let attachment_image = context
                .attachment_images
                .get(attachment_image_id.id())
                .unwrap();
            image_views.push(attachment_image.image_view);
        });

        resolve_images
            .iter()
            .for_each(|&additional_attachment_view| {
                image_views.push(additional_attachment_view);
            });

        let framebuffer_create = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&image_views)
            .width(width as u32)
            .height(height as u32)
            .layers(1)
            .build();
        let framebuffer = unsafe {
            context
                .core
                .dev
                .create_framebuffer(&framebuffer_create, None)
        }
        .map_err(|e| gpu_api_err!("vulkan lone framebuffer init {}", e))?;

        Ok(VkSingleFramebuffer {
            framebuffer,
            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
    }
}

impl Drop for VkSingleFramebuffer {
    fn drop(&mut self) {
        let framebuffer = self.framebuffer;
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_framebuffer(framebuffer, None);
            }))
    }
}
