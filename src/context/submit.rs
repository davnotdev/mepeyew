use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClearDepthStencil {
    pub depth: f32,
    pub stencil: u32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DrawType {
    Draw,
    DrawIndexed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DrawScissor {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DynamicGenericBufferId {
    Uniform(DynamicUniformBufferId),
}

#[derive(Debug, Clone)]
pub struct Draw {
    pub(crate) ty: DrawType,
    pub(crate) first: usize,
    pub(crate) count: usize,
    pub(crate) first_instance: usize,
    pub(crate) instance_count: usize,
    pub(crate) program: ProgramId,
    pub(crate) viewport: Option<DrawViewport>,
    pub(crate) scissor: Option<DrawScissor>,
    pub(crate) dynamic_buffer_indices: HashMap<DynamicGenericBufferId, usize>,
}

impl Draw {
    pub fn set_instance(&mut self, first: usize, count: usize) -> &mut Self {
        self.first_instance = first;
        self.instance_count = count;
        self
    }

    pub fn set_viewport(&mut self, viewport: DrawViewport) -> &mut Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn set_scissor(&mut self, scissor: DrawScissor) -> &mut Self {
        self.scissor = Some(scissor);
        self
    }

    /// Set the index to use for a dynamic uniform buffer for this draw.
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

#[derive(Default, Debug, Clone)]
pub struct StepSubmitData {
    pub(crate) draws: Vec<Draw>,
}

impl StepSubmitData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, program: ProgramId, first: usize, count: usize) -> &mut Draw {
        self.draws.push(Draw {
            ty: DrawType::Draw,
            first,
            count,
            program,
            first_instance: 0,
            instance_count: 1,
            viewport: None,
            scissor: None,
            dynamic_buffer_indices: HashMap::new(),
        });
        self.draws.last_mut().unwrap()
    }

    pub fn draw_indexed(&mut self, program: ProgramId, first: usize, count: usize) -> &mut Draw {
        self.draws.push(Draw {
            ty: DrawType::DrawIndexed,
            first,
            count,
            program,
            first_instance: 0,
            instance_count: 1,
            viewport: None,
            scissor: None,
            dynamic_buffer_indices: HashMap::new(),
        });
        self.draws.last_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct PassSubmitData {
    pub(crate) pass: CompiledPassId,
    pub(crate) steps_datas: Vec<StepSubmitData>,

    pub(crate) clear_colors: HashMap<PassLocalAttachment, ClearColor>,
    pub(crate) clear_depths: HashMap<PassLocalAttachment, ClearDepthStencil>,
}

impl PassSubmitData {
    pub fn new(pass: CompiledPassId) -> Self {
        PassSubmitData {
            pass,
            steps_datas: vec![],

            clear_colors: HashMap::new(),
            clear_depths: HashMap::new(),
        }
    }

    pub fn step(&mut self, pass: StepSubmitData) -> &mut Self {
        self.steps_datas.push(pass);
        self
    }

    /// If you plan on clearing the surface attachment, ensure that the [`NewPassExt::surface_attachment_load_op`] was set.
    pub fn set_attachment_clear_color(
        &mut self,
        attachment_ref: PassLocalAttachment,
        clear_color: ClearColor,
    ) -> &mut Self {
        self.clear_colors.insert(attachment_ref, clear_color);
        self
    }

    /// If you plan on clearing the surface attachment, ensure that the [`NewPassExt::surface_attachment_load_op`] was set.
    pub fn set_attachment_clear_depth_stencil(
        &mut self,
        attachment_ref: PassLocalAttachment,
        clear_depth_stencil: ClearDepthStencil,
    ) -> &mut Self {
        self.clear_depths
            .insert(attachment_ref, clear_depth_stencil);
        self
    }
}

#[derive(Debug, Clone)]
pub enum SubmitPassType {
    Render(PassSubmitData),
    Compute(extensions::ComputePassSubmitData),
}

#[derive(Default, Debug, Clone)]
pub struct Submit<'transfer> {
    pub(crate) passes: Vec<SubmitPassType>,
    pub(crate) vbo_transfers: Vec<(VertexBufferId, &'transfer [VertexBufferElement])>,
    pub(crate) ibo_transfers: Vec<(IndexBufferId, &'transfer [IndexBufferElement])>,
    pub(crate) ubo_transfers: Vec<(UniformBufferId, &'transfer [u8])>,
    pub(crate) dyn_ubo_transfers: Vec<(DynamicUniformBufferId, &'transfer [u8], usize)>,
    pub(crate) ssbo_copy_backs: Vec<extensions::ShaderStorageBufferId>,
}

impl<'transfer> Submit<'transfer> {
    pub fn new() -> Self {
        Submit {
            passes: vec![],
            vbo_transfers: vec![],
            ibo_transfers: vec![],
            ubo_transfers: vec![],
            dyn_ubo_transfers: vec![],
            ssbo_copy_backs: vec![],
        }
    }

    pub fn pass(&mut self, data: PassSubmitData) -> &mut Self {
        self.passes.push(SubmitPassType::Render(data));
        self
    }

    pub fn compute_pass(&mut self, data: extensions::ComputePassSubmitData) -> &mut Self {
        self.passes.push(SubmitPassType::Compute(data));
        self
    }

    /// Ensure that [`BufferStorageType`] is set to `BufferStorageType::Dynamic`.
    pub fn transfer_into_vertex_buffer(
        &mut self,
        vbo: VertexBufferId,
        data: &'transfer [VertexBufferElement],
    ) -> &mut Self {
        self.vbo_transfers.push((vbo, data));
        self
    }

    /// Ensure that [`BufferStorageType`] is set to `BufferStorageType::Dynamic`.
    pub fn transfer_into_index_buffer(
        &mut self,
        ibo: IndexBufferId,
        data: &'transfer [IndexBufferElement],
    ) -> &mut Self {
        self.ibo_transfers.push((ibo, data));
        self
    }

    pub fn transfer_into_uniform_buffer<T: Copy>(
        &mut self,
        guard: UniformBufferTypeGuard<T>,
        data: &'transfer T,
    ) -> &mut Self {
        unsafe { self.transfer_into_uniform_buffer_unchecked(guard.0, data) };
        self
    }

    /// # Safety
    ///
    /// The type `T` is not validated.
    /// For validation, use [`Submit::transfer_into_uniform_buffer`].
    pub unsafe fn transfer_into_uniform_buffer_unchecked<T: Copy>(
        &mut self,
        ubo: UniformBufferId,
        data: &'transfer T,
    ) -> &mut Self {
        let untyped_slice =
            std::slice::from_raw_parts(data as *const T as *const u8, std::mem::size_of::<T>());
        self.ubo_transfers.push((ubo, untyped_slice));
        self
    }

    pub fn transfer_into_dynamic_uniform_buffer<T: Copy>(
        &mut self,
        guard: DynamicUniformBufferTypeGuard<T>,
        data: &'transfer T,
        index: usize,
    ) -> &mut Self {
        unsafe { self.transfer_into_dynamic_uniform_buffer_unchecked(guard.0, data, index) };
        self
    }

    /// # Safety
    ///
    /// The type `T` is not validated.
    /// For validation, use [`Submit::transfer_into_dynamic_uniform_buffer`].
    pub unsafe fn transfer_into_dynamic_uniform_buffer_unchecked<T: Copy>(
        &mut self,
        ubo: DynamicUniformBufferId,
        data: &'transfer T,
        index: usize,
    ) -> &mut Self {
        let untyped_slice =
            std::slice::from_raw_parts(data as *const T as *const u8, std::mem::size_of::<T>());
        self.dyn_ubo_transfers.push((ubo, untyped_slice, index));
        self
    }

    /// Write the shader storage buffer back into CPU memory after rendering.
    /// This is essential for [`Context::read_synced_shader_storage_buffer`]
    pub fn sync_shader_storage_buffer(
        &mut self,
        ssbo: extensions::ShaderStorageBufferId,
    ) -> &mut Self {
        self.ssbo_copy_backs.push(ssbo);
        self
    }
}

#[derive(Default, Debug, Clone)]
pub struct SubmitExt {
    pub sync: Option<()>,
}

impl Context {
    pub fn submit(&mut self, submit: Submit, ext: Option<SubmitExt>) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.submit(submit, ext),
            Self::WebGpu(wgpu) => wgpu.submit(submit, ext),
        }
    }
}
