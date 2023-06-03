use super::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ShaderStorageBufferId(usize);
crate::def_id_ty!(ShaderStorageBufferId);

#[derive(Default, Debug, Clone)]
pub struct NewShaderStorageBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct ReadSyncedShaderStorageBufferExt {}

impl Context {
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
