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
    pub resolve_attachment_views: Vec<GpuTextureView>,
}

impl WebGpuCompiledPass {
    pub fn new(context: &WebGpuContext, pass: &Pass, ext: Option<CompilePassExt>) -> GResult<Self> {
        let ext = ext.unwrap_or_default();

        let mut resolve_attachment_views = vec![];

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
                        .cull_mode(if program.ext.enable_culling.is_some() {
                            match program.ext.cull_mode.unwrap_or_default() {
                                ShaderCullMode::Front => GpuCullMode::Front,
                                ShaderCullMode::Back => GpuCullMode::Back,
                            }
                        } else {
                            GpuCullMode::None
                        })
                        .front_face(match program.ext.cull_front_face.unwrap_or_default() {
                            ShaderCullFrontFace::Clockwise => GpuFrontFace::Cw,
                            ShaderCullFrontFace::CounterClockwise => GpuFrontFace::Ccw,
                        })
                        .topology(GpuPrimitiveTopology::TriangleList);

                    let mut pipeline_info = GpuRenderPipelineDescriptor::new(&layout, &vertex);
                    pipeline_info
                        .primitive(&primitive);

                    if program.ext.enable_depth_write.is_some() || program.ext.enable_stencil_test.is_some() {
                        let mut depth_stencil = GpuDepthStencilState::new(WEBGPU_DEPTH_ATTACHMENT_FORMAT);
                        fn compare_op_into_webgpu(compare_op: ShaderCompareOp) -> GpuCompareFunction {
                            match compare_op {
                                ShaderCompareOp::Never => GpuCompareFunction::Never,
                                ShaderCompareOp::Less => GpuCompareFunction::Less,
                                ShaderCompareOp::Equal => GpuCompareFunction::Equal,
                                ShaderCompareOp::LessOrEqual => GpuCompareFunction::LessEqual,
                                ShaderCompareOp::Greater => GpuCompareFunction::Greater,
                                ShaderCompareOp::NotEqual => GpuCompareFunction::NotEqual,
                                ShaderCompareOp::GreaterOrEqual => GpuCompareFunction::GreaterEqual,
                                ShaderCompareOp::Always => GpuCompareFunction::Always,
                            }
                        }
                        fn stencil_op_into_webgpu(stencil_op: ShaderStencilOp) -> GpuStencilOperation {
                            match stencil_op {
                                ShaderStencilOp::Keep => GpuStencilOperation::Keep,
                                ShaderStencilOp::Zero => GpuStencilOperation::Zero,
                                ShaderStencilOp::Replace => GpuStencilOperation::Replace,
                                ShaderStencilOp::IncrementClamp => GpuStencilOperation::IncrementClamp,
                                ShaderStencilOp::DecrementClamp => GpuStencilOperation::DecrementClamp,
                                ShaderStencilOp::Invert => GpuStencilOperation::Invert,
                                ShaderStencilOp::IncrementWrap => GpuStencilOperation::IncrementWrap,
                                ShaderStencilOp::DecrementWrap => GpuStencilOperation::DecrementWrap,
                            }
                        }
                        let mut stencil_state = GpuStencilFaceState::new();
                        stencil_state
                            .compare(compare_op_into_webgpu(program.ext.stencil_compare_op.unwrap_or_default()))
                            .depth_fail_op(stencil_op_into_webgpu(program.ext.stencil_depth_fail.unwrap_or_default()))
                            .fail_op(stencil_op_into_webgpu(program.ext.stencil_fail.unwrap_or_default()))
                            .pass_op(stencil_op_into_webgpu(program.ext.stencil_pass.unwrap_or_default()));
                        depth_stencil
                            .depth_write_enabled(program.ext.enable_depth_write.is_some())
                            .depth_compare(compare_op_into_webgpu(program.ext.depth_compare_op.unwrap_or_default()))
                            .stencil_read_mask(program.ext.stencil_compare_mask.unwrap_or_default())
                            .stencil_write_mask(program.ext.stencil_write_mask.unwrap_or_default());

                        if program.ext.enable_stencil_test.is_some() {
                            depth_stencil
                                .stencil_back(&stencil_state)
                                .stencil_front(&stencil_state);
                        }

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
                            let format = if let Some(output_image) = attachment.output_image {
                                let attachment = context.attachment_images.get(output_image.id())
                                    .ok_or(gpu_api_err!("webpgpu compile pass attachment image id {:?} does not exist", output_image))?;
                                attachment.format
                            } else {
                                context
                                    .surface
                                    .as_ref()
                                    .ok_or(gpu_api_err!("webgpu surface does not exist, WebGpuInit extension was probably not called."))?
                                    .present_format

                            };
                            let target = GpuColorTargetState::new(format);

                            if program.ext.enable_blend.is_some() {
                                fn blend_factor_webgpu(factor: ShaderBlendFactor) -> GpuBlendFactor {
                                    match factor {
                                        ShaderBlendFactor::Zero => GpuBlendFactor::Zero,
                                        ShaderBlendFactor::One => GpuBlendFactor::One,
                                        ShaderBlendFactor::SrcColor => GpuBlendFactor::Src,
                                        ShaderBlendFactor::OneMinusSrcColor => GpuBlendFactor::OneMinusSrc,
                                        ShaderBlendFactor::SrcAlpha => GpuBlendFactor::SrcAlpha,
                                        ShaderBlendFactor::OneMinusSrcAlpha => GpuBlendFactor::OneMinusSrcAlpha,
                                        ShaderBlendFactor::DstColor => GpuBlendFactor::Dst,
                                        ShaderBlendFactor::OneMinusDstColor => GpuBlendFactor::OneMinusDst,
                                        ShaderBlendFactor::DstAlpha => GpuBlendFactor::DstAlpha,
                                        ShaderBlendFactor::OneMinusDstAlpha => GpuBlendFactor::OneMinusDstAlpha,
                                        ShaderBlendFactor::SrcAlphaSaturated => GpuBlendFactor::SrcAlphaSaturated,
                                        ShaderBlendFactor::ConstantColor |
                                            ShaderBlendFactor::ConstantAlpha => GpuBlendFactor::Constant,
                                        ShaderBlendFactor::OneMinusConstantColor |
                                            ShaderBlendFactor::OneMinusConstantAlpha => GpuBlendFactor::OneMinusConstant
                                    }
                                }

                                fn blend_op_webgpu(op: ShaderBlendOperation) -> GpuBlendOperation {
                                    match op {
                                        ShaderBlendOperation::Add => GpuBlendOperation::Add,
                                        ShaderBlendOperation::Subtract => GpuBlendOperation::Subtract,
                                        ShaderBlendOperation::ReverseSubtract => GpuBlendOperation::ReverseSubtract,
                                        ShaderBlendOperation::Min => GpuBlendOperation::Min,
                                        ShaderBlendOperation::Max => GpuBlendOperation::Max,
                                    }
                                }

                                let color_component = GpuBlendComponent::new();
                                color_component.operation(blend_op_webgpu(program.ext.blend_color_operation.unwrap_or_default()))
                                    .src_factor(blend_factor_webgpu(program.ext.blend_color_src_factor.unwrap_or_default()))
                                    .dst_factor(blend_factor_webgpu(program.ext.blend_color_dst_factor.unwrap_or_default()));
                                let alpha_component = GpuBlendComponent::new();
                                alpha_component.operation(blend_op_webgpu(program.ext.blend_alpha_operation.unwrap_or_default()))
                                    .src_factor(blend_factor_webgpu(program.ext.blend_alpha_src_factor.unwrap_or_default()))
                                    .dst_factor(blend_factor_webgpu(program.ext.blend_alpha_dst_factor.unwrap_or_default()));
                                let blend_state = GpuBlendState::new(&color_component, &alpha_component);
                                target.blend(&blend_state);
                            }

                            targets.push(&target);

                            if ext.enable_msaa.is_some() {
                                if let Some(sample_count) = ext.msaa_samples {
                                    let sample_count = match sample_count {
                                        MsaaSampleCount::Sample1 => 1,
                                        MsaaSampleCount::Sample2 => 2,
                                        MsaaSampleCount::Sample4 => 4,
                                        MsaaSampleCount::Sample8 => 8,
                                        MsaaSampleCount::Sample16 => 16,
                                        MsaaSampleCount::Sample32 => 32,
                                        MsaaSampleCount::Sample64 => 64,
                                    };
                                    let mut multisample = GpuMultisampleState::new();
                                    multisample.count(sample_count);
                                    pipeline_info.multisample(&multisample);

                                    let format = if pass.surface_attachment {
                                        context
                                            .surface
                                            .as_ref()
                                            .ok_or(gpu_api_err!("webgpu surface does not exist, WebGpuInit extension was probably not called."))?
                                            .present_format
                                    } else {
                                        format
                                    };

                                    let size = Array::new();
                                    size.push(&JsValue::from(pass.render_width));
                                    size.push(&JsValue::from(pass.render_height));

                                    let usage = GpuTextureUsageFlags::RenderAttachment as u32;

                                    let mut resolve_texture_info = GpuTextureDescriptor::new(format, &size, usage);
                                    resolve_texture_info.sample_count(sample_count);
                                    let resolve_texture = context.device.create_texture(&resolve_texture_info);
                                    let resolve_texture_view = resolve_texture.create_view();
                                    resolve_attachment_views.push(resolve_texture_view);
                                }
                            }

                            Ok(())
                        })?;
                        let fragment = GpuFragmentState::new("main", fragment_module, &targets);
                        pipeline_info.fragment(&fragment);
                    }

                    if ext.enable_msaa.is_some() {
                        if let Some(sample_count) = ext.msaa_samples {
                            let sample_count = match sample_count {
                                MsaaSampleCount::Sample1 => 1,
                                MsaaSampleCount::Sample2 => 2,
                                MsaaSampleCount::Sample4 => 4,
                                MsaaSampleCount::Sample8 => 8,
                                MsaaSampleCount::Sample16 => 16,
                                MsaaSampleCount::Sample32 => 32,
                                MsaaSampleCount::Sample64 => 64,
                            };
                            let mut multisample = GpuMultisampleState::new();
                            multisample.count(sample_count);
                            pipeline_info.multisample(&multisample);
                        }
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
            resolve_attachment_views,
            original_pass: pass.clone(),
        })
    }
}
