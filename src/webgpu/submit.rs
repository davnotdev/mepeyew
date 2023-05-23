use super::*;

impl WebGpuContext {
    pub fn submit(&mut self, submit: Submit, _ext: Option<SubmitExt>) -> GResult<()> {
        submit_transfers(self, &submit)?;

        let submissions = Array::new();

        submit.passes.iter().try_for_each(|pass| {
            submit_pass(self, pass)?.iter().for_each(|submit| {
                submissions.push(&submit.finish());
            });
            Ok(())
        })?;

        self.device.queue().submit(&submissions);

        Ok(())
    }
}

fn submit_transfers(context: &WebGpuContext, submit: &Submit) -> GResult<()> {
    let queue = context.device.queue();
    submit.vbo_transfers.iter().try_for_each(|(vbo_id, data)| {
        let vbo = context.vbos.get(vbo_id.id()).ok_or(gpu_api_err!(
            "webgpu submit transfers vbo id {:?} does not exist",
            vbo_id
        ))?;
        queue.write_buffer_with_u32_and_u8_array(&vbo.buffer, 0, unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<VertexBufferElement>(),
            )
        });
        Ok(())
    })?;

    submit.ibo_transfers.iter().try_for_each(|(ibo_id, data)| {
        let ibo = context.ibos.get(ibo_id.id()).ok_or(gpu_api_err!(
            "webgpu submit transfers ibo id {:?} does not exist",
            ibo_id
        ))?;
        queue.write_buffer_with_u32_and_u8_array(&ibo.buffer, 0, unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<IndexBufferId>(),
            )
        });
        Ok(())
    })?;

    submit.ubo_transfers.iter().try_for_each(|(ubo_id, data)| {
        let ubo = context.ubos.get(ubo_id.id()).ok_or(gpu_api_err!(
            "webgpu submit transfers ubo id {:?} does not exist",
            ubo_id
        ))?;
        queue.write_buffer_with_u32_and_u8_array(&ubo.buffer, 0, data);
        Ok(())
    })?;

    Ok(())
}

fn submit_pass(
    context: &WebGpuContext,
    pass_submit: &PassSubmitData,
) -> GResult<Vec<GpuCommandEncoder>> {
    let pass = context
        .compiled_passes
        .get(pass_submit.pass.id())
        .ok_or(gpu_api_err!(
            "webgpu submit pass id {:?} does not exist",
            pass_submit.pass.id()
        ))?;

    let command_encoder = context.device.create_command_encoder();

    pass.original_pass
        .steps
        .iter()
        .zip(pass_submit.steps_datas.iter())
        .enumerate()
        .try_for_each(|(step_idx, (step, step_data))| {
            let color_attachments = Array::new();
            let mut depth_attachment = None;

            let surface_view = context
                .surface
                .as_ref()
                .map(|surface| surface.context.get_current_texture().create_view());

            step.write_colors.iter().try_for_each(|write_color| {
                let attachment =
                    pass.original_pass
                        .attachments
                        .get(write_color.id())
                        .ok_or(gpu_api_err!(
                            "webgpu submit write color {:?} does not exist",
                            write_color
                        ))?;

                let local_attachment_idx = write_color.id();

                let attachment_view = if local_attachment_idx == 0 {
                    surface_view.as_ref().unwrap()
                } else {
                    &pass.attachment_views[local_attachment_idx]
                };

                match &attachment.ty {
                    PassInputType::Color(load_op) => {
                        let op = match load_op {
                            PassInputLoadOpColorType::Clear => GpuLoadOp::Clear,
                            _ => GpuLoadOp::Load,
                        };

                        let mut color_attachment = GpuRenderPassColorAttachment::new(
                            op,
                            GpuStoreOp::Store,
                            attachment_view,
                        );

                        if op == GpuLoadOp::Clear {
                            let local_attachment_id =
                                PassLocalAttachment::from_id(attachment.local_attachment_idx);
                            let clear_val = pass_submit
                                .clear_colors
                                .get(&local_attachment_id)
                                .ok_or(gpu_api_err!(
                                    "webpgpu clear color for attachment index {:?} not set",
                                    local_attachment_id
                                ))?;
                            color_attachment.clear_value(&GpuColorDict::new(
                                clear_val.a as f64,
                                clear_val.b as f64,
                                clear_val.g as f64,
                                clear_val.r as f64,
                            ));
                        }
                        color_attachments.push(&color_attachment);
                    }
                    _ => unreachable!(),
                }

                Ok(())
            })?;

            if let Some(write_depth) = step.write_depth {
                let attachment =
                    pass.original_pass
                        .attachments
                        .get(write_depth.id())
                        .ok_or(gpu_api_err!(
                            "webgpu submit write depth {:?} does not exist",
                            write_depth
                        ))?;

                let local_attachment_idx = write_depth.id();
                let mut depth_stencil_attachment = GpuRenderPassDepthStencilAttachment::new(
                    &pass.attachment_views[local_attachment_idx],
                );
                let local_attachment_id =
                    PassLocalAttachment::from_id(attachment.local_attachment_idx);
                let depth_clear_val =
                    pass_submit
                        .clear_depths
                        .get(&local_attachment_id)
                        .ok_or(gpu_api_err!(
                            "webpgpu clear depth stencil for attachment index {:?} not set",
                            local_attachment_id
                        ))?;
                match &attachment.ty {
                    PassInputType::Depth(load_op) => {
                        depth_stencil_attachment
                            .depth_store_op(GpuStoreOp::Store)
                            .depth_load_op(match load_op {
                                PassInputLoadOpDepthStencilType::Clear => GpuLoadOp::Clear,
                                _ => GpuLoadOp::Load,
                            })
                            .depth_clear_value(depth_clear_val.depth)
                            .stencil_load_op(match load_op {
                                PassInputLoadOpDepthStencilType::Clear => GpuLoadOp::Clear,
                                _ => GpuLoadOp::Load,
                            })
                            .stencil_store_op(GpuStoreOp::Store)
                            .stencil_clear_value(depth_clear_val.stencil);
                    }
                    _ => unreachable!(),
                }

                depth_attachment = Some(depth_stencil_attachment);
            }

            let mut pass_info = GpuRenderPassDescriptor::new(&color_attachments);

            if let Some(depth_attachment) = depth_attachment {
                pass_info.depth_stencil_attachment(&depth_attachment);
            }

            let pass_encoder = command_encoder.begin_render_pass(&pass_info);

            step.vertex_buffers
                .iter()
                .enumerate()
                .try_for_each(|(slot_idx, vbo)| {
                    let vbo = context.vbos.get(vbo.id()).ok_or(gpu_api_err!(
                        "webgpu submit vertex buffer id {:?} does not exist",
                        vbo
                    ))?;
                    pass_encoder.set_vertex_buffer(slot_idx as u32, &vbo.buffer);
                    Ok(())
                })?;

            if let Some(ibo) = step.index_buffer {
                let ibo = context.ibos.get(ibo.id()).ok_or(gpu_api_err!(
                    "webgpu submit index buffer id {:?} does not exist",
                    ibo
                ))?;
                assert_eq!(std::mem::size_of::<IndexBufferElement>(), 4);
                pass_encoder.set_index_buffer(&ibo.buffer, GpuIndexFormat::Uint32);
            }

            step_data.draws.iter().try_for_each(|(program_id, draw)| {
                pass_encoder.set_pipeline(pass.pipelines[step_idx].get(&program_id).ok_or(
                    gpu_api_err!("webgpu submit draw missing program id {:?}", program_id),
                )?);

                let program = context.programs.get(program_id.id()).ok_or(gpu_api_err!(
                    "webgpu submit program id {:?} does not exist",
                    program_id
                ))?;

                program
                    .bind_groups
                    .iter()
                    .enumerate()
                    .for_each(|(slot_idx, bind_group)| {
                        pass_encoder.set_bind_group(slot_idx as u32, bind_group);
                    });
                pass_encoder.set_stencil_reference(program.ext.stencil_reference.unwrap_or_default());
                match draw.ty {
                    DrawType::Draw => {
                        pass_encoder.draw_with_instance_count_and_first_vertex(
                            draw.count as u32,
                            1,
                            draw.first as u32,
                        );
                    }
                    DrawType::DrawIndexed => {
                        pass_encoder.draw_indexed_with_instance_count_and_first_index(
                            draw.count as u32,
                            1,
                            draw.first as u32,
                        );
                    }
                }

                Ok(())
            })?;

            pass_encoder.end();

            Ok(())
        })?;

    Ok(vec![command_encoder])
}
