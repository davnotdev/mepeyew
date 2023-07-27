use super::*;
use context::extensions::*;

impl VkContext {
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

    pub async fn async_read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferId,
        ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        self.read_synced_shader_storage_buffer(ssbo, ext)
    }
}
