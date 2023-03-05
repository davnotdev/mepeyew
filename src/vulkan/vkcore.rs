use super::{debug::VkDebug, surface::VkSurface, *};
use std::ffi::CString;

pub struct VkCore {
    pub instance: Instance,

    pub surface: VkSurface,
    pub physical_dev: vk::PhysicalDevice,
    pub dev: Device,

    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,

    pub graphics_command_pool: vk::CommandPool,
    pub compute_command_pool: vk::CommandPool,

    pub debug: VkDebug,
    pub entry: Entry,
}

impl VkCore {
    pub fn new(display: &RawDisplayHandle, window: &RawWindowHandle) -> GResult<Self> {
        let Ok(entry) = (unsafe {Entry::load()}) else {
            Err(gpu_api_err!("vulkan not found"))?
        };

        //  # Extensions and Layers
        let mut instance_extensions_owned = vec![];
        let mut layers_owned = vec![];

        //  ## Debug
        VkDebug::get_additional_extensions()
            .into_iter()
            .for_each(|ext| {
                instance_extensions_owned.push(CString::new(ext.to_str().unwrap()).unwrap())
            });
        VkDebug::get_additional_layers()
            .iter()
            .for_each(|&layer| layers_owned.push(CString::new(layer).unwrap()));

        //  ## Surface
        VkSurface::get_additional_extensions()
            .iter()
            .for_each(|ext| instance_extensions_owned.push(CString::new(ext.to_string()).unwrap()));

        //  ## Rust to C ffi friendly strings
        let instance_extensions = instance_extensions_owned
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<*const i8>>();
        let layers = layers_owned
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<*const i8>>();

        //  # Make Instance
        let instance_create = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(&instance_extensions)
            .enabled_layer_names(&layers)
            .push_next(&mut VkDebug::get_debug_create())
            .build();
        let Ok(instance) = (unsafe { entry.create_instance(&instance_create, None)}) else {
                Err(gpu_api_err!("vulkan not supported"))?
            };

        //  # Make Debug
        let debug = VkDebug::new(&entry, &instance)?;

        //  # Make Surface
        let surface = VkSurface::new(&entry, &instance, display, window)?;

        //  # Get Physical Device and Queue Families
        let physical_devs = unsafe { instance.enumerate_physical_devices() }
            .map_err(|e| gpu_api_err!("vulkan gpu(s) not found {}", e))?;

        //  TODO OPT: Chose the proper GPU.
        let Some((physical_dev, queue_families)) = physical_devs
            .iter()
            .find_map(|&physical_dev| {
                QueueFamilies::new(&instance, &physical_dev, &surface)
                    .map(|queue_family| (physical_dev, queue_family))
            }) else {
            Err(gpu_api_err!("vulkan gpu(s) not suitable"))?
        };

        //  # Make Device
        let features = vk::PhysicalDeviceFeatures::default();
        let dev_extensions_owned = vec![extensions::khr::Swapchain::name()];
        let dev_extensions: Vec<*const i8> =
            dev_extensions_owned.iter().map(|s| s.as_ptr()).collect();

        //  ## The Lazy Way of Creating Queues
        let mut queues_create = vec![vk::DeviceQueueCreateInfo::builder()
            .queue_priorities(&[1.0])
            .queue_family_index(queue_families.graphics)
            .build()];
        if queue_families.graphics != queue_families.compute {
            queues_create.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_priorities(&[1.0])
                    .queue_family_index(queue_families.compute)
                    .build(),
            )
        }

        //  ## Actually Make It
        let dev_create = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&dev_extensions)
            .enabled_layer_names(&layers)
            .enabled_features(&features)
            .queue_create_infos(&queues_create)
            .build();
        let Ok(dev) = (unsafe {
            instance.create_device(physical_dev, &dev_create, None)
        }) else {
            Err(gpu_api_err!("vulkan device init"))?
        };

        //  # Get Queues
        let graphics_queue = unsafe { dev.get_device_queue(queue_families.graphics, 0) };
        let compute_queue = unsafe { dev.get_device_queue(queue_families.compute, 0) };

        //  # Create Command Pools
        let graphics_command_pool_create = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_families.graphics)
            .build();

        let compute_command_pool_create = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_families.compute)
            .build();

        let graphics_command_pool =
            unsafe { dev.create_command_pool(&graphics_command_pool_create, None) }
                .map_err(|e| gpu_api_err!("vulkan graphics command pool init {}", e))?;
        let compute_command_pool =
            unsafe { dev.create_command_pool(&compute_command_pool_create, None) }
                .map_err(|e| gpu_api_err!("vulkan compute command pool init {}", e))?;

        Ok(VkCore {
            instance,
            surface,
            physical_dev,
            dev,
            debug,
            entry,
            graphics_queue,
            compute_queue,
            graphics_command_pool,
            compute_command_pool,
        })
    }
}

impl Drop for VkCore {
    fn drop(&mut self) {
        unsafe {
            self.dev
                .destroy_command_pool(self.graphics_command_pool, None);
            self.dev
                .destroy_command_pool(self.compute_command_pool, None);

            self.surface.destroy();
            self.dev.destroy_device(None);
            self.debug.destory();
            self.instance.destroy_instance(None);
        }
    }
}

struct QueueFamilies {
    pub graphics: u32,
    pub compute: u32,
}

impl QueueFamilies {
    pub fn new(
        instance: &Instance,
        physical_dev: &vk::PhysicalDevice,
        _surface: &VkSurface,
    ) -> Option<Self> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_dev) };

        fn find_beefiest_queue_family_for(
            queue_families: &[vk::QueueFamilyProperties],
            ty: vk::QueueFlags,
        ) -> u32 {
            let mut res = queue_families
                .iter()
                .filter(|prop| prop.queue_flags.contains(ty))
                .enumerate()
                .collect::<Vec<_>>();
            res.sort_by(|(_, a), (_, b)| a.queue_count.cmp(&b.queue_count));
            res[0].0 as u32
        }

        //  Find the best graphics queue family
        Some(QueueFamilies {
            graphics: find_beefiest_queue_family_for(&queue_families, vk::QueueFlags::GRAPHICS),
            compute: find_beefiest_queue_family_for(&queue_families, vk::QueueFlags::COMPUTE),
        })
    }
}

pub fn new_semaphore(dev: &Device) -> GResult<vk::Semaphore> {
    let semaphore_create = vk::SemaphoreCreateInfo::builder().build();
    unsafe { dev.create_semaphore(&semaphore_create, None) }
        .map_err(|e| gpu_api_err!("vulkan semaphore {}", e))
}

pub fn new_fence(dev: &Device, signaled: bool) -> GResult<vk::Fence> {
    let fence_create = vk::FenceCreateInfo::builder()
        .flags(if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        })
        .build();
    unsafe { dev.create_fence(&fence_create, None) }.map_err(|e| gpu_api_err!("vulkan fence {}", e))
}
