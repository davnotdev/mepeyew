use super::*;
use std::mem::ManuallyDrop;

pub const VK_COLOR_ATTACHMENT_FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;
pub const VK_DEPTH_ATTACHMENT_FORMAT: vk::Format = vk::Format::D32_SFLOAT_S8_UINT;

pub struct VkImage {
    pub image: vk::Image,
    pub format: vk::Format,
    pub view_aspect: vk::ImageAspectFlags,
    pub allocation: ManuallyDrop<Allocation>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkImage {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        view_aspect: vk::ImageAspectFlags,
        samples: vk::SampleCountFlags,
        mip_levels: u32,
        is_cubemap: bool,
        extent: vk::Extent3D,
    ) -> GResult<Self> {
        let image_create = vk::ImageCreateInfo::builder()
            .format(format)
            .usage(usage)
            .extent(extent)
            .image_type(vk::ImageType::TYPE_2D)
            .mip_levels(mip_levels)
            .array_layers(if is_cubemap { 6 } else { 1 })
            .flags(if is_cubemap {
                vk::ImageCreateFlags::CUBE_COMPATIBLE
            } else {
                vk::ImageCreateFlags::empty()
            })
            .samples(samples)
            .tiling(vk::ImageTiling::OPTIMAL)
            .build();

        let image = unsafe { dev.create_image(&image_create, None) }.unwrap();
        let requirements = unsafe { dev.get_image_memory_requirements(image) };

        let allocation = alloc
            .allocate(&AllocationCreateDesc {
                //  TODO EXT: Allow the naming of buffers.
                name: "Vulkan Generic Buffer",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged
            })
            .unwrap();

        unsafe {
            dev.bind_image_memory(image, allocation.memory(), allocation.offset())
                .map_err(|e| gpu_api_err!("vulkan image bind {}", e))?;
        };

        Ok(VkImage {
            image,
            format,
            allocation: ManuallyDrop::new(allocation),
            view_aspect,

            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkImage {
    fn drop(&mut self) {
        let image = self.image;
        let allocation = unsafe { ManuallyDrop::take(&mut self.allocation) };
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, alloc| unsafe {
                alloc.free(allocation).unwrap();
                dev.destroy_image(image, None);
            }));
    }
}

pub fn new_image_view(
    dev: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
    mip_level: u32,
    is_cubemap: bool,
) -> GResult<vk::ImageView> {
    let image_view_create = vk::ImageViewCreateInfo::builder()
        .image(image)
        .format(format)
        .components(
            vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY)
                .build(),
        )
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect)
                .base_mip_level(0)
                .level_count(mip_level)
                .base_array_layer(0)
                .layer_count(if is_cubemap { 6 } else { 1 })
                .build(),
        )
        .view_type(if is_cubemap {
            vk::ImageViewType::CUBE
        } else {
            vk::ImageViewType::TYPE_2D
        })
        .build();
    unsafe { dev.create_image_view(&image_view_create, None) }
        .map_err(|e| gpu_api_err!("vulkan image view init {}", e))
}
