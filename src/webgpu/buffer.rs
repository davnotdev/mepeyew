use super::*;

impl WebGpuContext {
    pub fn new_vertex_buffer(
        &mut self,
        data: &[VertexBufferElement],
        storage_type: BufferStorageType,
        _ext: Option<NewVertexBufferExt>,
    ) -> GResult<VertexBufferId> {
        let size = data.len() * std::mem::size_of::<VertexBufferElement>();
        let buffer = WebGpuBuffer::new(
            &self.device,
            size as u32,
            GpuBufferUsageFlags::Vertex as u32
                | match storage_type {
                    BufferStorageType::Dynamic => GpuBufferUsageFlags::CopyDst as u32,
                    _ => 0,
                },
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, size) },
        );
        self.vbos.push(buffer);
        Ok(VertexBufferId::from_id(self.vbos.len() - 1))
    }

    pub fn new_index_buffer(
        &mut self,
        data: &[IndexBufferElement],
        storage_type: BufferStorageType,
        _ext: Option<NewIndexBufferExt>,
    ) -> GResult<IndexBufferId> {
        let size = data.len() * std::mem::size_of::<IndexBufferElement>();
        let buffer = WebGpuBuffer::new(
            &self.device,
            size as u32,
            GpuBufferUsageFlags::Index as u32
                | match storage_type {
                    BufferStorageType::Dynamic => GpuBufferUsageFlags::CopyDst as u32,
                    _ => 0,
                },
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, size) },
        );
        self.ibos.push(buffer);
        Ok(IndexBufferId::from_id(self.ibos.len() - 1))
    }

    pub fn new_uniform_buffer<T: Copy>(
        &mut self,
        data: &T,
        _ext: Option<NewUniformBufferExt>,
    ) -> GResult<UniformBufferId> {
        let size = std::mem::size_of::<T>();
        let buffer = WebGpuBuffer::new(
            &self.device,
            size as u32,
            GpuBufferUsageFlags::Uniform as u32 | GpuBufferUsageFlags::CopyDst as u32,
            unsafe { std::slice::from_raw_parts(data as *const T as *const u8, size) },
        );
        self.ubos.push(buffer);
        Ok(UniformBufferId::from_id(self.ubos.len() - 1))
    }
}

pub struct WebGpuBuffer {
    pub buffer: GpuBuffer,
}

impl WebGpuBuffer {
    pub fn new(device: &GpuDevice, size: u32, usage: u32, data: &[u8]) -> Self {
        let mut buffer_info = GpuBufferDescriptor::new(size as f64, usage);
        buffer_info.mapped_at_creation(true);

        let buffer = device.create_buffer(&buffer_info);
        let mapped_range = Uint8Array::new(&buffer.get_mapped_range().into());

        for (idx, &val) in data.iter().enumerate() {
            mapped_range.set_index(idx as u32, val);
        }

        buffer.unmap();

        WebGpuBuffer { buffer }
    }
}
