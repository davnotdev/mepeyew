use super::*;
use std::mem::ManuallyDrop;

//  TODO OPT: Question on optimizing buffers. So right now, pretty much everything relies on
//  staging buffer. Great right? GpuOnly == Better Performance, but wait. What about uploads?
//  staging buffers require a map, then upload. For per frame uploads, shouldn't it just be better
//  to use the Gpu Cpu shared buffers? I'm not sure if the CpuToGpu option allows for this, so for
//  now, staging buffers are the norm.

impl VkContext {
    pub fn new_vertex_buffer(
        &mut self,
        data: &[VertexBufferElement],
        storage_type: BufferStorageType,
        _ext: Option<NewVertexBufferExt>,
    ) -> GResult<VertexBufferId> {
        let (buf, staging) =
            self.new_generic_buffer(data, storage_type, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        let vbo = VkVertexBuffer {
            buffer: buf,
            staging,
        };
        self.vbos.push(vbo);
        Ok(VertexBufferId::from_id(self.vbos.len() - 1))
    }

    pub fn new_index_buffer(
        &mut self,
        data: &[IndexBufferElement],
        storage_type: BufferStorageType,
        _ext: Option<NewIndexBufferExt>,
    ) -> GResult<IndexBufferId> {
        let (buf, staging) =
            self.new_generic_buffer(data, storage_type, vk::BufferUsageFlags::INDEX_BUFFER)?;
        let ibo = VkIndexBuffer {
            buffer: buf,
            staging,
        };
        self.ibos.push(ibo);
        Ok(IndexBufferId::from_id(self.ibos.len() - 1))
    }

    pub fn new_uniform_buffer<T: Copy>(
        &mut self,
        data: &T,
        _ext: Option<NewUniformBufferExt>,
    ) -> GResult<UniformBufferId> {
        let (buf, staging) = self.new_generic_buffer(
            std::slice::from_ref(data),
            BufferStorageType::Dynamic,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        )?;
        let ubo = VkUniformBuffer {
            buffer: buf,
            staging,
        };
        self.ubos.push(ubo);
        Ok(UniformBufferId::from_id(self.ubos.len() - 1))
    }

    pub fn new_dynamic_uniform_buffer<T: Copy>(
        &mut self,
        data: &[T],
        _ext: Option<NewDynamicUniformBufferExt>,
    ) -> GResult<DynamicUniformBufferId> {
        let item_size = std::mem::size_of::<T>();
        let padded_size = get_padded_size(self, item_size);
        let padded_buf = unsafe { pad_slice(data, padded_size) };

        let (buf, staging) = self.new_generic_buffer(
            &padded_buf,
            BufferStorageType::Dynamic,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        )?;
        let ubo = VkDynamicUniformBuffer {
            buffer: buf,
            staging,
            per_index_offset: padded_size,
            item_size,
        };
        self.dyn_ubos.push(ubo);
        Ok(DynamicUniformBufferId::from_id(self.dyn_ubos.len() - 1))
    }

    pub fn new_shader_storage_buffer<T: Copy>(
        &mut self,
        data: &T,
        _ext: Option<NewShaderStorageBufferExt>,
    ) -> GResult<ShaderStorageBufferId> {
        let (buf, staging) = self.new_generic_buffer(
            std::slice::from_ref(data),
            BufferStorageType::Dynamic,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        )?;
        let ssbo = VkShaderStorageBuffer {
            buffer: buf,
            staging,
        };
        self.ssbos.push(ssbo);
        Ok(ShaderStorageBufferId::from_id(self.ssbos.len() - 1))
    }

    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferId,
        _ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        let ssbo = self.ssbos.get(ssbo.id()).ok_or(gpu_api_err!(
            "vulkan read synced shader buffer id {:?} does not exist",
            ssbo
        ))?;
        Ok(unsafe {
            std::ptr::read(ssbo.staging.as_ref().unwrap().mapped_ptr.unwrap() as *const T)
        })
    }
}

fn get_padded_size(ctx: &VkContext, size: usize) -> usize {
    let min_ubo_alignment = ctx
        .core
        .physical_dev_properties
        .limits
        .min_uniform_buffer_offset_alignment as usize;
    if min_ubo_alignment > 0 {
        (size + min_ubo_alignment - 1) & !(min_ubo_alignment - 1)
    } else {
        size
    }
}

unsafe fn pad_slice<T: Copy>(slice: &[T], padded_size: usize) -> Vec<u8> {
    let size = std::mem::size_of::<T>();
    let byte_slice = unsafe {
        std::slice::from_raw_parts(slice.as_ptr() as *const T as *const u8, size * slice.len())
    };
    let mut out = vec![];
    for i in 0..slice.len() {
        for s in 0..size {
            out.push(byte_slice[i * size + s]);
        }
        out.resize(out.len() + (padded_size - size), 0);
    }
    out
}

pub struct VkVertexBuffer {
    pub buffer: VkBuffer,
    staging: Option<VkBuffer>,
}

pub struct VkIndexBuffer {
    pub buffer: VkBuffer,
    staging: Option<VkBuffer>,
}

pub struct VkUniformBuffer {
    pub buffer: VkBuffer,
    staging: Option<VkBuffer>,
}

pub struct VkDynamicUniformBuffer {
    pub buffer: VkBuffer,
    staging: Option<VkBuffer>,
    pub per_index_offset: usize,
    item_size: usize,
}

pub struct VkShaderStorageBuffer {
    pub buffer: VkBuffer,
    pub staging: Option<VkBuffer>,
}

impl VkVertexBuffer {
    pub fn cmd_transfer(
        &mut self,
        dev: &Device,
        cmd_buf: vk::CommandBuffer,
        data: &[VertexBufferElement],
    ) -> GResult<()> {
        cmd_transfer_generic(
            dev,
            cmd_buf,
            &self.buffer,
            self.staging.as_mut().ok_or(gpu_api_err!(
                "vulkan this vertex buffer does not support transfers"
            ))?,
            data,
            self.buffer.size,
            self.buffer.size,
            0,
        )
    }
}

impl VkIndexBuffer {
    pub fn cmd_transfer(
        &mut self,
        dev: &Device,
        cmd_buf: vk::CommandBuffer,
        data: &[IndexBufferElement],
    ) -> GResult<()> {
        cmd_transfer_generic(
            dev,
            cmd_buf,
            &self.buffer,
            self.staging.as_mut().ok_or(gpu_api_err!(
                "vulkan this index buffer does not support transfers"
            ))?,
            data,
            self.buffer.size,
            self.buffer.size,
            0,
        )
    }
}

impl VkUniformBuffer {
    pub fn cmd_transfer<T>(
        &mut self,
        dev: &Device,
        cmd_buf: vk::CommandBuffer,
        data: &[T],
    ) -> GResult<()> {
        cmd_transfer_generic(
            dev,
            cmd_buf,
            &self.buffer,
            self.staging.as_mut().ok_or(gpu_api_err!(
                "vulkan this uniform buffer does not support transfers"
            ))?,
            data,
            self.buffer.size,
            self.buffer.size,
            0,
        )
    }
}

impl VkDynamicUniformBuffer {
    pub fn cmd_transfer<T>(
        &mut self,
        dev: &Device,
        cmd_buf: vk::CommandBuffer,
        data: &[T],
        index: usize,
    ) -> GResult<()> {
        cmd_transfer_generic(
            dev,
            cmd_buf,
            &self.buffer,
            self.staging.as_mut().ok_or(gpu_api_err!(
                "vulkan this dynamic uniform buffer does not support transfers"
            ))?,
            data,
            self.per_index_offset,
            self.item_size,
            index * self.per_index_offset,
        )
    }
}

fn cmd_transfer_generic<T>(
    dev: &Device,
    cmd_buf: vk::CommandBuffer,
    buffer: &VkBuffer,
    staging: &mut VkBuffer,
    data: &[T],
    size: usize,
    item_size: usize,
    offset: usize,
) -> GResult<()> {
    staging.map_copy_data(data.as_ptr() as *const u8, item_size, offset)?;
    VkBuffer::cmd_upload_copy_data(staging, buffer, dev, size, offset, cmd_buf);
    Ok(())
}

impl VkContext {
    fn new_generic_buffer<T>(
        &mut self,
        data: &[T],
        storage_type: BufferStorageType,
        additional_buffer_usage: vk::BufferUsageFlags,
    ) -> GResult<(VkBuffer, Option<VkBuffer>)> {
        let buf_size = std::mem::size_of::<T>() * data.len();

        let mut staging = VkBuffer::new(
            &self.core.dev,
            &self.drop_queue,
            &mut self.alloc,
            buf_size,
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        )?;
        staging.map_copy_data(data.as_ptr() as *const u8, buf_size, 0)?;
        let buf = VkBuffer::new(
            &self.core.dev,
            &self.drop_queue,
            &mut self.alloc,
            buf_size,
            vk::BufferUsageFlags::TRANSFER_DST | additional_buffer_usage,
            MemoryLocation::GpuOnly,
        )?;
        {
            VkBuffer::single_upload_copy_data(&staging, &buf, &self.core, staging.size, 0)?;
        }

        Ok((
            buf,
            if storage_type == BufferStorageType::Dynamic {
                Some(staging)
            } else {
                None
            },
        ))
    }
}

pub struct VkBuffer {
    pub buffer: vk::Buffer,
    pub allocation: ManuallyDrop<Allocation>,
    pub size: usize,

    mapped_ptr: Option<*mut u8>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkBuffer {
    pub fn new(
        dev: &Device,
        drop_queue_ref: &VkDropQueueRef,
        alloc: &mut Allocator,
        data_size: usize,
        usage: vk::BufferUsageFlags,
        mem_usage: MemoryLocation,
    ) -> GResult<Self> {
        //  Create and Allocate the Buffer
        let buffer_create = vk::BufferCreateInfo::builder()
            .size(data_size as u64)
            .usage(usage)
            .build();

        let buffer = unsafe { dev.create_buffer(&buffer_create, None) }.unwrap();
        let requirements = unsafe { dev.get_buffer_memory_requirements(buffer) };

        let allocation = alloc
            .allocate(&AllocationCreateDesc {
                //  TODO EXT: Support the naming of buffers.
                name: "Vulkan Generic Buffer",
                requirements,
                location: mem_usage,
                linear: true,
            })
            .unwrap();

        unsafe {
            dev.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .map_err(|e| gpu_api_err!("vulkan buffer bind {}", e))?;
        };

        Ok(VkBuffer {
            buffer,
            allocation: ManuallyDrop::new(allocation),
            size: data_size,

            mapped_ptr: None,

            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }

    pub fn map_copy_data(&mut self, ptr: *const u8, size: usize, offset: usize) -> GResult<()> {
        if let Some(mapped_ptr) = self.mapped_ptr {
            unsafe {
                std::hint::black_box(std::ptr::copy_nonoverlapping::<u8>(
                    ptr,
                    mapped_ptr.add(offset),
                    size,
                ));
            }
        } else {
            let data = self
                .allocation
                .mapped_ptr()
                .ok_or(gpu_api_err!(
                    "vulkan gpu_allocator, this buffer cannot be mapped"
                ))?
                .as_ptr();
            self.mapped_ptr = Some(data as *mut u8);
            self.map_copy_data(ptr, size, offset)?;
        }
        Ok(())
    }

    fn cmd_upload_copy_data(
        src: &Self,
        dst: &Self,
        dev: &Device,
        size: usize,
        offset: usize,
        command_buf: vk::CommandBuffer,
    ) {
        let copy_create = vk::BufferCopy::builder()
            .src_offset(offset as u64)
            .dst_offset(offset as u64)
            .size(size as u64)
            .build();
        unsafe { dev.cmd_copy_buffer(command_buf, src.buffer, dst.buffer, &[copy_create]) };
    }

    pub fn single_upload_copy_data(
        src: &Self,
        dst: &Self,
        core: &VkCore,
        size: usize,
        offset: usize,
    ) -> GResult<()> {
        if src.size > dst.size {
            Err(gpu_api_err!(
                "vulkan upload copy src.size ({}) > dst.size ({})",
                src.size,
                dst.size
            ))?
        }
        let _misc_cmd = core.misc_command()?;
        Self::cmd_upload_copy_data(src, dst, &core.dev, size, offset, core.misc_command_buffer);
        Ok(())
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        let allocation = unsafe { ManuallyDrop::take(&mut self.allocation) };

        let buffer = self.buffer;
        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, alloc| unsafe {
                alloc.free(allocation).unwrap();
                dev.destroy_buffer(buffer, None);
            }));
    }
}
