use super::*;

impl WebGpuContext {
    pub fn new_compute_program(
        &mut self,
        code: &[u8],
        uniforms: &[ShaderUniform],
        ext: Option<NewComputeProgramExt>,
    ) -> GResult<ComputeProgramId> {
        let compute_program = WebGpuComputeProgram::new(self, code, uniforms, ext)?;
        self.compute_programs.push(compute_program);
        Ok(ComputeProgramId::from_id(self.compute_programs.len() - 1))
    }

    pub fn compile_compute_pass(
        &mut self,
        compute_pass: ComputePass,
        ext: Option<CompileComputePassExt>,
    ) -> GResult<CompiledComputePassId> {
        let compiled_compute_pass = WebGpuCompiledComputePass::new(compute_pass, ext);
        self.compiled_compute_passes.push(compiled_compute_pass);
        Ok(CompiledComputePassId::from_id(
            self.compiled_compute_passes.len() - 1,
        ))
    }
}

pub struct WebGpuComputeProgram {
    pub pipeline: GpuComputePipeline,
    pub bind_groups: WebGpuBindGroups,
}

impl WebGpuComputeProgram {
    pub fn new(
        context: &WebGpuContext,
        code: &[u8],
        uniforms: &[ShaderUniform],
        _ext: Option<NewComputeProgramExt>,
    ) -> GResult<Self> {
        let module = context
            .device
            .create_shader_module(&GpuShaderModuleDescriptor::new(
                std::str::from_utf8(code).unwrap(),
            ));

        let bind_groups = WebGpuBindGroups::new(context, uniforms)?;

        let mut layout = JsValue::from_str("auto");
        if !bind_groups.bind_group_layouts.is_empty() {
            let layouts = Array::new();
            bind_groups.bind_group_layouts.iter().for_each(|layout| {
                layouts.push(layout);
            });

            let layout_info = GpuPipelineLayoutDescriptor::new(&layouts);
            let pipeline_layout = context.device.create_pipeline_layout(&layout_info);
            layout = pipeline_layout.into();
        }

        let pipeline_programmable_stage = GpuProgrammableStage::new("main", &module);

        let pipeline = context
            .device
            .create_compute_pipeline(&GpuComputePipelineDescriptor::new(
                &layout,
                &pipeline_programmable_stage,
            ));

        Ok(WebGpuComputeProgram {
            pipeline,
            bind_groups,
        })
    }
}

pub struct WebGpuCompiledComputePass {
    pub added_programs: HashSet<ComputeProgramId>,
}

impl WebGpuCompiledComputePass {
    pub fn new(compute_pass: ComputePass, _ext: Option<CompileComputePassExt>) -> Self {
        WebGpuCompiledComputePass {
            added_programs: compute_pass.programs.iter().cloned().collect(),
        }
    }
}
