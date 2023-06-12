use super::*;
use std::collections::HashMap;

//  TODO docs ALL OF THIS!

/// A compute pass that is later compiled with [`Context::compile_compute_pass`].
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ComputePass {
    pub(crate) programs: Vec<ComputeProgramId>,
    pub(crate) set_blocking: bool,
}

impl ComputePass {
    pub fn new() -> Self {
        ComputePass::default()
    }

    /// Registers compute programs that can then be bound to dispatches later.
    pub fn add_program(&mut self, program: ComputeProgramId) -> &mut Self {
        self.programs.push(program);
        self
    }

    /// A hint for whether this pass must complete before the next one.
    /// Although this is a hint, please use it anyway to reduce platform-specific bugs.
    /// This option is `false` by default.
    pub fn set_blocking(&mut self, set: bool) -> &mut Self {
        self.set_blocking = set;
        self
    }
}

/// Configure options regarding compute dispatches such as dynamic buffer indices, the
/// bound program, work group counts, etc.
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
    /// Set the index to use for a dynamic uniform buffer for this dispatch.
    /// If you are using a dynamic uniform buffer, this option is MANDITORY.
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

/// Submit data for compute passes.
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

    /// Fills in the basic options for a non-blocking dispatch.
    /// Subsequent dispatches MAY NOT wait for this one.
    /// Also returns a [`Dispatch`] with additional options.
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

    /// Fills in the basic options for dispatches.
    /// Subsequent dispatches much wait for this one.
    /// Also returns a [`Dispatch`] with additional options.
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

/// Currently has extra extension options.
#[derive(Default, Debug, Clone)]
pub struct NewComputeProgramExt {}
/// Currently has extra extension options.
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
