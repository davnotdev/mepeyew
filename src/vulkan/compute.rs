use super::*;
use std::{collections::HashSet, ffi::CString};

impl VkContext {
    pub fn new_compute_program(
        &mut self,
        code: &[u8],
        uniforms: &[ShaderUniform],
        ext: Option<NewComputeProgramExt>,
    ) -> GResult<ComputeProgramId> {
        let compute_program = VkComputeProgram::new(self, code, uniforms, ext)?;
        self.compute_programs.push(compute_program);
        Ok(ComputeProgramId::from_id(self.compute_programs.len() - 1))
    }

    pub fn compile_compute_pass(
        &mut self,
        compute_pass: ComputePass,
        ext: Option<CompileComputePassExt>,
    ) -> GResult<CompiledComputePassId> {
        let compiled_compute_pass = VkCompiledComputePass::new(compute_pass, ext)?;
        self.compiled_compute_passes.push(compiled_compute_pass);
        Ok(CompiledComputePassId::from_id(
            self.compiled_compute_passes.len() - 1,
        ))
    }
}

pub struct VkComputeProgram {
    module: vk::ShaderModule,
    pub descriptors: VkDescriptors,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,

    drop_queue_ref: VkDropQueueRef,
}

impl VkComputeProgram {
    pub fn new(
        context: &VkContext,
        code: &[u8],
        uniforms: &[ShaderUniform],
        _ext: Option<NewComputeProgramExt>,
    ) -> GResult<Self> {
        let descriptors = VkDescriptors::new(context, uniforms)?;
        let layout = new_pipeline_layout(&context.core.dev, &descriptors.descriptor_set_layouts)?;
        let shader_create = vk::ShaderModuleCreateInfo::builder()
            .code(unsafe {
                std::slice::from_raw_parts(code.as_ptr() as *const u32, code.len() / (32 / 8))
            })
            .build();
        let module = unsafe { context.core.dev.create_shader_module(&shader_create, None) }
            .map_err(|e| gpu_api_err!("vulkan shader init {}", e))?;

        let entry_point_name = CString::new("main").unwrap();
        let pipeline_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .name(&entry_point_name)
            .module(module)
            .build();

        let pipeline_info = vk::ComputePipelineCreateInfo::builder()
            .stage(pipeline_stage_info)
            .layout(layout)
            .build();

        let pipeline = unsafe {
            context.core.dev.create_compute_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_info],
                None,
            )
        }
        .map_err(|e| gpu_api_err!("vulkan create compute pipeline {:?}", e))?
        .into_iter()
        .next()
        .unwrap();

        Ok(VkComputeProgram {
            descriptors,
            module,
            layout,
            pipeline,

            drop_queue_ref: Arc::clone(&context.drop_queue),
        })
    }
}

pub struct VkCompiledComputePass {
    pub set_blocking: bool,
    pub added_programs: HashSet<ComputeProgramId>,
}

impl VkCompiledComputePass {
    pub fn new(compute_pass: ComputePass, _ext: Option<CompileComputePassExt>) -> GResult<Self> {
        let added_programs = compute_pass
            .programs
            .iter()
            .cloned()
            .collect::<HashSet<_>>();

        Ok(VkCompiledComputePass {
            set_blocking: compute_pass.set_blocking,
            added_programs,
        })
    }
}

impl Drop for VkComputeProgram {
    fn drop(&mut self) {
        let module = self.module;
        let layout = self.layout;
        let pipeline = self.pipeline;

        self.drop_queue_ref
            .lock()
            .unwrap()
            .push(Box::new(move |dev, _| unsafe {
                dev.destroy_shader_module(module, None);
                dev.destroy_pipeline_layout(layout, None);
                dev.destroy_pipeline(pipeline, None);
            }));
    }
}
