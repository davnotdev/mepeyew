use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BufferStorageType {
    Static,
    Dynamic,
}

pub type VertexBufferElement = f32;
pub type IndexBufferElement = u32;

#[derive(Default)]
pub struct NewVertexBufferExt {}
#[derive(Default)]
pub struct NewIndexBufferExt {}

impl Context {
    pub fn new_vertex_buffer(
        &mut self,
        data: &[VertexBufferElement],
        storage_type: BufferStorageType,
        ext: Option<NewVertexBufferExt>,
    ) -> GResult<VertexBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_vertex_buffer(data, storage_type, ext),
        }
    }

    pub fn new_index_buffer(
        &mut self,
        data: &[IndexBufferElement],
        storage_type: BufferStorageType,
        ext: Option<NewIndexBufferExt>,
    ) -> GResult<IndexBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_index_buffer(data, storage_type, ext),
        }
    }
}
