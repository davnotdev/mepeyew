use super::*;
use context::extensions::*;
use std::collections::HashSet;

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

        let bind_groups = WebGpuBindGroups::new(context, uniforms, true)?;

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

pub fn submit_compute_pass(
    context: &WebGpuContext,
    pass_submit: &ComputePassSubmitData,
    command_encoder: &GpuCommandEncoder,
) -> GResult<()> {
    let compute_pass = context
        .compiled_compute_passes
        .get(pass_submit.compute_pass.id())
        .ok_or(gpu_api_err!(
            "webgpu submit compute pass {:?} does not exist",
            pass_submit.compute_pass,
        ))?;

    let pass_encoder = command_encoder.begin_compute_pass();

    for dispatch in pass_submit.dispatches.iter() {
        compute_pass
            .added_programs
            .contains(&dispatch.program)
            .then_some(())
            .ok_or(gpu_api_err!(
                "webgpu submit compute program {:?} was not added",
                dispatch.program
            ))?;

        let program = context
            .compute_programs
            .get(dispatch.program.id())
            .ok_or(gpu_api_err!(
                "webgpu submit compute program {:?}",
                dispatch.program
            ))?;

        pass_encoder.set_pipeline(&program.pipeline);
        program.bind_groups.cmd_compute_bind_groups(
            context,
            &pass_encoder,
            &dispatch.dynamic_buffer_indices,
        )?;
        pass_encoder.dispatch_workgroups_with_workgroup_count_y_and_workgroup_count_z(
            dispatch.workgroup_count_x as u32,
            dispatch.workgroup_count_y as u32,
            dispatch.workgroup_count_z as u32,
        );
    }

    pass_encoder.end();

    Ok(())
}
