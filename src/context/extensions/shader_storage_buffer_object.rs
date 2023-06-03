use super::*;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ShaderStorageBufferId(usize);
crate::def_id_ty!(ShaderStorageBufferId);

#[derive(Default, Debug, Clone)]
pub struct NewShaderStorageBufferExt {}
#[derive(Default, Debug, Clone)]
pub struct ReadSyncedShaderStorageBufferExt {}

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

    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferTypeGuard<T>,
        ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        unsafe { self.read_synced_shader_storage_buffer_unchecked(ssbo.0, ext) }
    }

    /// # Safety
    ///
    ///  The type `T` is not validated.
    ///  For validation, use [`Context::read_synced_shader_storage_buffer`].
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
