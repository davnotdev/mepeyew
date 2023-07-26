use super::*;
use std::collections::HashMap;

impl WebGpuContext {
    pub fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        format: TextureFormat,
        ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        let texture =
            WebGpuTexture::new(&self.device, width, height, format, ext.unwrap_or_default());
        self.textures.push(texture);

        Ok(TextureId::from_id(self.textures.len() - 1))
    }

    pub fn upload_texture(
        &mut self,
        texture: TextureId,
        data: &[u8],
        ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        let ext = ext.unwrap_or_default();

        let texture = self.textures.get(texture.id()).ok_or(gpu_api_err!(
            "webgpu upload texture id {:?} does not exist",
            texture
        ))?;

        let queue = self.device.queue();
        let size = Array::new();
        size.push(&JsValue::from(texture.width));
        size.push(&JsValue::from(texture.height));

        let mut layout = GpuImageDataLayout::new();
        layout.offset(0.0).bytes_per_row(texture.width as u32 * 4);

        queue.write_texture_with_u8_array_and_u32_sequence(
            &GpuImageCopyTexture::new(&texture.texture),
            data,
            &layout,
            &size,
        );

        if ext.generate_mipmaps.is_some() {
            self.mipmap_state_cache
                .generate_mipmap(&self.device, texture);
        }

        Ok(())
    }

    pub fn get_texture_max_lod(&self, texture: TextureId) -> GResult<f32> {
        let texture = self.textures.get(texture.id()).ok_or(gpu_api_err!(
            "webgpu get_texture_max_lod texture {:?} does not exist",
            texture
        ))?;
        Ok(texture.mip_levels as f32)
    }
}

pub struct WebGpuTexture {
    texture: GpuTexture,
    pub texture_view: GpuTextureView,
    usage: u32,
    width: usize,
    height: usize,
    mip_levels: u32,
    format: GpuTextureFormat,
    original_format: TextureFormat,
}

impl WebGpuTexture {
    pub fn new(
        device: &GpuDevice,
        width: usize,
        height: usize,
        format: TextureFormat,
        ext: NewTextureExt,
    ) -> Self {
        let mip_levels = if ext.enable_mipmaps.is_some() {
            ext.mip_levels
                .unwrap_or(std::cmp::max(width, height).ilog2())
        } else {
            1
        };

        let texture_format = match format {
            TextureFormat::Rgba => GpuTextureFormat::Rgba8unorm,
        };

        let size = Array::new();
        size.push(&JsValue::from(width));
        size.push(&JsValue::from(height));

        let usage =
            GpuTextureUsageFlags::CopyDst as u32 | GpuTextureUsageFlags::TextureBinding as u32;

        let mut texture_info = GpuTextureDescriptor::new(texture_format, &size, usage);
        texture_info.mip_level_count(mip_levels);
        let texture = device.create_texture(&texture_info);
        let texture_view = texture.create_view();

        WebGpuTexture {
            texture,
            texture_view,
            usage,
            width,
            height,
            mip_levels,
            format: texture_format,
            original_format: format,
        }
    }
}

//  Derived from https://github.com/toji/web-texture-tool/blob/main/src/webgpu-mipmap-generator.js
//  Thank you Brandon Jones (toji)!
pub struct WebGpuMipmapStateCache {
    sampler: GpuSampler,
    module: GpuShaderModule,
    pipeline_layout: GpuPipelineLayout,
    pipelines: HashMap<TextureFormat, GpuRenderPipeline>,
}

impl WebGpuMipmapStateCache {
    pub fn new(device: &GpuDevice) -> Self {
        let code = r#"
            var<private> pos : array<vec2<f32>, 3> = array<vec2<f32>, 3>(
              vec2<f32>(-1.0, -1.0), vec2<f32>(-1.0, 3.0), vec2<f32>(3.0, -1.0));

            struct VertexOutput {
              @builtin(position) position : vec4<f32>,
              @location(0) texCoord : vec2<f32>,
            };

            @vertex
            fn vertexMain(@builtin(vertex_index) vertexIndex : u32) -> VertexOutput {
              var output : VertexOutput;
              output.texCoord = pos[vertexIndex] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
              output.position = vec4<f32>(pos[vertexIndex], 0.0, 1.0);
              return output;
            }

            @group(0) @binding(0) var imgSampler : sampler;
            @group(0) @binding(1) var img : texture_2d<f32>;

            @fragment
            fn fragmentMain(@location(0) texCoord : vec2<f32>) -> @location(0) vec4<f32> {
              return textureSample(img, imgSampler, texCoord);
            }"#;

        let module = device.create_shader_module(&GpuShaderModuleDescriptor::new(code));

        let pipeline_layout_entries = Array::new();
        pipeline_layout_entries.push(
            GpuBindGroupLayoutEntry::new(0, GpuShaderStageFlags::Fragment as u32)
                .sampler(&GpuSamplerBindingLayout::new()),
        );
        pipeline_layout_entries.push(
            GpuBindGroupLayoutEntry::new(1, GpuShaderStageFlags::Fragment as u32)
                .texture(&GpuTextureBindingLayout::new()),
        );

        let bind_group_layouts = Array::new();
        bind_group_layouts.push(&device.create_bind_group_layout(
            &GpuBindGroupLayoutDescriptor::new(&pipeline_layout_entries),
        ));

        let pipeline_layout =
            device.create_pipeline_layout(&GpuPipelineLayoutDescriptor::new(&bind_group_layouts));

        let sampler = device.create_sampler_with_descriptor(
            GpuSamplerDescriptor::new().min_filter(GpuFilterMode::Linear),
        );

        WebGpuMipmapStateCache {
            module,
            sampler,
            pipeline_layout,
            pipelines: HashMap::new(),
        }
    }

    pub fn generate_mipmap(&mut self, device: &GpuDevice, texture: &WebGpuTexture) {
        let size = Array::new();
        size.push(&JsValue::from(texture.width / 2));
        size.push(&JsValue::from(texture.height / 2));

        let mip_texture = device.create_texture(
            GpuTextureDescriptor::new(
                texture.format,
                &size,
                texture.usage
                    | GpuTextureUsageFlags::RenderAttachment as u32
                    | GpuTextureUsageFlags::CopySrc as u32,
            )
            .mip_level_count(texture.mip_levels - 1),
        );

        let pipeline = if let Some(pipeline) = self.pipelines.get(&texture.original_format) {
            pipeline.clone()
        } else {
            let fragment_targets = Array::new();
            fragment_targets.push(&GpuColorTargetState::new(texture.format));

            let pipeline = device.create_render_pipeline(
                GpuRenderPipelineDescriptor::new(
                    &self.pipeline_layout,
                    &GpuVertexState::new("vertexMain", &self.module),
                )
                .primitive(
                    GpuPrimitiveState::new()
                        .topology(GpuPrimitiveTopology::TriangleStrip)
                        .strip_index_format(GpuIndexFormat::Uint32),
                )
                .fragment(&GpuFragmentState::new(
                    "fragmentMain",
                    &self.module,
                    &fragment_targets,
                )),
            );
            self.pipelines
                .insert(texture.original_format, pipeline.clone());
            pipeline
        };

        let src_view = texture.texture.create_view_with_descriptor(
            GpuTextureViewDescriptor::new()
                .base_mip_level(0)
                .mip_level_count(1),
        );

        let command_encoder = device.create_command_encoder();

        for i in 1..texture.mip_levels {
            let dst_view = mip_texture.create_view_with_descriptor(
                GpuTextureViewDescriptor::new()
                    .base_mip_level(i - 1)
                    .mip_level_count(1),
            );

            let color_attachments = Array::new();
            color_attachments.push(&GpuRenderPassColorAttachment::new(
                GpuLoadOp::Clear,
                GpuStoreOp::Store,
                &dst_view,
            ));
            let pass_encoder = command_encoder
                .begin_render_pass(&GpuRenderPassDescriptor::new(&color_attachments));

            let bind_group_entries = Array::new();
            bind_group_entries.push(&GpuBindGroupEntry::new(0, &self.sampler));
            bind_group_entries.push(&GpuBindGroupEntry::new(1, &src_view));
            let bind_group = device.create_bind_group(&GpuBindGroupDescriptor::new(
                &bind_group_entries,
                &pipeline.get_bind_group_layout(0),
            ));

            pass_encoder.set_pipeline(&pipeline);
            pass_encoder.set_bind_group(0, &bind_group);
            pass_encoder.draw(4);
            pass_encoder.end();
        }

        let mut mip_level_width = texture.width / 2;
        let mut mip_level_height = texture.height / 2;
        for i in 1..texture.mip_levels {
            command_encoder.copy_texture_to_texture_with_gpu_extent_3d_dict(
                GpuImageCopyTexture::new(&mip_texture).mip_level(i - 1),
                GpuImageCopyTexture::new(&texture.texture).mip_level(i),
                GpuExtent3dDict::new(mip_level_width as u32).height(mip_level_height as u32),
            );

            mip_level_width /= 2;
            mip_level_height /= 2;
        }

        let submits = Array::new();
        submits.push(&command_encoder.finish());
        device.queue().submit(&submits);
    }
}
