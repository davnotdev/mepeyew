use super::*;

#[derive(Default, Clone)]
pub struct PassStep {
    pub(crate) step_dependency: PassStepDependency,

    pub(crate) vertex_buffers: Vec<VertexBufferId>,
    pub(crate) index_buffer: Option<IndexBufferId>,
    pub(crate) program: Option<ProgramId>,

    pub(crate) write_colors: Vec<PassLocalAttachment>,
    pub(crate) write_depth: Option<PassLocalAttachment>,

    pub(crate) wait_for_color_from: Option<(PassStepDependency, ShaderType)>,
    pub(crate) wait_for_depth_from: Option<(PassStepDependency, ShaderType)>,
}

//  TODO EXT: Quiz Users ie Have users input the shader attachment / location indices which we then validate.

impl PassStep {
    pub fn add_vertex_buffer(&mut self, vbo: VertexBufferId) -> &mut Self {
        self.vertex_buffers.push(vbo);
        self
    }

    pub fn set_index_buffer(&mut self, ibo: IndexBufferId) -> &mut Self {
        self.index_buffer = Some(ibo);
        self
    }

    pub fn set_program(&mut self, program: ProgramId) -> &mut Self {
        self.program = Some(program);
        self
    }

    pub fn set_wait_for_color_from_step(
        &mut self,
        dependency: PassStepDependency,
        shader_usage: ShaderType,
    ) -> &mut Self {
        self.wait_for_color_from = Some((dependency, shader_usage));
        self
    }

    pub fn set_wait_for_depth_from_step(
        &mut self,
        dependency: PassStepDependency,
        shader_usage: ShaderType,
    ) -> &mut Self {
        self.wait_for_depth_from = Some((dependency, shader_usage));
        self
    }

    pub fn add_write_color(&mut self, local_attachment: PassLocalAttachment) -> &mut Self {
        self.write_colors.push(local_attachment);
        self
    }

    pub fn set_write_depth(&mut self, local_attachment: PassLocalAttachment) -> &mut Self {
        self.write_depth = Some(local_attachment);
        self
    }

    pub fn get_step_dependency(&self) -> PassStepDependency {
        self.step_dependency
    }
}
