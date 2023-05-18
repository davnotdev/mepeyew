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

pub struct Draw {
    pub first: usize,
    pub count: usize,
}

#[derive(Default)]
pub struct StepSubmitData {
    pub(crate) draws: Vec<Draw>,
    pub(crate) draws_indexed: Vec<Draw>,
}

impl StepSubmitData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw(&mut self, first: usize, count: usize) -> &mut Self {
        self.draws.push(Draw { first, count });
        self
    }

    pub fn draw_indexed(&mut self, first: usize, count: usize) -> &mut Self {
        self.draws_indexed.push(Draw { first, count });
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

    /// Ensure that the `surface_attachment_load_op` from [`NewPassExt`] was set.
    pub fn set_attachment_clear_color(
        &mut self,
        attachment_ref: PassLocalAttachment,
        clear_color: ClearColor,
    ) -> &mut Self {
        self.clear_colors.insert(attachment_ref, clear_color);
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
