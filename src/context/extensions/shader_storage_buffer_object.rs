use super::*;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ShaderStorageBufferId(usize);
crate::def_id_ty!(ShaderStorageBufferId);

#[derive(Default, Debug, Clone)]
pub struct NewShaderStorageBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct ReadSyncedShaderStorageBufferExt {}

/// Use to ensure that the correct type is used later when accessing data.
/// This guard is merely a design decision and serves no other purpose.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ShaderStorageBufferTypeGuard<T>(pub ShaderStorageBufferId, PhantomData<T>);

impl Context {
    pub fn new_shader_storage_buffer<T: Copy>(
        &mut self,
        data: &T,
        ext: Option<NewShaderStorageBufferExt>,
    ) -> GResult<(ShaderStorageBufferId, ShaderStorageBufferTypeGuard<T>)> {
        let id = match self {
            Self::Vulkan(vk) => vk.new_shader_storage_buffer(data, ext),
            Self::WebGpu(wgpu) => wgpu.new_shader_storage_buffer(data, ext),
        }?;

        Ok((id, ShaderStorageBufferTypeGuard(id, PhantomData)))
    }

    /// Read from a synced shader storage buffer object after rendering.
    /// Sync a shader storage buffer using `Submit::sync_shader_storage_buffer`.
    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferTypeGuard<T>,
        ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        unsafe { self.read_synced_shader_storage_buffer_unchecked(ssbo.0, ext) }
    }

    /// Read from a synced shader storage buffer object after rendering.
    /// Sync a shader storage buffer using `Submit::sync_shader_storage_buffer`.
    ///
    /// # Safety
    ///
    /// The type `T` is not validated.
    /// For validation, use [`Context::read_synced_shader_storage_buffer`].
    pub unsafe fn read_synced_shader_storage_buffer_unchecked<T: Copy>(
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
