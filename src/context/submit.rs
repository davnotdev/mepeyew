use super::*;
use std::collections::HashMap;

//  TODO FIX: Assert that no two passes have the output attachment.

pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub struct ClearDepthStencil {
    pub depth: f32,
    pub stencil: u32,
}

pub enum DrawType {
    Draw,
    DrawIndexed,
}

#[derive(Clone, Copy)]
pub struct DrawViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy)]
pub struct DrawScissor {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

pub struct Draw {
    pub(crate) ty: DrawType,
    pub(crate) first: usize,
    pub(crate) count: usize,
    pub(crate) program: ProgramId,
    pub(crate) viewport: Option<DrawViewport>,
    pub(crate) scissor: Option<DrawScissor>,
}

#[derive(Default)]
pub struct StepSubmitData {
    pub(crate) draws: Vec<Draw>,
}

impl StepSubmitData {
    pub fn new() -> Self {
        Self::default()
    }

    //  TODO docs
    pub fn set_draw_viewport(&mut self, viewport: DrawViewport) {
        let idx = self.draws.len() - 1;
        self.draws[idx].viewport = Some(viewport);
    }

    //  TODO docs
    pub fn set_draw_scissor(&mut self, scissor: DrawScissor) {
        let idx = self.draws.len() - 1;
        self.draws[idx].scissor = Some(scissor);
    }

    pub fn draw(&mut self, program: ProgramId, first: usize, count: usize) -> &mut Self {
        self.draws.push(Draw {
            ty: DrawType::Draw,
            first,
            count,
            program,
            viewport: None,
            scissor: None,
        });
        self
    }

    pub fn draw_indexed(&mut self, program: ProgramId, first: usize, count: usize) -> &mut Self {
        self.draws.push(Draw {
            ty: DrawType::DrawIndexed,
            first,
            count,
            program,
            viewport: None,
            scissor: None,
        });
        self
    }
}

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

    //  TODO EXT: Validate attachment type.

    /// If you plan on clearing the surface attachment, ensure that the [`NewPassExt::surface_attachment_load_op`] was set.
    pub fn set_attachment_clear_color(
        &mut self,
        attachment_ref: PassLocalAttachment,
        clear_color: ClearColor,
    ) -> &mut Self {
        self.clear_colors.insert(attachment_ref, clear_color);
        self
    }

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

#[derive(Default)]
pub struct Submit<'transfer> {
    pub(crate) passes: Vec<PassSubmitData>,
    pub(crate) vbo_transfers: Vec<(VertexBufferId, &'transfer [VertexBufferElement])>,
    pub(crate) ibo_transfers: Vec<(IndexBufferId, &'transfer [IndexBufferElement])>,
    pub(crate) ubo_transfers: Vec<(UniformBufferId, &'transfer [u8])>,
}

impl<'transfer> Submit<'transfer> {
    pub fn new() -> Self {
        Submit {
            passes: vec![],
            vbo_transfers: vec![],
            ibo_transfers: vec![],
            ubo_transfers: vec![],
        }
    }

    pub fn pass(&mut self, data: PassSubmitData) -> &mut Self {
        self.passes.push(data);
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

    /// Note that this function may not be safe as the `T` type is not validated.
    pub fn transfer_into_uniform_buffer<T: Copy>(
        &mut self,
        ubo: UniformBufferId,
        data: &'transfer T,
    ) -> &mut Self {
        let untyped_slice = unsafe {
            std::slice::from_raw_parts(data as *const T as *const u8, std::mem::size_of::<T>())
        };
        self.ubo_transfers.push((ubo, untyped_slice));
        self
    }
}

#[derive(Default)]
pub struct SubmitExt {}

impl Context {
    pub fn submit(&mut self, submit: Submit, ext: Option<SubmitExt>) -> GResult<()> {
        match self {
            Self::Vulkan(vk) => vk.submit(submit, ext),
            Self::WebGpu(wgpu) => wgpu.submit(submit, ext),
        }
    }
}
