use super::*;
use std::marker::PhantomData;

///  Defines the stride (see [`VertexInputArgCount`]) of each vbo item.
///  ```
///  [A, A, A, B, B, C, C, C]
///          ^3   ^2       ^3
///  let _ = VertexBufferInput {
///      args: vec![3, 2, 3],
///  };
///  ```
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VertexBufferInput {
    pub args: Vec<usize>,
}

///  Whether you plan on dynamically upload to a buffer later on.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BufferStorageType {
    Static,
    Dynamic,
}

///  The expected type of all vertex buffers.
pub type VertexBufferElement = f32;
///  The expected type of all index buffers.
pub type IndexBufferElement = u32;

#[derive(Default, Debug, Clone)]
pub struct NewVertexBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct NewIndexBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct NewUniformBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct NewDynamicUniformBufferExt {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct UniformBufferTypeGuard<T: Copy>(pub UniformBufferId, PhantomData<T>);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DynamicUniformBufferTypeGuard<T: Copy>(pub DynamicUniformBufferId, PhantomData<T>);

impl Context {
    pub fn new_vertex_buffer(
        &mut self,
        data: &[VertexBufferElement],
        storage_type: BufferStorageType,
        ext: Option<NewVertexBufferExt>,
    ) -> GResult<VertexBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_vertex_buffer(data, storage_type, ext),
            Self::WebGpu(wgpu) => wgpu.new_vertex_buffer(data, storage_type, ext),
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
            Self::WebGpu(wgpu) => wgpu.new_index_buffer(data, storage_type, ext),
        }
    }

    pub fn new_uniform_buffer<T: Copy>(
        &mut self,
        data: &T,
        ext: Option<NewUniformBufferExt>,
    ) -> GResult<(UniformBufferId, UniformBufferTypeGuard<T>)> {
        let id = match self {
            Self::Vulkan(vk) => vk.new_uniform_buffer::<T>(data, ext),
            Self::WebGpu(wgpu) => wgpu.new_uniform_buffer::<T>(data, ext),
        }?;

        Ok((id, UniformBufferTypeGuard(id, PhantomData)))
    }

    //  TODO docs
    pub fn new_dynamic_uniform_buffer<T: Copy>(
        &mut self,
        data: &[T],
        ext: Option<NewDynamicUniformBufferExt>,
    ) -> GResult<(DynamicUniformBufferId, DynamicUniformBufferTypeGuard<T>)> {
        let id = match self {
            Self::Vulkan(vk) => vk.new_dynamic_uniform_buffer(data, ext),
            Self::WebGpu(wgpu) => wgpu.new_dynamic_uniform_buffer(data, ext),
        }?;

        Ok((id, DynamicUniformBufferTypeGuard(id, PhantomData)))
    }
}
