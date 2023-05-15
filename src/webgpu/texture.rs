use super::*;

impl WebGpuContext {
    pub fn new_texture(
        &mut self,
        _width: usize,
        _height: usize,
        _sampler: SamplerId,
        _format: TextureFormat,
        _ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        todo!()
    }

    pub fn resize_texture(
        &mut self,
        _texture: TextureId,
        _width: usize,
        _height: usize,
        _ext: Option<ResizeTextureExt>,
    ) -> GResult<()> {
        todo!()
    }

    pub fn upload_texture(
        &mut self,
        _texture: TextureId,
        _data: &[u8],
        _ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        todo!()
    }
}
