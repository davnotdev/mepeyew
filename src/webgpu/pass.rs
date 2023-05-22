use super::*;
use std::collections::HashMap;

impl WebGpuContext {
    pub fn compile_pass(
        &mut self,
        pass: &Pass,
        ext: Option<CompilePassExt>,
    ) -> GResult<CompiledPassId> {
        let pass = WebGpuCompiledPass::new(self, pass, ext)?;
        self.compiled_passes.push(pass);
        Ok(CompiledPassId::from_id(self.compiled_passes.len() - 1))
    }
}

pub struct WebGpuCompiledPass {
    pub ext: CompilePassExt,

    pub original_pass: Pass,
    pub pipelines: Vec<HashMap<ProgramId, GpuRenderPipeline>>,
    pub attachment_views: Vec<GpuTextureView>,
}

impl WebGpuCompiledPass {
    pub fn new(context: &WebGpuContext, pass: &Pass, ext: Option<CompilePassExt>) -> GResult<Self> {
        let ext = ext.unwrap_or_default();

        let pipelines = pass
            .steps
            .iter()
            .map(|step| {
                step.programs.iter().map(|&program_id| {
                    let program = context
                        .programs
                        .get(
                            program_id
                                .id(),
                        )
                        .ok_or(gpu_api_err!(
                            "webgpu pass step program id {:?} does not exist.",
                            program_id
                        ))?;
                    let vertex_buffers = Array::new();
                    vertex_buffers.push(&program.vertex_buffer_layout);
                    let mut vertex = GpuVertexState::new("main", &program.vertex_module);
                    vertex.buffers(&vertex_buffers);

                    let mut layout = JsValue::from_str("auto");
                    if !program.bind_group_layouts.is_empty() {
                        let layouts = Array::new();
                        program.bind_group_layouts.iter().for_each(|layout| {
                            layouts.push(layout);
                        });

                        let layout_info = GpuPipelineLayoutDescriptor::new(&layouts);
                        let pipeline_layout = context.device.create_pipeline_layout(&layout_info);
                        layout = pipeline_layout.into();
                    }

                    let mut primitive = GpuPrimitiveState::new();
                    primitive
                        .cull_mode(GpuCullMode::None)
                        .front_face(GpuFrontFace::Ccw)
                        .topology(GpuPrimitiveTopology::TriangleList);

                    let mut pipeline_info = GpuRenderPipelineDescriptor::new(&layout, &vertex);
                    pipeline_info
                        .primitive(&primitive);

                    if program.ext.enable_depth_test.is_some() {
                        let mut depth_stencil = GpuDepthStencilState::new(WEBGPU_DEPTH_ATTACHMENT_FORMAT);
                        depth_stencil
                            .depth_write_enabled(true)
                            .depth_compare(match program.ext.depth_compare_op.unwrap_or_default() {
                                ShaderCompareOp::Never => GpuCompareFunction::Never,
                                ShaderCompareOp::Less => GpuCompareFunction::Less,
                                ShaderCompareOp::Equal => GpuCompareFunction::Equal,
                                ShaderCompareOp::LessOrEqual => GpuCompareFunction::LessEqual,
                                ShaderCompareOp::Greater => GpuCompareFunction::Greater,
                                ShaderCompareOp::NotEqual => GpuCompareFunction::NotEqual,
                                ShaderCompareOp::GreaterOrEqual => GpuCompareFunction::GreaterEqual,
                                ShaderCompareOp::Always => GpuCompareFunction::Always,
                            });
                        pipeline_info.depth_stencil(&depth_stencil);
                    }

                    if let Some(fragment_module) = &program.fragment_module {
                        let targets = Array::new();
                        step.write_colors.iter().try_for_each(|write_color| {
                            let attachment =
                                pass.attachments.get(write_color.id()).ok_or(gpu_api_err!(
                                    "webgpu write color local attachment {:?} does not exist.",
                                    write_color
                                ))?;
                            let format = if attachment.output_image.is_none() {
                                context
                                    .surface
                                    .as_ref()
                                    .ok_or(gpu_api_err!("webgpu surface does not exist, WebGpuInit extension was probably not called."))?
                                    .present_format
                            } else {
                                WEBGPU_COLOR_ATTACHMENT_FORMAT
                            };
                            targets.push(&GpuColorTargetState::new(format));
                            Ok(())
                        })?;
                        let fragment = GpuFragmentState::new("main", fragment_module, &targets);
                        pipeline_info.fragment(&fragment);
                    }

                    Ok((program_id, context.device.create_render_pipeline(&pipeline_info)))

                }).collect::<GResult<HashMap<_, _>>>()
            })
            .collect::<GResult<Vec<_>>>()?;

        let attachment_views = pass
            .attachments
            .iter()
            .map(|attachment| {
                Ok(if let Some(attachment_image) = attachment.output_image {
                    let attachment_image = context
                        .attachment_images
                        .get(attachment_image.id())
                        .ok_or(gpu_api_err!(
                            "webgpu compile pass attachment image id {:?} does not exist",
                            attachment_image
                        ))?;
                    attachment_image.texture_view.clone()
                } else {
                    JsValue::null().into()
                })
            })
            .collect::<GResult<Vec<_>>>()?;

        Ok(WebGpuCompiledPass {
            ext,
            pipelines,
            attachment_views,
            original_pass: pass.clone(),
        })
    }
}
