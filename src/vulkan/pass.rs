use super::*;

pub struct VkCompiledPass {
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<vk::Pipeline>,
    pub framebuffer: VkFramebuffer,
    pub render_extent: vk::Extent2D,

    pub original_pass: Pass,
    //  Although these fields are derived from pass, let's still keep these.
    //  I believe that this should help with readability. (?)
    pub steps: Vec<PassStep>,
    pub attachment_count: usize,
    pub should_present: bool,

    drop_queue_ref: VkDropQueueRef,
}

impl VkCompiledPass {
    pub fn new(context: &VkContext, pass: &Pass) -> GResult<Self> {
        //  Create render pass.
        let render_pass = new_render_pass(context, pass)?;

        //  Patch framebuffers.
        let framebuffer = new_framebuffer(
            context,
            pass,
            render_pass,
            pass.render_width,
            pass.render_height,
        )?;

        let render_extent = vk::Extent2D::builder()
            .width(pass.render_width as u32)
            .height(pass.render_height as u32)
            .build();

        //  Create one pipeline per subpass.
        let pipelines = pass
            .steps
            .iter()
            .enumerate()
            .map(|(subpass_idx, step)| {
                let program = context
                    .programs
                    .get(
                        step.program
                            .ok_or(gpu_api_err!("vulkan pass step has no shader"))?
                            .id(),
                    )
                    .unwrap();
                program.new_graphics_pipeline(
                    &context.core.dev,
                    render_pass,
                    render_extent,
                    subpass_idx,
                )
            })
            .collect::<GResult<Vec<_>>>()?;

        //  Done!
        Ok(VkCompiledPass {
            render_pass,
            pipelines,
            framebuffer,
            render_extent,

            original_pass: pass.clone(),
            steps: pass.steps.clone(),
            attachment_count: pass.attachments.len(),
            should_present: pass.surface_attachment,

            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
    }
}

impl VkContext {
    pub fn compile_pass(
        &mut self,
        pass: &Pass,
        _ext: Option<CompilePassExt>,
    ) -> GResult<CompiledPassId> {
        let compiled_pass = VkCompiledPass::new(self, pass)?;
        self.compiled_passes.push(compiled_pass);
        Ok(CompiledPassId::from_id(self.compiled_passes.len() - 1))
    }
}

fn new_framebuffer(
    ctx: &VkContext,
    pass: &Pass,
    render_pass: vk::RenderPass,
    width: usize,
    height: usize,
) -> GResult<VkFramebuffer> {
    let images = pass
        .attachments
        .iter()
        .filter_map(|attachment| {
            if pass.surface_attachment && attachment.local_attachment_idx == 0 {
                None
            } else {
                Some(attachment.output_image)
            }
        })
        .collect::<Vec<_>>();

    VkFramebuffer::new(
        ctx,
        render_pass,
        &images,
        width,
        height,
        pass.surface_attachment,
    )
}

fn new_render_pass(ctx: &VkContext, pass: &Pass) -> GResult<vk::RenderPass> {
    let swapchain_format = ctx
        .surface_ext
        .as_ref()
        .map(|surface_ext| surface_ext.swapchain.format);

    //  Use the attachment index order used by pass's local attachment indices.
    //  Remember to be careful because `surface_attachment` should be attachment index 0.
    let pass_input_attachments = pass
        .attachments
        .iter()
        .map(|attachment| {
            Ok((
                vk::AttachmentDescription::builder()
                    .format(match attachment.ty {
                        PassInputType::Color(_) => {
                            if attachment.local_attachment_idx == 0 && pass.surface_attachment {
                                swapchain_format.ok_or(gpu_api_err!("vulkan tried to use surface attachment without surface extension"))?
                            } else {
                                VK_COLOR_ATTACHMENT_FORMAT
                            }
                        }
                        PassInputType::Depth(_) => VK_DEPTH_ATTACHMENT_FORMAT,
                    })
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(match &attachment.ty {
                        PassInputType::Color(load_op) => match load_op {
                            PassInputLoadOpColorType::Load => vk::AttachmentLoadOp::LOAD,
                            PassInputLoadOpColorType::Clear => vk::AttachmentLoadOp::CLEAR,
                        },
                        PassInputType::Depth(load_op) => match load_op {
                            PassInputLoadOpDepthStencilType::Load => vk::AttachmentLoadOp::LOAD,
                            PassInputLoadOpDepthStencilType::Clear => vk::AttachmentLoadOp::CLEAR,
                        },
                    })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(match attachment.ty {
                        PassInputType::Color(_) => {
                            if attachment.local_attachment_idx == 0 && pass.surface_attachment {
                                vk::ImageLayout::PRESENT_SRC_KHR
                            } else {
                                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                            }
                        }
                        PassInputType::Depth(_) => vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                    })
                    .build(),
                vk::AttachmentReference::builder()
                    .attachment(attachment.local_attachment_idx as u32)
                    .layout(match attachment.ty {
                        PassInputType::Color(_) => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        PassInputType::Depth(_) => vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                    })
                    .build(),
            ))
        })
        .collect::<GResult<Vec<_>>>()?;

    //  These values must outlive subpasses.
    let mut each_total_input_attachments = vec![];
    let mut each_color_input_attachments = vec![];
    let mut each_depth_input_attachments = vec![];

    let subpasses = pass
        .steps
        .iter()
        .map(|step| {
            let color_input_attachments = step
                .write_colors
                .iter()
                .map(|dep| pass_input_attachments[dep.id()].1)
                .collect::<Vec<_>>();
            let mut total_input_attachments = color_input_attachments.clone();
            let depth_input_attachment = step.write_depth.map(|write_depth| {
                let attachment = pass_input_attachments[write_depth.id()].1;
                total_input_attachments.push(attachment);
                attachment
            });

            let partial = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                // .input_attachments(&total_input_attachments)
                .color_attachments(&color_input_attachments);
            let partial = if let Some(depth) = &depth_input_attachment {
                partial.depth_stencil_attachment(depth)
            } else {
                partial
            };

            let subpass = partial.build();

            each_total_input_attachments.push(total_input_attachments);
            each_color_input_attachments.push(color_input_attachments);
            each_depth_input_attachments.push(depth_input_attachment);

            subpass
        })
        .collect::<Vec<_>>();

    let subpass_dependencies = pass
        .steps
        .iter()
        .enumerate()
        .flat_map(|(subpass_idx, step)| {
            //  Reminder of stage orders:
            //
            //  TOP_OF_PIPE_BIT
            //  DRAW_INDIRECT_BIT
            //  VERTEX_INPUT_BIT
            //  VERTEX_SHADER_BIT
            //  TESSELLATION_CONTROL_SHADER_BIT
            //  TESSELLATION_EVALUATION_SHADER_BIT
            //  GEOMETRY_SHADER_BIT
            //  FRAGMENT_SHADER_BIT
            //  EARLY_FRAGMENT_TESTS_BIT
            //  LATE_FRAGMENT_TESTS_BIT
            //  COLOR_ATTACHMENT_OUTPUT_BIT
            //  TRANSFER_BIT
            //  COMPUTE_SHADER_BIT
            //  BOTTOM_OF_PIPE_BIT

            vec![
                Some(
                    vk::SubpassDependency::builder()
                        .src_subpass(vk::SUBPASS_EXTERNAL)
                        .dst_subpass(0)
                        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                        .build(),
                ),
                step.wait_for_color_from
                    .as_ref()
                    .map(|(wait_for_color, shader_stage_usage)| {
                        let subpass_dep = vk::SubpassDependency::builder()
                            .src_subpass(wait_for_color.id() as u32)
                            .dst_subpass(subpass_idx as u32)
                            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                            .dst_stage_mask(match shader_stage_usage {
                                ShaderType::Vertex => vk::PipelineStageFlags::VERTEX_SHADER,
                                ShaderType::Fragment => vk::PipelineStageFlags::VERTEX_SHADER,
                            })
                            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ);
                        subpass_dep.build()
                    }),
                step.wait_for_depth_from
                    .as_ref()
                    .map(|(wait_for_depth, shader_stage_usage)| {
                        let subpass_dep = vk::SubpassDependency::builder()
                            .src_subpass(wait_for_depth.id() as u32)
                            .dst_subpass(subpass_idx as u32)
                            .src_stage_mask(
                                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            )
                            .dst_stage_mask(match shader_stage_usage {
                                ShaderType::Vertex => vk::PipelineStageFlags::VERTEX_SHADER,
                                ShaderType::Fragment => vk::PipelineStageFlags::VERTEX_SHADER,
                            })
                            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                            .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ);
                        subpass_dep.build()
                    }),
            ]
        })
        .filter(|sd| sd.is_some())
        .collect::<Option<Vec<_>>>()
        .unwrap();

    let attachments = pass_input_attachments
        .iter()
        .map(|a| a.0)
        .collect::<Vec<_>>();
    let render_pass_create = vk::RenderPassCreateInfo::builder()
        .dependencies(&subpass_dependencies)
        .attachments(&attachments)
        .subpasses(&subpasses)
        .build();

    unsafe { ctx.core.dev.create_render_pass(&render_pass_create, None) }
        .map_err(|e| gpu_api_err!("vulkan render pass {}", e))
}

impl Drop for VkCompiledPass {
    fn drop(&mut self) {
        let render_pass = self.render_pass;
        let pipelines = self.pipelines.clone();

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_render_pass(render_pass, None);
                pipelines
                    .iter()
                    .for_each(|&pipeline| dev.destroy_pipeline(pipeline, None));
            }))
    }
}
