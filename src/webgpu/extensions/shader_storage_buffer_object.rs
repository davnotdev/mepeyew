use super::*;
use context::extensions::*;

impl WebGpuContext {
    pub fn new_shader_storage_buffer<T: Copy>(
        &mut self,
        data: &T,
        _ext: Option<NewShaderStorageBufferExt>,
    ) -> GResult<ShaderStorageBufferId> {
        let size = std::mem::size_of::<T>();
        let buffer = WebGpuBuffer::new(
            &self.device,
            size as u32,
            GpuBufferUsageFlags::Storage as u32
                | GpuBufferUsageFlags::CopyDst as u32
                | GpuBufferUsageFlags::CopySrc as u32,
            unsafe { std::slice::from_raw_parts(data as *const T as *const u8, size) },
            true,
        );
        self.ssbos.push(buffer);
        Ok(ShaderStorageBufferId::from_id(self.ssbos.len() - 1))
    }

    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        _ssbo: ShaderStorageBufferId,
        _ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        Err(gpu_api_err!(
            "webgpu does not support this operation, please use async_read_synced_shader_storage_buffer instead"
        ))
    }

    pub async fn async_read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferId,
        _ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        let ssbo = self.ssbos.get(ssbo.id()).ok_or(gpu_api_err!(
            "webgpu read synced shader buffer id {:?} does not exist",
            ssbo
        ))?;

        let readable_buffer = ssbo.readable_buffer.as_ref().unwrap();

        let command_encoder = self.device.create_command_encoder();
        command_encoder.copy_buffer_to_buffer_with_u32_and_u32_and_u32(
            &ssbo.buffer,
            0,
            readable_buffer,
            0,
            ssbo.size,
        );
        let command_buffer = command_encoder.finish();
        let commands = Array::new();
        commands.push(&command_buffer);
        self.device.queue().submit(&commands);

        JsFuture::from(readable_buffer.map_async(GpuMapModeFlags::Read as u32))
            .await
            .map_err(|e| {
                gpu_api_err!(
                    "webgpu failed to map buffer in async_read_synced_shader_storage_buffer: {:?}",
                    e
                )
            })?;

        let mapped_buf = readable_buffer.get_mapped_range();
        let u8_js_buf = Uint8Array::new(&mapped_buf);
        let mut u8_rs_buf = vec![0u8; u8_js_buf.length() as usize];
        for i in 0..u8_js_buf.length() {
            u8_rs_buf[i as usize] = u8_js_buf.get_index(i);
        }

        ssbo.buffer.unmap();

        Ok(unsafe { std::ptr::read(u8_rs_buf.as_ptr() as *const T) })
    }
}
