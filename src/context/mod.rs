use super::error::*;

#[allow(unused_imports)]
use super::mock::*;
#[cfg(all(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    feature = "vulkan"
))]
use super::vulkan::*;
#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
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

#[macro_export]
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
pub struct DynamicUniformBufferId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ProgramId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ComputeProgramId(usize);
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CompiledComputePassId(usize);

def_id_ty!(VertexBufferId);
def_id_ty!(IndexBufferId);
def_id_ty!(UniformBufferId);
def_id_ty!(DynamicUniformBufferId);
def_id_ty!(ProgramId);
def_id_ty!(ComputeProgramId);
def_id_ty!(SamplerId);
def_id_ty!(TextureId);
def_id_ty!(AttachmentImageId);
def_id_ty!(PassStepDependency);
def_id_ty!(PassLocalAttachment);
def_id_ty!(CompiledPassId);
def_id_ty!(CompiledComputePassId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Api {
    Vulkan,
    WebGpu,
}

#[allow(clippy::large_enum_variant)]
pub enum Context {
    #[cfg(all(
        not(all(target_arch = "wasm32", target_os = "unknown")),
        feature = "vulkan"
    ))]
    Vulkan(VkContext),
    #[cfg(any(target_arch = "wasm32", target_os = "unknown", not(feature = "vulkan")))]
    Vulkan(MockContext),
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    WebGpu(WebGpuContext),
    #[cfg(not(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown")))]
    WebGpu(MockContext),
}

pub use buffer::{
    BufferStorageType, DynamicUniformBufferTypeGuard, IndexBufferElement,
    NewDynamicUniformBufferExt, NewIndexBufferExt, NewUniformBufferExt, NewVertexBufferExt,
    UniformBufferTypeGuard, VertexBufferElement, VertexBufferInput,
};
pub use extensions::{Extension, Extensions};
pub use pass::{
    CompilePassExt, MsaaSampleCount, NewPassExt, Pass, PassAttachment, PassInputLoadOpColorType,
    PassInputLoadOpDepthStencilType, PassInputType,
};
pub use pass_step::PassStep;
pub use program::{
    NewProgramExt, ShaderBlendFactor, ShaderBlendOperation, ShaderCompareOp, ShaderCullFrontFace,
    ShaderCullMode, ShaderPrimitiveTopology, ShaderSet, ShaderStage, ShaderStencilOp, ShaderType,
    ShaderUniform, ShaderUniformData, ShaderUniformType,
};
pub use sampler::{GetSamplerExt, MipSamplerFilter, SamplerFilter, SamplerMode};
pub use submit::{
    BlitToSurface, ClearColor, ClearDepthStencil, Draw, DrawScissor, DrawType, DrawViewport,
    DynamicGenericBufferId, PassSubmitData, StepSubmitData, Submit, SubmitExt, SubmitPassType,
};
pub use texture::{
    AttachmentImageColorFormat, AttachmentImageUsage, CubemapTextureUpload, NewAttachmentImageExt,
    NewTextureExt, TextureFormat, UploadCubemapTextureExt, UploadTextureExt,
};
