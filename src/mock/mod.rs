mod extensions;

use super::context::{
    extensions as context_extensions, BufferStorageType, CompilePassExt, CompiledPassId, Extension,
    ExtensionType, ImageId, ImageUsage, IndexBufferElement, IndexBufferId, NewImageExt,
    NewIndexBufferExt, NewProgramExt, NewVertexBufferExt, Pass, ProgramId, ShaderSet, ShaderType,
    Submit, SubmitExt, VertexBufferElement, VertexBufferId,
};
use super::error::GResult;

//  This is used when disabling backends.
//  You can use this as a sort of reference for implementations.

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

    pub fn new_image(
        &mut self,
        _width: usize,
        _height: usize,
        _usage: ImageUsage,
        _ext: NewImageExt,
    ) -> GResult<ImageId> {
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
        _ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        unimplemented!("No backend chosen")
    }

    pub fn submit(&mut self, _submit: Submit, _ext: Option<SubmitExt>) -> GResult<()> {
        unimplemented!("No backend chosen")
    }
}
