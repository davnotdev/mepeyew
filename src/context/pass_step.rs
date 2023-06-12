use super::*;

#[derive(Default, Debug, Clone)]
pub struct PassStep {
    pub(crate) step_dependency: PassStepDependency,

    pub(crate) vertex_buffers: Vec<VertexBufferId>,
    pub(crate) index_buffer: Option<IndexBufferId>,

    pub(crate) programs: Vec<ProgramId>,

    pub(crate) write_colors: Vec<PassLocalAttachment>,
    pub(crate) write_depth: Option<PassLocalAttachment>,

    pub(crate) wait_for_color_from: Option<(PassStepDependency, ShaderStage)>,
    pub(crate) wait_for_depth_from: Option<(PassStepDependency, ShaderStage)>,

    pub(crate) read_attachment: Vec<PassLocalAttachment>,
}

impl PassStep {
    pub fn add_vertex_buffer(&mut self, vbo: VertexBufferId) -> &mut Self {
        self.vertex_buffers.push(vbo);
        self
    }

    pub fn set_index_buffer(&mut self, ibo: IndexBufferId) -> &mut Self {
        self.index_buffer = Some(ibo);
        self
    }

    pub fn add_program(&mut self, program: ProgramId) -> &mut Self {
        self.programs.push(program);
        self
    }

    /// Wait for a specific step to finish rendering.
    /// The dependency can be obtained with `step.get_step_dependency()`.
    pub fn set_wait_for_color_from_step(
        &mut self,
        dependency: PassStepDependency,
        shader_usage: ShaderStage,
    ) -> &mut Self {
        self.wait_for_color_from = Some((dependency, shader_usage));
        self
    }

    /// Wait for a specific step to complete depth testing.
    /// The dependency can be obtained with `step.get_step_dependency()`.
    pub fn set_wait_for_depth_from_step(
        &mut self,
        dependency: PassStepDependency,
        shader_usage: ShaderStage,
    ) -> &mut Self {
        self.wait_for_depth_from = Some((dependency, shader_usage));
        self
    }

    /// Add a [`PassLocalAttachment`] to draw into.
    pub fn add_write_color(&mut self, local_attachment: PassLocalAttachment) -> &mut Self {
        self.write_colors.push(local_attachment);
        self
    }

    /// Add a [`PassLocalAttachment`] to use for depth testing.
    pub fn set_write_depth(&mut self, local_attachment: PassLocalAttachment) -> &mut Self {
        self.write_depth = Some(local_attachment);
        self
    }

    /// Denote that a [`PassLocalAttachment`] is used as input.
    pub fn read_local_attachment(&mut self, local_attachment: PassLocalAttachment) -> &mut Self {
        self.read_attachment.push(local_attachment);
        self
    }

    /// Get a [`PassStepDependency`] to have another step wait for this one to complete.
    pub fn get_step_dependency(&self) -> PassStepDependency {
        self.step_dependency
    }
}
