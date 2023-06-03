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
        );
        self.ssbos.push(buffer);
        Ok(ShaderStorageBufferId::from_id(self.ssbos.len() - 1))
    }

    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        ssbo: ShaderStorageBufferId,
        _ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        Err(gpu_api_err!(
            "webgpu read_synced_shader_storage_buffer is unimplemented."
        ))?;

        let ssbo = self.ssbos.get(ssbo.id()).ok_or(gpu_api_err!(
            "webgpu read synced shader buffer id {:?} does not exist",
            ssbo
        ))?;

        let mapped_buf = ssbo.buffer.get_mapped_range();
        let u8_js_buf = Uint8Array::new(&mapped_buf);
        let mut u8_rs_buf = vec![0u8; u8_js_buf.length() as usize];
        for i in 0..u8_js_buf.length() {
            u8_rs_buf[i as usize] = u8_js_buf.get_index(i);
        }

        Ok(unsafe { std::ptr::read(u8_rs_buf.as_ptr() as *const T) })
    }
}
