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
    pub fn submit(&mut self, submit: Submit, ext: Option<SubmitExt>) -> GResult<()> {
        let ext = ext.unwrap_or_default();

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

            submit
                .dyn_ubo_transfers
                .iter()
                .for_each(|(ubo, data, index)| {
                    let ubo = self.dyn_ubos.get_mut(ubo.id()).unwrap();
                    ubo.cmd_transfer(
                        &self.core.dev.clone(),
                        graphics_command_buffer,
                        data,
                        *index,
                    )
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
                        | vk::AccessFlags::TRANSFER_READ,
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
                match pass_data {
                    SubmitPassType::Render(pass_data) => {
                        let pass = self.compiled_passes.get(pass_data.pass.id()).unwrap();

                        //  Clear Values
                        let mut clear_values = vec![
                            vk::ClearValue::default();
                            pass.attachment_count
                                + pass.resolve_image_offsets.len()
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
                                        _ => {
                                            unimplemented!("vulkan bad GpuIndexBufferElement type")
                                        }
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
                                    *pass.pipelines[step_idx].get(&draw.program).ok_or(
                                        gpu_api_err!(
                                            "vulkan submit draw missing program id {:?}",
                                            draw.program
                                        ),
                                    )?,
                                );

                                //  Descriptor Sets
                                //  TODO OPT: Maybe don't do this.
                                let program = self.programs.get(draw.program.id()).unwrap();
                                program.descriptors.cmd_bind(
                                    self,
                                    graphics_command_buffer,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    program.layout,
                                    &draw.dynamic_buffer_indices,
                                )?;

                                //  Draw
                                match draw.ty {
                                    DrawType::Draw => {
                                        self.core.dev.cmd_draw(
                                            graphics_command_buffer,
                                            draw.count as u32,
                                            draw.instance_count as u32,
                                            draw.first as u32,
                                            draw.first_instance as u32,
                                        );
                                    }
                                    DrawType::DrawIndexed => {
                                        self.core.dev.cmd_draw_indexed(
                                            graphics_command_buffer,
                                            draw.count as u32,
                                            draw.instance_count as u32,
                                            draw.first as u32,
                                            0,
                                            draw.first_instance as u32,
                                        );
                                    }
                                }
                            }

                            //  Progress
                            if step_idx != pass.steps.len() - 1 {
                                self.core.dev.cmd_next_subpass(
                                    graphics_command_buffer,
                                    vk::SubpassContents::INLINE,
                                );
                            }
                        }
                        self.core.dev.cmd_end_render_pass(graphics_command_buffer);
                    }
                    SubmitPassType::Compute(pass_data) => {
                        let compute_pass = self
                            .compiled_compute_passes
                            .get(pass_data.compute_pass.id())
                            .ok_or(gpu_api_err!(
                                "vulkan submit compute pass {:?} does not exist",
                                pass_data.compute_pass,
                            ))?;

                        unsafe fn compute_barrier(
                            dev: &Device,
                            graphics_command_buffer: vk::CommandBuffer,
                        ) {
                            let memory_barrier = vk::MemoryBarrier::builder()
                                .src_access_mask(vk::AccessFlags::SHADER_READ)
                                .dst_access_mask(vk::AccessFlags::SHADER_WRITE)
                                .build();

                            dev.cmd_pipeline_barrier(
                                graphics_command_buffer,
                                vk::PipelineStageFlags::COMPUTE_SHADER,
                                vk::PipelineStageFlags::VERTEX_SHADER,
                                vk::DependencyFlags::empty(),
                                &[memory_barrier],
                                &[],
                                &[],
                            )
                        }

                        for dispatch in pass_data.dispatches.iter() {
                            compute_pass
                                .added_programs
                                .contains(&dispatch.program)
                                .then_some(())
                                .ok_or(gpu_api_err!(
                                    "vulkan submit compute program {:?} was not added",
                                    dispatch.program
                                ))?;
                            let program = self.compute_programs.get(dispatch.program.id()).ok_or(
                                gpu_api_err!(
                                    "vulkan submit compute program {:?}",
                                    dispatch.program
                                ),
                            )?;
                            self.core.dev.cmd_bind_pipeline(
                                graphics_command_buffer,
                                vk::PipelineBindPoint::COMPUTE,
                                program.pipeline,
                            );
                            program.descriptors.cmd_bind(
                                self,
                                graphics_command_buffer,
                                vk::PipelineBindPoint::COMPUTE,
                                program.layout,
                                &dispatch.dynamic_buffer_indices,
                            )?;
                            self.core.dev.cmd_dispatch(
                                graphics_command_buffer,
                                dispatch.workgroup_count_x as u32,
                                dispatch.workgroup_count_y as u32,
                                dispatch.workgroup_count_z as u32,
                            );

                            if dispatch.ty == context::extensions::DispatchType::Blocking {
                                compute_barrier(&self.core.dev, graphics_command_buffer);
                            }
                        }

                        if compute_pass.set_blocking {
                            compute_barrier(&self.core.dev, graphics_command_buffer);
                        }
                    }
                }
            }

            //  Blit to Surface
            if let Some(blit) = submit.blit_to_surface.as_ref() {
                let src = self
                    .attachment_images
                    .get(blit.src.id())
                    .ok_or(gpu_api_err!(
                        "vulkan blit to surface src {:?} does not exist",
                        blit.src
                    ))?;
                let Some(surface) = self.surface_ext.as_ref() else {
                    Err(gpu_api_err!("vulkan blit_to_surface without surface"))?
                };
                let dst = surface.swapchain.swapchain_images[swapchain_image_index as usize];

                let src_x = blit.src_x.unwrap_or(0);
                let src_y = blit.src_x.unwrap_or(0);
                let dst_x = blit.src_x.unwrap_or(0);
                let dst_y = blit.src_x.unwrap_or(0);

                let src_width = blit.src_width.unwrap_or(src.image.extent.width as usize);
                let src_height = blit.src_height.unwrap_or(src.image.extent.height as usize);
                let dst_width = blit
                    .dst_width
                    .unwrap_or(surface.swapchain.extent.width as usize);
                let dst_height = blit
                    .dst_height
                    .unwrap_or(surface.swapchain.extent.height as usize);

                let range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .layer_count(1)
                    .build();

                let src_image_transfer_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .image(src.image.image)
                    .subresource_range(range)
                    .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .build();

                let dst_image_transfer_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .image(dst)
                    .subresource_range(range)
                    .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .build();

                self.core.dev.cmd_pipeline_barrier(
                    graphics_command_buffer,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[src_image_transfer_barrier, dst_image_transfer_barrier],
                );

                let subresource_range_layers = vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .layer_count(1)
                    .base_array_layer(0)
                    .build();
                let region = vk::ImageBlit::builder()
                    .src_offsets([
                        vk::Offset3D::builder()
                            .x(src_x as i32)
                            .y(src_y as i32)
                            .z(0)
                            .build(),
                        vk::Offset3D::builder()
                            .x((src_x + src_width) as i32)
                            .y((src_y + src_height) as i32)
                            .z(1)
                            .build(),
                    ])
                    .dst_offsets([
                        vk::Offset3D::builder()
                            .x(dst_x as i32)
                            .y(dst_y as i32)
                            .z(0)
                            .build(),
                        vk::Offset3D::builder()
                            .x((dst_x + dst_width) as i32)
                            .y((dst_y + dst_height) as i32)
                            .z(1)
                            .build(),
                    ])
                    .src_subresource(subresource_range_layers)
                    .dst_subresource(subresource_range_layers)
                    .build();

                self.core.dev.cmd_blit_image(
                    graphics_command_buffer,
                    src.image.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    dst,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[region],
                    filter_into_vk(blit.filter),
                );

                let src_image_transfer_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                    .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .image(src.image.image)
                    .subresource_range(range)
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::empty())
                    .build();

                let dst_image_transfer_barrier = vk::ImageMemoryBarrier::builder()
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .image(dst)
                    .subresource_range(range)
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::empty())
                    .build();

                self.core.dev.cmd_pipeline_barrier(
                    graphics_command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[src_image_transfer_barrier, dst_image_transfer_barrier],
                );
            }

            //  SSBO Copy Backs
            submit.ssbo_copy_backs.iter().try_for_each(|ssbo_id| {
                let ssbo = self.ssbos.get(ssbo_id.id()).ok_or(gpu_api_err!(
                    "vulkan shader storage buffer sync id {:?} does not exist",
                    ssbo_id
                ))?;

                let barrier = vk::BufferMemoryBarrier::builder()
                    .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .buffer(ssbo.buffer.buffer)
                    .size(vk::WHOLE_SIZE)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .build();

                self.core.dev.cmd_pipeline_barrier(
                    graphics_command_buffer,
                    vk::PipelineStageFlags::VERTEX_SHADER
                        | vk::PipelineStageFlags::FRAGMENT_SHADER
                        | vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[barrier],
                    &[],
                );

                let copy_region = vk::BufferCopy::builder()
                    .size(ssbo.buffer.size as u64)
                    .build();

                self.core.dev.cmd_copy_buffer(
                    graphics_command_buffer,
                    ssbo.buffer.buffer,
                    ssbo.staging.as_ref().unwrap().buffer,
                    &[copy_region],
                );

                let barrier = vk::BufferMemoryBarrier::builder()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::HOST_READ)
                    .buffer(ssbo.staging.as_ref().unwrap().buffer)
                    .size(vk::WHOLE_SIZE)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .build();

                self.core.dev.cmd_pipeline_barrier(
                    graphics_command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::HOST,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[barrier],
                    &[],
                );

                Ok(())
            })?;

            self.core
                .dev
                .end_command_buffer(graphics_command_buffer)
                .unwrap();

            let mut submit_signal_semaphores = vec![];
            let mut submit_wait_semaphores = vec![];

            let should_present = submit.passes.iter().any(|pass| {
                match pass {
                    SubmitPassType::Render(pass) => {
                        let pass = &self.compiled_passes[pass.pass.id()];
                        pass.should_present
                    }
                    _ => false,

                }
            }) || submit.blit_to_surface.is_some() /* && self.surface_ext.is_some() */;
            //  Purposely don't check for surface_ext.

            if should_present {
                submit_signal_semaphores.push(render_semaphore);
                submit_wait_semaphores.push(image_aquire_semaphore);
            }
            let submit_create = vk::SubmitInfo::builder()
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .wait_semaphores(&submit_wait_semaphores)
                .signal_semaphores(&submit_signal_semaphores)
                .command_buffers(&[graphics_command_buffer])
                .build();

            self.core
                .dev
                .queue_submit(self.core.graphics_queue, &[submit_create], frame_fence)
                .unwrap();

            if ext.sync.is_some() {
                self.core
                    .dev
                    .wait_for_fences(&[frame_fence], true, std::u64::MAX)
                    .unwrap();
            }

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
