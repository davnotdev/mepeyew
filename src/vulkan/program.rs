use super::*;
use std::ffi::CString;

impl VkContext {
    pub fn new_program(
        &mut self,
        shaders: &ShaderSet,
        uniforms: &[ShaderUniform],
        ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        let shaders = shaders
            .0
            .iter()
            .map(|(ty, src)| VkShader::new(&self.core.dev, &self.drop_queue, ty, src))
            .collect::<GResult<Vec<_>>>()?;

        let descriptors = VkDescriptors::new(self, uniforms)?;

        let program = VkProgram {
            layout: new_pipeline_layout(&self.core.dev, &descriptors.descriptor_set_layouts)?,
            shaders,
            descriptors,
            ext: ext.unwrap_or_default(),
            drop_queue: Arc::clone(&self.drop_queue),
        };
        self.programs.push(program);

        Ok(ProgramId::from_id(self.programs.len() - 1))
    }
}

fn new_pipeline_layout(
    dev: &Device,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> GResult<vk::PipelineLayout> {
    let pipeline_layout_create = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(descriptor_set_layouts)
        .build();
    unsafe { dev.create_pipeline_layout(&pipeline_layout_create, None) }
        .map_err(|e| gpu_api_err!("vulkan pipeline layout {}", e))
}

pub struct VkProgram {
    pub descriptors: VkDescriptors,
    pub layout: vk::PipelineLayout,
    pub ext: NewProgramExt,
    shaders: Vec<VkShader>,

    drop_queue: VkDropQueueRef,
}

impl VkProgram {
    //  TODO OPT: Find seemless and safe way to generate pipelines in one go.
    pub fn new_graphics_pipeline(
        &self,
        dev: &Device,
        render_pass: vk::RenderPass,
        subpass: usize,
        sample_count: Option<vk::SampleCountFlags>,
        ext: &NewProgramExt,
    ) -> GResult<vk::Pipeline> {
        //  Vertex Input State Info
        let (attributes, bindings) = self
            .shaders
            .iter()
            .find_map(|shader| {
                if let ShaderType::Vertex(vertex_inputs) = &shader.shader_ty {
                    Some(VkShader::get_vertex_inputs(vertex_inputs))
                } else {
                    None
                }
            })
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
            .rasterization_samples(sample_count.unwrap_or(vk::SampleCountFlags::TYPE_1))
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
            .viewports(&[])
            .scissors(&[])
            .viewport_count(1)
            .scissor_count(1)
            .build();
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
            .build();

        //  Depth Stencil
        fn stencil_op_into_vk(op: ShaderStencilOp) -> vk::StencilOp {
            match op {
                ShaderStencilOp::Keep => vk::StencilOp::KEEP,
                ShaderStencilOp::Zero => vk::StencilOp::ZERO,
                ShaderStencilOp::Replace => vk::StencilOp::REPLACE,
                ShaderStencilOp::IncrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
                ShaderStencilOp::DecrementClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
                ShaderStencilOp::Invert => vk::StencilOp::INVERT,
                ShaderStencilOp::IncrementWrap => vk::StencilOp::INCREMENT_AND_WRAP,
                ShaderStencilOp::DecrementWrap => vk::StencilOp::DECREMENT_AND_WRAP,
            }
        }

        fn compare_op_into_vk(op: ShaderCompareOp) -> vk::CompareOp {
            match op {
                ShaderCompareOp::Never => vk::CompareOp::NEVER,
                ShaderCompareOp::Less => vk::CompareOp::LESS,
                ShaderCompareOp::Equal => vk::CompareOp::EQUAL,
                ShaderCompareOp::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
                ShaderCompareOp::Greater => vk::CompareOp::GREATER,
                ShaderCompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
                ShaderCompareOp::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
                ShaderCompareOp::Always => vk::CompareOp::ALWAYS,
            }
        }

        let stencil_op_state = vk::StencilOpState::builder()
            .compare_op(compare_op_into_vk(
                ext.stencil_compare_op.unwrap_or_default(),
            ))
            .fail_op(stencil_op_into_vk(ext.stencil_fail.unwrap_or_default()))
            .pass_op(stencil_op_into_vk(ext.stencil_pass.unwrap_or_default()))
            .depth_fail_op(stencil_op_into_vk(
                ext.stencil_depth_fail.unwrap_or_default(),
            ))
            .reference(ext.stencil_reference.unwrap_or_default())
            .compare_mask(ext.stencil_compare_mask.unwrap_or_default())
            .write_mask(ext.stencil_write_mask.unwrap_or_default())
            .build();

        let depth_create = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(ext.enable_depth_test.is_some())
            .depth_write_enable(ext.enable_depth_test.is_some())
            .depth_compare_op(compare_op_into_vk(ext.depth_compare_op.unwrap_or_default()))
            .depth_bounds_test_enable(false)
            .stencil_test_enable(ext.enable_stencil_test.is_some())
            .front(stencil_op_state)
            .back(stencil_op_state)
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
                    .stage(match shader.shader_ty {
                        ShaderType::Vertex(_) => vk::ShaderStageFlags::VERTEX,
                        ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
                    })
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
            .dynamic_state(&dynamic_state)
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
