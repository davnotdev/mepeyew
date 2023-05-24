use super::*;
use std::mem::ManuallyDrop;

pub struct VkSubmitData {
    frame_fence: ManuallyDrop<VkFrameDependent<vk::Fence>>,
    render_semaphore: ManuallyDrop<VkFrameDependent<vk::Semaphore>>,
    image_aquire_semaphore: ManuallyDrop<VkFrameDependent<vk::Semaphore>>,

    graphics_command_buffer: VkFrameDependent<vk::CommandBuffer>,

    drop_queue_ref: VkDropQueueRef,
}

impl VkSubmitData {
    pub fn new(
        dev: &Device,
        frame: &VkFrame,
        graphics_command_pool: vk::CommandPool,
        drop_queue_ref: &VkDropQueueRef,
    ) -> GResult<Self> {
        let frame_fence = ManuallyDrop::new(VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| new_fence(dev, true))
                .collect::<GResult<Vec<_>>>()?,
        ));
        let render_semaphore = ManuallyDrop::new(VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| new_semaphore(dev))
                .collect::<GResult<Vec<_>>>()?,
        ));
        let image_aquire_semaphore = ManuallyDrop::new(VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| new_semaphore(dev))
                .collect::<GResult<Vec<_>>>()?,
        ));
        let graphics_command_buffer = VkFrameDependent::from_iter(
            (0..frame.get_flight_frames_count())
                .map(|_| {
                    let command_buffer_alloc = vk::CommandBufferAllocateInfo::builder()
                        .command_pool(graphics_command_pool)
                        .command_buffer_count(1)
                        .build();
                    Ok(unsafe {
                        dev.allocate_command_buffers(&command_buffer_alloc)
                            .map_err(|e| gpu_api_err!("vulkan submit new command buffer {}", e))?[0]
                    })
                })
                .collect::<GResult<Vec<_>>>()?,
        );
        Ok(VkSubmitData {
            frame_fence,
            render_semaphore,
            image_aquire_semaphore,
            graphics_command_buffer,
            drop_queue_ref: Arc::clone(drop_queue_ref),
        })
    }
}

impl Drop for VkSubmitData {
    fn drop(&mut self) {
        let frame_fence = unsafe { ManuallyDrop::take(&mut self.frame_fence).take_all() };
        let render_semaphore = unsafe { ManuallyDrop::take(&mut self.render_semaphore).take_all() };
        let image_aquire_semaphore =
            unsafe { ManuallyDrop::take(&mut self.image_aquire_semaphore).take_all() };

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                frame_fence
                    .into_iter()
                    .for_each(|fence| dev.destroy_fence(fence, None));
                render_semaphore
                    .into_iter()
                    .for_each(|semaphore| dev.destroy_semaphore(semaphore, None));
                image_aquire_semaphore
                    .into_iter()
                    .for_each(|semaphore| dev.destroy_semaphore(semaphore, None));
            }));
    }
}

impl VkContext {
    pub fn submit(&mut self, submit: Submit, _ext: Option<SubmitExt>) -> GResult<()> {
        let frame_fence = *self.submit.frame_fence.get(&self.frame);
        let render_semaphore = *self.submit.render_semaphore.get(&self.frame);
        let image_aquire_semaphore = *self.submit.image_aquire_semaphore.get(&self.frame);
        let graphics_command_buffer = *self.submit.graphics_command_buffer.get(&self.frame);

        //  TODO FIX: Replace unwraps.
        unsafe {
            let (swapchain_image_index, _suboptimal) = if let Some(surface) = &*self.surface_ext {
                match surface.swapchain.swapchain_ext.acquire_next_image(
                    surface.swapchain.swapchain,
                    std::u64::MAX,
                    image_aquire_semaphore,
                    vk::Fence::null(),
                ) {
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        self.surface_extension_set_surface_size(
                            surface.swapchain.extent.width as usize,
                            surface.swapchain.extent.height as usize,
                        )?;
                        return Ok(());
                    }
                    Err(e) => Err(gpu_api_err!("vulkan aquire image {}", e))?,
                    Ok(ret) => ret,
                }
            } else {
                (0, false)
            };

            self.core
                .dev
                .wait_for_fences(&[frame_fence], true, std::u64::MAX)
                .unwrap();
            self.core.dev.reset_fences(&[frame_fence]).unwrap();

            self.core
                .dev
                .reset_command_buffer(
                    graphics_command_buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .unwrap();

            let command_create = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            self.core
                .dev
                .begin_command_buffer(graphics_command_buffer, &command_create)
                .unwrap();

            //  Graphics Related Transfers
            submit.vbo_transfers.iter().for_each(|(vbo, data)| {
                let vbo = self.vbos.get_mut(vbo.id()).unwrap();
                vbo.cmd_transfer(&self.core.dev.clone(), graphics_command_buffer, data)
                    .unwrap();
            });

            submit.ibo_transfers.iter().for_each(|(ibo, data)| {
                let ibo = self.ibos.get_mut(ibo.id()).unwrap();
                ibo.cmd_transfer(&self.core.dev.clone(), graphics_command_buffer, data)
                    .unwrap();
            });

            submit.ubo_transfers.iter().for_each(|(ubo, data)| {
                let ubo = self.ubos.get_mut(ubo.id()).unwrap();
                ubo.cmd_transfer(&self.core.dev.clone(), graphics_command_buffer, data)
                    .unwrap();
            });

            //  Read somewhere that this is actually unneccessary.
            let graphics_memory_barrier = vk::MemoryBarrier::builder()
                .src_access_mask(vk::AccessFlags::HOST_WRITE)
                .dst_access_mask(
                    vk::AccessFlags::INDEX_READ
                        | vk::AccessFlags::VERTEX_ATTRIBUTE_READ
                        | vk::AccessFlags::UNIFORM_READ
                        | vk::AccessFlags::SHADER_READ
                        | vk::AccessFlags::SHADER_WRITE
                        | vk::AccessFlags::TRANSFER_READ
                        | vk::AccessFlags::TRANSFER_WRITE,
                )
                .build();
            self.core.dev.cmd_pipeline_barrier(
                graphics_command_buffer,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[graphics_memory_barrier],
                &[],
                &[],
            );

            //  Render.
            for pass_data in submit.passes.iter() {
                let pass = self.compiled_passes.get(pass_data.pass.id()).unwrap();

                //  Clear Values
                let mut clear_values = vec![
                    vk::ClearValue::default();
                    pass.attachment_count + pass.resolve_image_offsets.len()
                ];

                for (&attachment, clear) in pass_data.clear_colors.iter() {
                    let resolved_idx = if pass.resolve_image_offsets.is_empty() {
                        attachment.id()
                    } else {
                        pass.attachment_count + pass.resolve_image_offsets[&attachment.id()]
                    };

                    clear_values[resolved_idx] = vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [clear.r, clear.g, clear.b, clear.a],
                        },
                    };
                }

                for (&attachment, clear) in pass_data.clear_depths.iter() {
                    clear_values[attachment.id()] = vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: clear.depth,
                            stencil: clear.stencil,
                        },
                    };
                }

                //  Rendering Time!

                let render_pass_begin = vk::RenderPassBeginInfo::builder()
                    .render_pass(pass.render_pass)
                    .clear_values(&clear_values)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: pass.render_extent,
                    })
                    .framebuffer(
                        pass.framebuffer
                            .get_current_framebuffer(swapchain_image_index),
                    )
                    .build();
                self.core.dev.cmd_begin_render_pass(
                    graphics_command_buffer,
                    &render_pass_begin,
                    vk::SubpassContents::INLINE,
                );
                for (step_idx, (step, step_data)) in pass
                    .steps
                    .iter()
                    .zip(pass_data.steps_datas.iter())
                    .enumerate()
                {
                    //  Index Buffer
                    if let Some(ibo) = step.index_buffer {
                        let ibo = self.ibos.get(ibo.id()).unwrap();
                        self.core.dev.cmd_bind_index_buffer(
                            graphics_command_buffer,
                            ibo.buffer.buffer,
                            0,
                            match std::mem::size_of::<IndexBufferElement>() {
                                4 => vk::IndexType::UINT32,
                                2 => vk::IndexType::UINT16,
                                _ => unimplemented!("vulkan bad GpuIndexBufferElement type"),
                            },
                        )
                    }

                    //  Vertex Buffers
                    let vbo_buffers = step
                        .vertex_buffers
                        .iter()
                        .map(|vbo| {
                            Ok(self
                                .vbos
                                .get(vbo.id())
                                .ok_or(gpu_api_err!("vulkan bad vbo ({})", vbo.id()))?
                                .buffer
                                .buffer)
                        })
                        .collect::<GResult<Vec<_>>>()?;
                    let vbo_offsets = (0..step.vertex_buffers.len())
                        .map(|_| 0)
                        .collect::<Vec<_>>();
                    self.core.dev.cmd_bind_vertex_buffers(
                        graphics_command_buffer,
                        0,
                        &vbo_buffers,
                        &vbo_offsets,
                    );

                    //  Draw
                    for draw in step_data.draws.iter() {
                        //  Dynamic Viewport / Scissor
                        let viewport = draw.viewport.unwrap_or(DrawViewport {
                            x: 0.0,
                            y: 0.0,
                            width: pass.original_pass.render_width as f32,
                            height: pass.original_pass.render_height as f32,
                        });
                        let scissor = draw.scissor.unwrap_or(DrawScissor {
                            x: 0,
                            y: 0,
                            width: pass.original_pass.render_width,
                            height: pass.original_pass.render_height,
                        });

                        self.core.dev.cmd_set_viewport(
                            graphics_command_buffer,
                            0,
                            &[vk::Viewport {
                                x: viewport.x,
                                y: viewport.y,
                                width: viewport.width,
                                height: viewport.height,
                                min_depth: 0.0,
                                max_depth: 1.0,
                            }],
                        );

                        self.core.dev.cmd_set_scissor(
                            graphics_command_buffer,
                            0,
                            &[vk::Rect2D::builder()
                                .offset(vk::Offset2D {
                                    x: scissor.x as i32,
                                    y: scissor.y as i32,
                                })
                                .extent(vk::Extent2D {
                                    width: scissor.width as u32,
                                    height: scissor.height as u32,
                                })
                                .build()],
                        );

                        //  Program
                        self.core.dev.cmd_bind_pipeline(
                            graphics_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            *pass.pipelines[step_idx]
                                .get(&draw.program)
                                .ok_or(gpu_api_err!(
                                    "vulkan submit draw missing program id {:?}",
                                    draw.program
                                ))?,
                        );

                        //  Descriptor Sets
                        //  TODO OPT: Maybe don't do this.
                        let program = self.programs.get(draw.program.id()).unwrap();
                        self.core.dev.cmd_bind_descriptor_sets(
                            graphics_command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            program.layout,
                            0,
                            &program.descriptors.descriptor_sets,
                            &[],
                        );

                        //  Draw
                        match draw.ty {
                            DrawType::Draw => {
                                self.core.dev.cmd_draw(
                                    graphics_command_buffer,
                                    draw.count as u32,
                                    1,
                                    draw.first as u32,
                                    0,
                                );
                            }
                            DrawType::DrawIndexed => {
                                self.core.dev.cmd_draw_indexed(
                                    graphics_command_buffer,
                                    draw.count as u32,
                                    1,
                                    draw.first as u32,
                                    0,
                                    0,
                                );
                            }
                        }
                    }

                    //  Progress
                    if step_idx != pass.steps.len() - 1 {
                        self.core
                            .dev
                            .cmd_next_subpass(graphics_command_buffer, vk::SubpassContents::INLINE);
                    }
                }
                self.core.dev.cmd_end_render_pass(graphics_command_buffer);
            }

            self.core
                .dev
                .end_command_buffer(graphics_command_buffer)
                .unwrap();

            let mut submit_signal_semaphores = vec![];

            let should_present = submit.passes.iter().any(|pass| {
                let pass = &self.compiled_passes[pass.pass.id()];
                pass.should_present
            }) /* && self.surface_ext.is_some() */;
            //  Purposely don't check for surface_ext.

            if should_present {
                submit_signal_semaphores.push(render_semaphore);
            }
            let submit_create = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .wait_semaphores(&[image_aquire_semaphore])
                .signal_semaphores(&submit_signal_semaphores)
                .command_buffers(&[graphics_command_buffer])
                .build();

            self.core
                .dev
                .queue_submit(self.core.graphics_queue, &[submit_create], frame_fence)
                .unwrap();

            self.frame.advance_frame();

            if should_present {
                if let Some(surface) = &*self.surface_ext {
                    let present_create = vk::PresentInfoKHR::builder()
                        .wait_semaphores(&[render_semaphore])
                        .swapchains(&[surface.swapchain.swapchain])
                        .image_indices(&[swapchain_image_index])
                        .build();

                    match surface
                        .swapchain
                        .swapchain_ext
                        .queue_present(self.core.graphics_queue, &present_create)
                    {
                        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                            self.surface_extension_set_surface_size(
                                surface.swapchain.extent.width as usize,
                                surface.swapchain.extent.height as usize,
                            )?;
                        }
                        Err(e) => Err(gpu_api_err!("vulkan queue present {}", e))?,
                        _ => {}
                    };
                } else {
                    Err(gpu_api_err!(
                        "vulkan tried to render to surface without surface extension"
                    ))?;
                }
            }
        }

        Ok(())
    }
}
