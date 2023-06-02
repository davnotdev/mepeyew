use super::{debug::VkDebug, *};
use std::ffi::CString;

pub struct VkCore {
    pub instance: Instance,

    pub physical_dev: vk::PhysicalDevice,
    pub dev: Device,

    pub physical_dev_properties: vk::PhysicalDeviceProperties,

    pub graphics_queue: vk::Queue,
    pub compute_queue: vk::Queue,

    pub graphics_command_pool: vk::CommandPool,
    pub compute_command_pool: vk::CommandPool,

    pub misc_fence: vk::Fence,
    pub misc_command_buffer: vk::CommandBuffer,

    pub debug: Option<VkDebug>,
    pub entry: Entry,
}

pub enum VkCoreGpuPreference {
    Discrete,
    Integrated,
}

pub struct VkCoreConfiguration {
    pub gpu_preference: VkCoreGpuPreference,
    pub use_debug: bool,
    pub use_surface: bool,
}

impl VkCore {
    pub fn new(config: VkCoreConfiguration) -> GResult<Self> {
        let Ok(entry) = (unsafe {Entry::load()}) else {
            Err(gpu_api_err!("vulkan not found"))?
        };

        //  # Extensions and Layers
        let mut instance_extensions_owned =
            vec![CString::new("VK_KHR_portability_enumeration").unwrap()];
        let mut layers_owned = vec![];

        //  ## Debug
        if config.use_debug {
            VkDebug::get_additional_extensions()
                .into_iter()
                .for_each(|ext| {
                    instance_extensions_owned.push(CString::new(ext.to_str().unwrap()).unwrap())
                });
            VkDebug::get_additional_layers()
                .iter()
                .for_each(|&layer| layers_owned.push(CString::new(layer).unwrap()));
        }

        //  ## Surface Extension
        if config.use_surface {
            extensions::VkSurfaceExt::get_additional_instance_extensions()
                .iter()
                .for_each(|ext| {
                    instance_extensions_owned.push(CString::new(ext.to_string()).unwrap())
                });
        }

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
            .flags(vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR)
            .push_next(&mut VkDebug::get_debug_create())
            .build();
        let Ok(instance) = (unsafe { entry.create_instance(&instance_create, None)}) else {
                Err(gpu_api_err!("vulkan not supported"))?
            };

        //  # Make Debug
        let debug = if config.use_debug {
            Some(VkDebug::new(&entry, &instance)?)
        } else {
            None
        };

        //  # Get Physical Device and Queue Families
        let mut physical_devs = unsafe { instance.enumerate_physical_devices() }
            .map_err(|e| gpu_api_err!("vulkan gpu(s) not found {}", e))?
            .into_iter()
            .filter_map(|physical_dev| {
                QueueFamilies::new(&instance, &physical_dev)
                    .map(|queue_families| (physical_dev, queue_families))
            })
            .map(|(physical_dev, queue_families)| {
                (
                    physical_dev,
                    unsafe { instance.get_physical_device_properties(physical_dev) },
                    unsafe { instance.get_physical_device_memory_properties(physical_dev) },
                    queue_families,
                )
            })
            .collect::<Vec<_>>();

        fn score_gpu(
            props: &vk::PhysicalDeviceProperties,
            mem_props: &vk::PhysicalDeviceMemoryProperties,
        ) -> usize {
            //  Discrete or Integrated or neither?
            let score = match props.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 9999999999999,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 99999999,
                _ => 0usize,
            };

            //  Who has the most memory?
            let score = mem_props.memory_heaps.iter().fold(score, |acc, heap| {
                if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                    acc + heap.size as usize
                } else {
                    acc
                }
            });

            score
        }

        physical_devs.sort_by(
            |(_, a_props, a_mem_props, _), (_, b_props, b_mem_props, _)| {
                //  Maybe I don't understand `sort_by`, but shouldn't this be `a.cmp(b)`?
                // score_gpu(a_props, a_mem_props).cmp(&score_gpu(b_props, b_mem_props))
                score_gpu(b_props, b_mem_props).cmp(&score_gpu(a_props, a_mem_props))
            },
        );

        fn try_find_with_type(
            physical_devs: &[(
                vk::PhysicalDevice,
                vk::PhysicalDeviceProperties,
                vk::PhysicalDeviceMemoryProperties,
                QueueFamilies,
            )],
            ty: vk::PhysicalDeviceType,
        ) -> Option<(vk::PhysicalDevice, QueueFamilies)> {
            physical_devs
                .iter()
                .find_map(|&(physical_dev, props, _, queue_families)| {
                    (props.device_type == ty).then_some((physical_dev, queue_families))
                })
        }

        let mut physical_dev = None;

        match config.gpu_preference {
            VkCoreGpuPreference::Discrete => {
                if physical_dev.is_none() {
                    physical_dev =
                        try_find_with_type(&physical_devs, vk::PhysicalDeviceType::DISCRETE_GPU);
                }
            }
            VkCoreGpuPreference::Integrated => {}
        };

        [
            vk::PhysicalDeviceType::INTEGRATED_GPU,
            vk::PhysicalDeviceType::VIRTUAL_GPU,
            vk::PhysicalDeviceType::CPU,
            vk::PhysicalDeviceType::OTHER,
        ]
        .into_iter()
        .for_each(|ty| {
            if physical_dev.is_none() {
                physical_dev = try_find_with_type(&physical_devs, ty);
            }
        });

        let Some((physical_dev, queue_families)) = physical_dev else {
            Err(gpu_api_err!("vulkan gpu(s) not suitable"))?
        };

        let physical_dev_properties =
            unsafe { instance.get_physical_device_properties(physical_dev) };

        //  # Make Device
        let features = vk::PhysicalDeviceFeatures::default();
        let dev_storage_buffer_ext = CString::new("VK_KHR_storage_buffer_storage_class").unwrap();
        let mut dev_extensions_owned = vec![dev_storage_buffer_ext.as_c_str()];
        if config.use_surface {
            dev_extensions_owned.push(extensions::VkSurfaceExt::get_additional_device_extension());
        }
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

        //  # Create Single Use Fence and Command Buffer
        let misc_fence = new_fence(&dev, false)?;
        let misc_command_buffer_alloc = vk::CommandBufferAllocateInfo::builder()
            .command_pool(compute_command_pool)
            .command_buffer_count(1)
            .build();
        let misc_command_buffer =
            unsafe { dev.allocate_command_buffers(&misc_command_buffer_alloc) }
                .map_err(|e| gpu_api_err!("vulkan alloc misc command buffer {}", e))?
                .into_iter()
                .next()
                .unwrap();

        Ok(VkCore {
            instance,
            physical_dev,
            physical_dev_properties,
            dev,
            debug,
            entry,
            graphics_queue,
            compute_queue,
            graphics_command_pool,
            compute_command_pool,
            misc_fence,
            misc_command_buffer,
        })
    }

    pub fn misc_command(&self) -> GResult<VkMiscCommand> {
        self.begin_misc_cmd()?;
        Ok(VkMiscCommand { vkcore: self })
    }

    fn begin_misc_cmd(&self) -> GResult<()> {
        let cmd_begin_create = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();
        unsafe {
            self.dev
                .begin_command_buffer(self.misc_command_buffer, &cmd_begin_create)
                .map_err(|e| gpu_api_err!("vulkan misc begin command buffer {}", e))?;
        }
        Ok(())
    }

    fn end_misc_cmd(&self) -> GResult<()> {
        unsafe {
            self.dev
                .end_command_buffer(self.misc_command_buffer)
                .map_err(|e| gpu_api_err!("vulkan misc end command buffer {}", e))?;

            let submit_create = vk::SubmitInfo::builder()
                .command_buffers(&[self.misc_command_buffer])
                .build();

            self.dev
                .queue_submit(self.compute_queue, &[submit_create], self.misc_fence)
                .map_err(|e| gpu_api_err!("vulkan misc submit {}", e))?;
            self.dev
                .wait_for_fences(&[self.misc_fence], true, std::u64::MAX)
                .map_err(|e| gpu_api_err!("vulkan misc wait fence {}", e))?;

            self.dev
                .reset_fences(&[self.misc_fence])
                .map_err(|e| gpu_api_err!("vulkan misc copy reset fence {}", e))?;
            self.dev
                .reset_command_buffer(
                    self.misc_command_buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .map_err(|e| gpu_api_err!("vulkan misc reset command buffer {}", e))?;
        }
        Ok(())
    }
}

impl Drop for VkCore {
    fn drop(&mut self) {
        unsafe {
            self.dev.destroy_fence(self.misc_fence, None);
            self.dev
                .destroy_command_pool(self.graphics_command_pool, None);
            self.dev
                .destroy_command_pool(self.compute_command_pool, None);

            self.dev.destroy_device(None);

            if let Some(debug) = &mut self.debug {
                debug.destory();
            }

            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Clone, Copy)]
struct QueueFamilies {
    pub graphics: u32,
    pub compute: u32,
}

impl QueueFamilies {
    pub fn new(instance: &Instance, physical_dev: &vk::PhysicalDevice) -> Option<Self> {
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

pub struct VkMiscCommand<'a> {
    vkcore: &'a VkCore,
}

impl<'a> Drop for VkMiscCommand<'a> {
    fn drop(&mut self) {
        self.vkcore.end_misc_cmd().unwrap();
    }
}
