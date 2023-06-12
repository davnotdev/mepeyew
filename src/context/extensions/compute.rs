use super::*;
use std::collections::HashMap;

//  TODO docs ALL OF THIS!

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComputePass {
    pub(crate) programs: Vec<ComputeProgramId>,
    pub(crate) set_blocking: bool,
}

impl ComputePass {
    pub fn new() -> Self {
        ComputePass::default()
    }

    pub fn add_program(&mut self, program: ComputeProgramId) -> &mut Self {
        self.programs.push(program);
        self
    }

    //  TODO doc no wait by default
    pub fn set_blocking(&mut self, set: bool) -> &mut Self {
        self.set_blocking = set;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispatch {
    pub(crate) ty: DispatchType,
    pub(crate) program: ComputeProgramId,

    pub(crate) workgroup_count_x: usize,
    pub(crate) workgroup_count_y: usize,
    pub(crate) workgroup_count_z: usize,

    pub(crate) dynamic_buffer_indices: HashMap<DynamicGenericBufferId, usize>,
}

impl Dispatch {
    pub fn set_dynamic_uniform_buffer_index(
        &mut self,
        ubo: DynamicUniformBufferId,
        index: usize,
    ) -> &mut Self {
        self.dynamic_buffer_indices
            .insert(DynamicGenericBufferId::Uniform(ubo), index);
        self
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DispatchType {
    Blocking,
    NonBlocking,
}

#[derive(Debug, Clone)]
pub struct ComputePassSubmitData {
    pub(crate) compute_pass: CompiledComputePassId,
    pub(crate) dispatches: Vec<Dispatch>,
}

impl ComputePassSubmitData {
    pub fn new(compute_pass: CompiledComputePassId) -> Self {
        ComputePassSubmitData {
            compute_pass,
            dispatches: vec![],
        }
    }

    pub fn dispatch(
        &mut self,
        compute_program: ComputeProgramId,
        workgroup_count_x: usize,
        workgroup_count_y: usize,
        workgroup_count_z: usize,
    ) -> &mut Dispatch {
        self.dispatches.push(Dispatch {
            ty: DispatchType::NonBlocking,
            program: compute_program,
            workgroup_count_x,
            workgroup_count_y,
            workgroup_count_z,
            dynamic_buffer_indices: HashMap::new(),
        });
        self.dispatches.last_mut().unwrap()
    }

    pub fn dispatch_blocking(
        &mut self,
        compute_program: ComputeProgramId,
        workgroup_count_x: usize,
        workgroup_count_y: usize,
        workgroup_count_z: usize,
    ) -> &mut Dispatch {
        self.dispatches.push(Dispatch {
            ty: DispatchType::Blocking,
            program: compute_program,
            workgroup_count_x,
            workgroup_count_y,
            workgroup_count_z,
            dynamic_buffer_indices: HashMap::new(),
        });
        self.dispatches.last_mut().unwrap()
    }
}

#[derive(Default, Debug, Clone)]
pub struct NewComputeProgramExt {}
#[derive(Default, Debug, Clone)]
pub struct CompileComputePassExt {}

impl Context {
    pub fn new_compute_program(
        &mut self,
        code: &[u8],
        uniforms: &[ShaderUniform],
        ext: Option<NewComputeProgramExt>,
    ) -> GResult<ComputeProgramId> {
        match self {
            Context::Vulkan(vk) => vk.new_compute_program(code, uniforms, ext),
            Context::WebGpu(wgpu) => wgpu.new_compute_program(code, uniforms, ext),
        }
    }

    pub fn compile_compute_pass(
        &mut self,
        compute_pass: ComputePass,
        ext: Option<CompileComputePassExt>,
    ) -> GResult<CompiledComputePassId> {
        match self {
            Context::Vulkan(vk) => vk.compile_compute_pass(compute_pass, ext),
            Context::WebGpu(wgpu) => wgpu.compile_compute_pass(compute_pass, ext),
        }
    }
}
