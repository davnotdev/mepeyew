use super::*;

impl WebGpuContext {
    pub fn new_vertex_buffer(
        &mut self,
        _data: &[VertexBufferElement],
        _storage_type: BufferStorageType,
        _ext: Option<NewVertexBufferExt>,
    ) -> GResult<VertexBufferId> {
        todo!()
    }

    pub fn new_index_buffer(
        &mut self,
        _data: &[IndexBufferElement],
        _storage_type: BufferStorageType,
        _ext: Option<NewIndexBufferExt>,
    ) -> GResult<IndexBufferId> {
        todo!()
    }

    pub fn new_uniform_buffer<T: Copy>(
        &mut self,
        _data: &T,
        _ext: Option<NewUniformBufferExt>,
    ) -> GResult<UniformBufferId> {
        todo!()
    }
}
