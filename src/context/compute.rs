use super::*;

//  TODO docs ALL OF THIS!

#[derive(Default, Clone)]
pub struct ComputePass {
    pub(crate) programs: Vec<ComputeProgramId>,
    pub(crate) set_blocking: bool,
}

impl ComputePass {
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

#[derive(Debug, Clone, Copy)]
pub struct Dispatch {
    pub workgroup_count_x: usize,
    pub workgroup_count_y: usize,
    pub workgroup_count_z: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchType {
    Blocking,
    NonBlocking,
}

#[derive(Debug, Clone)]
pub struct ComputePassSubmitData {
    pub(crate) compute_pass: CompiledComputePassId,
    pub(crate) dispatches: Vec<(ComputeProgramId, Dispatch, DispatchType)>,
}

impl ComputePassSubmitData {
    pub fn new(compute_pass: CompiledComputePassId) -> Self {
        ComputePassSubmitData {
            compute_pass,
            dispatches: vec![],
        }
    }

    pub fn dispatch(&mut self, compute_program: ComputeProgramId, dispatch: Dispatch) -> &mut Self {
        self.dispatches
            .push((compute_program, dispatch, DispatchType::NonBlocking));
        self
    }

    pub fn dispatch_blocking(
        &mut self,
        compute_program: ComputeProgramId,
        dispatch: Dispatch,
    ) -> &mut Self {
        self.dispatches
            .push((compute_program, dispatch, DispatchType::Blocking));
        self
    }
}

#[derive(Default, Clone)]
pub struct NewComputeProgramExt {}
#[derive(Default, Clone)]
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
            Context::WebGpu(_) => todo!(),
        }
    }

    pub fn compile_compute_pass(
        &mut self,
        compute_pass: ComputePass,
        ext: Option<CompileComputePassExt>,
    ) -> GResult<CompiledComputePassId> {
        match self {
            Context::Vulkan(vk) => vk.compile_compute_pass(compute_pass, ext),
            Context::WebGpu(_) => todo!(),
        }
    }
}
