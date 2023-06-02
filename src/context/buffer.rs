use super::*;

///  Count represents the following.
///  ```
///  [A, A, A, B, B, C, C, C]
///          ^3   ^2       ^3
///  ```
#[derive(Debug, Clone, Copy)]
pub struct VertexInputArgCount(pub usize);

///  Defines the stride (see [`VertexInputArgCount`]) of each vbo item.
///  ```
///  [A, A, A, B, B, C, C, C]
///          ^3   ^2       ^3
///  let _ = VertexBufferInput {
///      args: vec![3, 2, 3],
///  };
///  ```
#[derive(Debug, Clone)]
pub struct VertexBufferInput {
    pub args: Vec<VertexInputArgCount>,
}

///  Whether you plan on dynamically upload to a buffer later on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Default, Debug, Clone)]
pub struct NewShaderStorageBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct ReadSyncedShaderStorageBufferExt {}

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
    ) -> GResult<UniformBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_uniform_buffer::<T>(data, ext),
            Self::WebGpu(wgpu) => wgpu.new_uniform_buffer::<T>(data, ext),
        }
    }

    //  TODO docs
    pub fn new_dynamic_uniform_buffer<T: Copy>(
        &mut self,
        data: &[T],
        ext: Option<NewDynamicUniformBufferExt>,
    ) -> GResult<DynamicUniformBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_dynamic_uniform_buffer(data, ext),
            Self::WebGpu(wgpu) => wgpu.new_dynamic_uniform_buffer(data, ext),
        }
    }

    pub fn new_shader_storage_buffer<T: Copy>(
        &mut self,
        data: &T,
        ext: Option<NewShaderStorageBufferExt>,
    ) -> GResult<ShaderStorageBufferId> {
        match self {
            Self::Vulkan(vk) => vk.new_shader_storage_buffer(data, ext),
            Self::WebGpu(wgpu) => wgpu.new_shader_storage_buffer(data, ext),
        }
    }

    //  Fix these docs.
    /// Read from a shader storage buffer.
    /// Ensure that [`PassStep::sync_shader_storage_buffer`] was called
    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferId,
        ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        match self {
            Self::Vulkan(vk) => vk.read_synced_shader_storage_buffer(ssbo, ext),
            Self::WebGpu(wgpu) => wgpu.read_synced_shader_storage_buffer(ssbo, ext),
        }
    }
}
