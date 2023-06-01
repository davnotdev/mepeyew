mod extensions;

use super::context::*;
use super::error::GResult;

///  This is used when disabling backends.
///  You can use this as a sort of reference for implementations.
pub struct MockContext;

#[allow(dead_code)]
impl MockContext {
    pub fn new(_extensions: &[Extension]) -> GResult<Self> {
        unimplemented!("No backend chosen")
    }

    pub fn new_vertex_buffer(
        &mut self,
        _data: &[VertexBufferElement],
        _storage_type: BufferStorageType,
        _ext: Option<NewVertexBufferExt>,
    ) -> GResult<VertexBufferId> {
        unimplemented!("No backend chosen")
    }

    pub fn new_index_buffer(
        &mut self,
        _data: &[IndexBufferElement],
        _storage_type: BufferStorageType,
        _ext: Option<NewIndexBufferExt>,
    ) -> GResult<IndexBufferId> {
        unimplemented!("No backend chosen")
    }

    pub fn new_uniform_buffer<T: Copy>(
        &mut self,
        _data: &T,
        _ext: Option<NewUniformBufferExt>,
    ) -> GResult<UniformBufferId> {
        unimplemented!("No backend chosen")
    }

    pub fn new_dynamic_uniform_buffer<T: Copy>(
        &mut self,
        _data: &[T],
        _ext: Option<NewDynamicUniformBufferExt>,
    ) -> GResult<DynamicUniformBufferId> {
        todo!()
    }

    pub fn get_sampler(&mut self, _ext: Option<GetSamplerExt>) -> GResult<SamplerId> {
        unimplemented!("No backend chosen")
    }

    pub fn new_shader_storage_buffer<T: Copy>(
        &mut self,
        _data: &T,
        _ext: Option<NewShaderStorageBufferExt>,
    ) -> GResult<ShaderStorageBufferId> {
        unimplemented!("No backend chosen")
    }

    pub fn read_synced_shader_storage_buffer<T: Copy>(
        &self,
        _ssbo: ShaderStorageBufferId,
        _ext: Option<ReadSyncedShaderStorageBufferExt>,
    ) -> GResult<T> {
        unimplemented!("No backend chosen")
    }

    pub fn new_texture(
        &mut self,
        _width: usize,
        _height: usize,
        _format: TextureFormat,
        _ext: Option<NewTextureExt>,
    ) -> GResult<TextureId> {
        unimplemented!("No backend chosen")
    }

    pub fn upload_texture(
        &mut self,
        _texture: TextureId,
        _data: &[u8],
        _ext: Option<UploadTextureExt>,
    ) -> GResult<()> {
        unimplemented!("No backend chosen")
    }

    pub fn get_texture_max_lod(&self, _texture: TextureId) -> GResult<f32> {
        unimplemented!("No backend chosen")
    }

    pub fn new_attachment_image(
        &mut self,
        _initial_width: usize,
        _initial_height: usize,
        _attachment_usage: AttachmentImageUsage,
        _ext: Option<NewAttachmentImageExt>,
    ) -> GResult<AttachmentImageId> {
        unimplemented!("No backend chosen")
    }

    pub fn compile_pass(
        &mut self,
        _pass: &Pass,
        _ext: Option<CompilePassExt>,
    ) -> GResult<CompiledPassId> {
        unimplemented!("No backend chosen")
    }

    pub fn new_program(
        &mut self,
        _shaders: &ShaderSet,
        _uniforms: &[ShaderUniform],
        _ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        unimplemented!("No backend chosen")
    }

    pub fn submit(&mut self, _submit: Submit, _ext: Option<SubmitExt>) -> GResult<()> {
        unimplemented!("No backend chosen")
    }

    pub fn new_compute_program(
        &mut self,
        _code: &[u8],
        _uniforms: &[ShaderUniform],
        _ext: Option<NewComputeProgramExt>,
    ) -> GResult<ComputeProgramId> {
        unimplemented!("No backend chosen")
    }

    pub fn compile_compute_pass(
        &mut self,
        _compute_pass: ComputePass,
        _ext: Option<CompileComputePassExt>,
    ) -> GResult<CompiledComputePassId> {
        unimplemented!("No backend chosen")
    }
}
