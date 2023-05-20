use super::*;

impl WebGpuContext {
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        sampler: SamplerId,
        format: TextureFormat,
        ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        let texture = WebGpuTexture::new(
            &self.device,
            width,
            height,
            sampler,
            format,
            ext.unwrap_or_default(),
        );
        self.textures.push(texture);

        Ok(TextureId::from_id(self.textures.len() - 1))
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
        texture: TextureId,
        data: &[u8],
        _ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        let texture = self.textures.get(texture.id()).ok_or(gpu_api_err!(
            "webgpu upload texture id {:?} does not exist",
            texture
        ))?;

        let queue = self.device.queue();
        let size = Array::new();
        size.push(&JsValue::from(texture.width));
        size.push(&JsValue::from(texture.height));
        queue.write_texture_with_u8_array_and_u32_sequence(
            &GpuImageCopyTexture::new(&texture.texture),
            data,
            &GpuImageDataLayout::new(),
            &size,
        );

        Ok(())
    }
}

pub struct WebGpuTexture {
    sampler: SamplerId,
    texture: GpuTexture,
    texture_view: GpuTextureView,
    width: usize,
    height: usize,
}

impl WebGpuTexture {
    pub fn new(
        device: &GpuDevice,
        width: usize,
        height: usize,
        sampler: SamplerId,
        format: TextureFormat,
        _ext: NewTextureExt,
    ) -> Self {
        let format = match format {
            TextureFormat::Rgba => GpuTextureFormat::Rgba8uint,
        };

        let size = Array::new();
        size.push(&JsValue::from(width));
        size.push(&JsValue::from(height));

        let usage =
            GpuTextureUsageFlags::CopyDst as u32 | GpuTextureUsageFlags::RenderAttachment as u32;

        let texture_info = GpuTextureDescriptor::new(format, &size, usage);
        let texture = device.create_texture(&texture_info);
        let texture_view = texture.create_view();

        WebGpuTexture {
            sampler,
            texture,
            texture_view,
            width,
            height,
        }
    }
}
