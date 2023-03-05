use super::*;
use std::ffi::CString;

impl VkContext {
    pub fn new_program(&mut self, shaders: &ShaderSet) -> GResult<ProgramId> {
        let shaders = shaders
            .0
            .iter()
            .map(|(ty, src)| {
                let shader_ty = match ty {
                    ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
                    ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
                };
                VkShader::new(&self.core.dev, &self.drop_queue, shader_ty, src)
            })
            .collect::<GResult<Vec<_>>>()?;

        let program = VkProgram {
            layout: new_pipeline_layout(&self.core.dev)?,
            shaders,
            drop_queue: Arc::clone(&self.drop_queue),
        };
        self.programs.push(program);

        Ok(ProgramId::from_id(self.programs.len() - 1))
    }
}

fn new_pipeline_layout(dev: &Device) -> GResult<vk::PipelineLayout> {
    let pipeline_layout_create = vk::PipelineLayoutCreateInfo::builder().build();
    unsafe { dev.create_pipeline_layout(&pipeline_layout_create, None) }
        .map_err(|e| gpu_api_err!("vulkan pipeline layout {}", e))
}

pub struct VkProgram {
    layout: vk::PipelineLayout,
    shaders: Vec<VkShader>,

    drop_queue: VkDropQueueRef,
}

impl VkProgram {
    //  TODO OPT: Find seemless and safe way to generate pipelines in one go.
    pub fn new_graphics_pipeline(
        &self,
        dev: &Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        subpass: usize,
    ) -> GResult<vk::Pipeline> {
        //  Vertex Input State Info
        let (attributes, bindings) = self
            .shaders
            .iter()
            .find(|shader| shader.shader_stage == vk::ShaderStageFlags::VERTEX)
            .map(|vert_shader| vert_shader.get_inputs())
            .unwrap_or_else(|| (vec![], vk::VertexInputBindingDescription::default()));
        let vertex_input_state_create = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&[bindings])
            .vertex_attribute_descriptions(&attributes)
            .build();

        //  Input Assembly Info
        let input_assembly_create = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        //  Rasterization Info
        let raster_create = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_bias_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .build();

        //  Multisample Info
        let multisample_create = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        //  Color Blend Info
        let color_blend_state = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .build();

        let color_blend_create = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[color_blend_state])
            .build();

        //  Viewport State
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }])
            .scissors(&[vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            }])
            .build();

        //  Depth Stencil
        let depth_create = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .build();

        //  Shader Stage Info
        let entry_point = CString::new("main").unwrap();

        let shader_stage_creates = self
            .shaders
            .iter()
            .map(|shader| {
                vk::PipelineShaderStageCreateInfo::builder()
                    .name(&entry_point)
                    .stage(shader.shader_stage)
                    .module(shader.module)
                    .build()
            })
            .collect::<Vec<_>>();

        //  Graphics Pipeline
        let graphics_pipeline_create = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_creates)
            .vertex_input_state(&vertex_input_state_create)
            .input_assembly_state(&input_assembly_create)
            .viewport_state(&viewport_state)
            .rasterization_state(&raster_create)
            .multisample_state(&multisample_create)
            .color_blend_state(&color_blend_create)
            .layout(self.layout)
            .depth_stencil_state(&depth_create)
            .render_pass(render_pass)
            .subpass(subpass as u32)
            .build();

        unsafe {
            dev.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphics_pipeline_create],
                None,
            )
        }
        .map(|pipelines| pipelines.into_iter().next().unwrap())
        .map_err(|(_, e)| gpu_api_err!("vulkan graphics pipelines {}", e))
    }
}

impl Drop for VkProgram {
    fn drop(&mut self) {
        let pipeline_layout = self.layout;

        self.drop_queue
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_pipeline_layout(pipeline_layout, None);
            }))
    }
}
