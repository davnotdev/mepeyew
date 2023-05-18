use super::error::*;

#[allow(unused_imports)]
use super::mock::*;
#[cfg(feature = "vulkan")]
use super::vulkan::*;
#[cfg(feature = "webgpu")]
use super::webgpu::*;

pub mod extensions;

mod buffer;
mod pass;
mod pass_step;
mod platform;
mod program;
mod sampler;
mod submit;
mod texture;

macro_rules! def_id_ty {
    ($NAME: ident) => {
        impl $NAME {
            pub fn from_id(id: usize) -> Self {
                Self(id)
            }

            pub fn id(&self) -> usize {
                self.0
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct VertexBufferId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct IndexBufferId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct UniformBufferId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ProgramId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SamplerId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TextureId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AttachmentImageId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct PassStepDependency(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PassLocalAttachment(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CompiledPassId(usize);

def_id_ty!(VertexBufferId);
def_id_ty!(IndexBufferId);
def_id_ty!(UniformBufferId);
def_id_ty!(ProgramId);
def_id_ty!(SamplerId);
def_id_ty!(TextureId);
def_id_ty!(AttachmentImageId);
def_id_ty!(PassStepDependency);
def_id_ty!(PassLocalAttachment);
def_id_ty!(CompiledPassId);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Api {
    #[cfg(feature = "vulkan")]
    Vulkan,
    #[cfg(feature = "webgpu")]
    WebGpu,
}

pub enum Context {
    #[cfg(feature = "vulkan")]
    Vulkan(VkContext),
    #[cfg(not(feature = "vulkan"))]
    Vulkan(MockContext),
    #[cfg(feature = "webgpu")]
    WebGpu(WebGpuContext),
    #[cfg(not(feature = "webgpu"))]
    WebGpu(MockContext),
}

pub use buffer::{
    BufferStorageType, IndexBufferElement, NewIndexBufferExt, NewUniformBufferExt,
    NewVertexBufferExt, VertexBufferElement, VertexBufferInput, VertexInputArgCount,
};
pub use extensions::{Extension, ExtensionType};
pub use pass::{
    CompilePassExt, NewPassExt, Pass, PassAttachment, PassInputLoadOpColorType,
    PassInputLoadOpDepthStencilType, PassInputType,
};
pub use pass_step::PassStep;
pub use program::{
    NewProgramExt, ShaderDepthCompareOp, ShaderSet, ShaderType, ShaderUniform,
    ShaderUniformFrequencyHint, ShaderUniformType,
};
pub use sampler::{GetSamplerExt, SamplerFilter, SamplerMode};
pub use submit::{
    ClearColor, ClearDepthStencil, Draw, PassSubmitData, StepSubmitData, Submit, SubmitExt,
};
pub use texture::{
    AttachmentImageUsage, NewAttachmentImageExt, NewTextureExt, ResizeTextureExt, TextureFormat,
    UploadTextureExt,
};
