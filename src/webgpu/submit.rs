use super::*;

impl WebGpuContext {
    pub fn submit(&mut self, submit: Submit, _ext: Option<SubmitExt>) -> GResult<()> {
        submit_transfers(self, &submit);

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
        .try_for_each(|(step, step_data)| {
            let color_attachments = Array::new();

            if let Some(surface) = &context.surface {
                let surface_texture_view = surface.context.get_current_texture().create_view();
                let attachment = GpuRenderPassColorAttachment::new(
                    GpuLoadOp::Load,
                    GpuStoreOp::Store,
                    &surface_texture_view,
                );
                color_attachments.push(&attachment);
            }

            let pass_info = GpuRenderPassDescriptor::new(&color_attachments);
            let pass_encoder = command_encoder.begin_render_pass(&pass_info);
            pass_encoder.set_pipeline(&pass.pipelines[0]);

            if let Some(program) = step.program {
                let program = context.programs.get(program.id()).ok_or(gpu_api_err!(
                    "webgpu submit program id {:?} does not exist",
                    step.program
                ))?;

                program
                    .bind_groups
                    .iter()
                    .enumerate()
                    .for_each(|(slot_idx, bind_group)| {
                        pass_encoder.set_bind_group(slot_idx as u32, bind_group);
                    })
            }

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

            step_data.draws.iter().for_each(|draw| {
                pass_encoder.draw_with_instance_count_and_first_vertex(
                    draw.count as u32,
                    1,
                    draw.first as u32,
                );
            });

            step_data.draws_indexed.iter().for_each(|draw| {
                pass_encoder.draw_indexed_with_instance_count_and_first_index(
                    draw.count as u32,
                    1,
                    draw.first as u32,
                );
            });

            pass_encoder.end();

            Ok(())
        })?;

    Ok(vec![command_encoder])
}
