use super::*;

///  Stride represents the following.
///  ```
///  [1, 1, 1, 2, 2, 3, 3, 3]
///   ^0       ^3    ^5
///  ```
#[derive(Debug, Clone, Copy)]
pub struct VertexInputArgStride(pub usize);

///  Defines the stride (see [`VertexInputArgStride`]) of each vbo item.
///  ```
///  [1, 1, 1, 2, 2, 3, 3, 3]
///   ^0       ^3    ^5
///  let _ = VertexBufferInput {
///      args: vec![0, 3, 5],
///  };
///  ```
#[derive(Debug, Clone)]
pub struct VertexBufferInput {
    pub args: Vec<VertexInputArgStride>,
}

///  Whether you plan on dynamically upload to a buffer later on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BufferStorageType {
    Static,
    Dynamic,
}

///  The expected type of all vertex buffers.
pub type VertexBufferElement = f32;
///  The expected type of all index buffers.
pub type IndexBufferElement = u32;

#[derive(Default)]
pub struct NewVertexBufferExt {}
#[derive(Default)]
pub struct NewIndexBufferExt {}
#[derive(Default)]
pub struct NewUniformBufferExt {}

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

    pub fn new_uniform_buffer<T: Copy>(
        &mut self,
        data: &T,
        ext: Option<NewUniformBufferExt>,
    ) -> GResult<UniformBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_uniform_buffer::<T>(data, ext),
        }
    }
}
