use super::*;
use std::collections::HashMap;

pub struct VkCompiledPass {
    pub render_pass: vk::RenderPass,
    pub pipelines: Vec<HashMap<ProgramId, vk::Pipeline>>,
    pub framebuffer: VkFramebuffer,
    pub render_extent: vk::Extent2D,

    pub original_pass: Pass,
    pub original_ext: CompilePassExt,
    //  Although these fields are derived from pass, let's still keep these.
    //  I believe that this should help with readability. (?)
    pub steps: Vec<PassStep>,
    pub attachment_count: usize,
    pub should_present: bool,

    pub resolve_images: Vec<(VkImage, vk::ImageView)>,
    //  attachment index -> actual attachment index
    pub resolve_image_offsets: HashMap<usize, usize>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkCompiledPass {
    pub fn new(context: &mut VkContext, pass: &Pass, ext: &CompilePassExt) -> GResult<Self> {
        //  Create render pass.
        let NewRenderPassOutput {
            render_pass,
            sample_count,
            resolve_images,
            resolve_image_offsets,
        } = new_render_pass(context, pass, ext)?;

        let resolve_images_views = resolve_images
            .iter()
            .map(|&(_, image_view)| image_view)
            .collect::<Vec<_>>();

        //  Patch framebuffers.
        let framebuffer = new_framebuffer(
            context,
            pass,
            render_pass,
            &resolve_images_views,
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
                step.programs
                    .iter()
                    .map(|&program_id| {
                        let program = context.programs.get(program_id.id()).unwrap();
                        Ok((
                            program_id,
                            program.new_graphics_pipeline(
                                &context.core.dev,
                                render_pass,
                                render_extent,
                                subpass_idx,
                                sample_count,
                                &program.ext,
                            )?,
                        ))
                    })
                    .collect::<GResult<HashMap<_, _>>>()
            })
            .collect::<GResult<Vec<_>>>()?;

        //  Done!
        Ok(VkCompiledPass {
            render_pass,
            pipelines,
            framebuffer,
            render_extent,

            original_pass: pass.clone(),
            original_ext: ext.clone(),
            steps: pass.steps.clone(),
            attachment_count: pass.attachments.len(),
            should_present: pass.surface_attachment,

            resolve_images,
            resolve_image_offsets,

            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
    }
}

impl VkContext {
    pub fn compile_pass(
        &mut self,
        pass: &Pass,
        ext: Option<CompilePassExt>,
    ) -> GResult<CompiledPassId> {
        let ext = ext.unwrap_or_default();
        let compiled_pass = VkCompiledPass::new(self, pass, &ext)?;
        self.compiled_passes.push(compiled_pass);
        Ok(CompiledPassId::from_id(self.compiled_passes.len() - 1))
    }
}

fn new_framebuffer(
    ctx: &VkContext,
    pass: &Pass,
    render_pass: vk::RenderPass,
    resolve_images: &[vk::ImageView],
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
                Some(attachment.output_image.unwrap())
            }
        })
        .collect::<Vec<_>>();

    VkFramebuffer::new(
        ctx,
        render_pass,
        &images,
        resolve_images,
        width,
        height,
        pass.surface_attachment,
    )
}

struct NewRenderPassOutput {
    render_pass: vk::RenderPass,
    sample_count: Option<vk::SampleCountFlags>,
    resolve_images: Vec<(VkImage, vk::ImageView)>,
    resolve_image_offsets: HashMap<usize, usize>,
}

fn new_render_pass(
    ctx: &mut VkContext,
    pass: &Pass,
    ext: &CompilePassExt,
) -> GResult<NewRenderPassOutput> {
    let swapchain_format = ctx
        .surface_ext
        .as_ref()
        .map(|surface_ext| surface_ext.swapchain.format);

    //  Use the attachment index order used by pass's local attachment indices.
    //  Remember to be careful because `surface_attachment` should be attachment index 0.
    let mut pass_input_attachments = pass
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
                    .stencil_load_op(match &attachment.ty {
                        PassInputType::Depth(load_op) => match load_op {
                            PassInputLoadOpDepthStencilType::Load => vk::AttachmentLoadOp::LOAD,
                            PassInputLoadOpDepthStencilType::Clear => vk::AttachmentLoadOp::CLEAR,
                        },
                        _ => vk::AttachmentLoadOp::DONT_CARE,
                    })
                    .stencil_store_op(vk::AttachmentStoreOp::STORE)
                    .initial_layout(
                    match &attachment.ty {
                        PassInputType::Color(load_op) => match load_op {
                            PassInputLoadOpColorType::Load => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            PassInputLoadOpColorType::Clear => vk::ImageLayout::UNDEFINED,
                        },
                        PassInputType::Depth(load_op) => match load_op {
                            PassInputLoadOpDepthStencilType::Load => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                            PassInputLoadOpDepthStencilType::Clear => vk::ImageLayout::UNDEFINED,
                        },
                    }
                        )
                    .final_layout(match attachment.ty {
                        PassInputType::Color(_) => {
                            if attachment.local_attachment_idx == 0 && pass.surface_attachment {
                                vk::ImageLayout::PRESENT_SRC_KHR
                            } else {
                                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                            }
                        }
                        PassInputType::Depth(_) => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    })
                    .build(),
                vk::AttachmentReference::builder()
                    .attachment(attachment.local_attachment_idx as u32)
                    .layout(match attachment.ty {
                        PassInputType::Color(_) => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        PassInputType::Depth(_) => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    })
                    .build(),
            ))
        })
        .collect::<GResult<Vec<_>>>()?;

    let mut sample_count = None;

    //  Assuming that MSAA is enabled:
    //  This is later appended onto `pass_input_attachments`.
    let mut i_pass_resolve_attachments = 0;
    let mut resolve_image_offsets = HashMap::new();
    let (pass_resolve_attachments, resolve_images) = if ext.enable_msaa.is_some() {
        let sample_ty = match ext.msaa_samples.unwrap_or_default() {
            MsaaSampleCount::Sample1 => vk::SampleCountFlags::TYPE_1,
            MsaaSampleCount::Sample2 => vk::SampleCountFlags::TYPE_2,
            MsaaSampleCount::Sample4 => vk::SampleCountFlags::TYPE_4,
            MsaaSampleCount::Sample8 => vk::SampleCountFlags::TYPE_8,
            MsaaSampleCount::Sample16 => vk::SampleCountFlags::TYPE_16,
            MsaaSampleCount::Sample32 => vk::SampleCountFlags::TYPE_32,
            MsaaSampleCount::Sample64 => vk::SampleCountFlags::TYPE_64,
        };

        sample_count = Some(sample_ty);
        pass_input_attachments
            .iter_mut()
            .for_each(|(desc, reference)| {
                if reference.layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
                    desc.samples = sample_ty;
                }
            });

        pass_input_attachments
            .clone()
            .iter()
            .enumerate()
            .filter(|(_, (_, reference))| {
                matches!(
                    reference.layout,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL | vk::ImageLayout::PRESENT_SRC_KHR
                )
            })
            .map(|(idx, (mut desc, mut reference))| {
                let original_attachment_lengths = pass_input_attachments.len();
                let original = &mut pass_input_attachments[idx].0;
                desc.samples = sample_ty;

                original.load_op = vk::AttachmentLoadOp::DONT_CARE;
                original.stencil_load_op = vk::AttachmentLoadOp::DONT_CARE;

                reference.attachment =
                    (i_pass_resolve_attachments + original_attachment_lengths) as u32;

                let resolve_image = new_resolve_image(
                    ctx,
                    vk::Extent3D {
                        width: pass.render_width as u32,
                        height: pass.render_height as u32,
                        depth: 1,
                    },
                    original.format,
                    original.final_layout,
                    sample_ty,
                )?;

                let resolve_images_views = new_image_view(
                    &ctx.core.dev,
                    resolve_image.image,
                    resolve_image.format,
                    resolve_image.view_aspect,
                )?;

                resolve_image_offsets.insert(idx, i_pass_resolve_attachments);
                i_pass_resolve_attachments += 1;

                Ok(((desc, reference), (resolve_image, resolve_images_views)))
            })
            .collect::<GResult<Vec<_>>>()?
            .into_iter()
            .unzip()
    } else {
        (vec![], vec![])
    };

    //  These values must outlive subpasses.
    let mut each_total_input_attachments = vec![];
    let mut each_color_input_attachments = vec![];
    let mut each_resolve_attachments = vec![];
    let mut each_depth_ptrs = vec![];

    let subpasses = pass
        .steps
        .iter()
        .map(|step| {
            let color_input_attachments = step
                .write_colors
                .iter()
                .map(|dep| pass_input_attachments[dep.id()].1)
                .collect::<Vec<_>>();

            let input_attachments = step
                .read_attachment
                .iter()
                .map(|local| {
                    vk::AttachmentReference::builder()
                        .attachment(local.id() as u32)
                        .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .build()
                })
                .collect::<Vec<_>>();

            let mut subpass = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .input_attachments(&input_attachments)
                .color_attachments(&color_input_attachments)
                .build();

            if let Some(depth) = step.write_depth {
                let depth = Box::new(pass_input_attachments[depth.id()].1);
                subpass.p_depth_stencil_attachment =
                    depth.as_ref() as *const vk::AttachmentReference;
                each_depth_ptrs.push(depth);
            }

            if ext.enable_msaa.is_some() {
                let old = subpass.p_color_attachments;
                let resolve_attachments = step
                    .write_colors
                    .iter()
                    .map(|local| pass_resolve_attachments[resolve_image_offsets[&local.id()]].1)
                    .collect::<Vec<_>>();
                subpass.p_resolve_attachments = old;
                subpass.p_color_attachments = resolve_attachments.as_ptr();
                each_resolve_attachments.push(resolve_attachments);
            }

            each_total_input_attachments.push(input_attachments);
            each_color_input_attachments.push(color_input_attachments);

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
                                ShaderStage::Vertex => vk::PipelineStageFlags::VERTEX_SHADER,
                                ShaderStage::Fragment => vk::PipelineStageFlags::FRAGMENT_SHADER,
                            })
                            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                            .dst_access_mask(vk::AccessFlags::SHADER_READ);
                        subpass_dep.build()
                    }),
                step.wait_for_depth_from
                    .as_ref()
                    .map(|(wait_for_depth, _)| {
                        let subpass_dep = vk::SubpassDependency::builder()
                            .src_subpass(wait_for_depth.id() as u32)
                            .dst_subpass(subpass_idx as u32)
                            //  TODO CHK: Is this valid use?
                            .src_stage_mask(
                                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            )
                            .dst_stage_mask(
                                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            )
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
        .chain(pass_resolve_attachments.iter())
        .map(|a| a.0)
        .collect::<Vec<_>>();

    let render_pass_create = vk::RenderPassCreateInfo::builder()
        .dependencies(&subpass_dependencies)
        .attachments(&attachments)
        .subpasses(&subpasses)
        .build();

    let render_pass = unsafe { ctx.core.dev.create_render_pass(&render_pass_create, None) }
        .map_err(|e| gpu_api_err!("vulkan render pass {}", e))?;

    Ok(NewRenderPassOutput {
        render_pass,
        sample_count,
        resolve_images,
        resolve_image_offsets,
    })
}

fn new_resolve_image(
    ctx: &mut VkContext,
    extent: vk::Extent3D,
    format: vk::Format,
    layout: vk::ImageLayout,
    sample_count: vk::SampleCountFlags,
) -> GResult<VkImage> {
    let (format, usage, aspect) = match layout {
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL | vk::ImageLayout::PRESENT_SRC_KHR => (
            format,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::ImageAspectFlags::COLOR,
        ),
        _ => unreachable!(),
    };

    VkImage::new(
        &ctx.core.dev,
        &ctx.drop_queue,
        &mut ctx.alloc,
        format,
        usage,
        aspect,
        sample_count,
        extent,
    )
}

impl Drop for VkCompiledPass {
    fn drop(&mut self) {
        let render_pass = self.render_pass;
        let pipelines = self.pipelines.clone();
        let resolve_image_views = self
            .resolve_images
            .iter()
            .map(|&(_, image_view)| image_view)
            .collect::<Vec<_>>();

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_render_pass(render_pass, None);
                pipelines.iter().for_each(|pipelines| {
                    pipelines.values().for_each(|&pipeline| {
                        dev.destroy_pipeline(pipeline, None);
                    })
                });
                resolve_image_views.iter().for_each(|&image_view| {
                    dev.destroy_image_view(image_view, None);
                })
            }))
    }
}
