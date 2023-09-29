//! # Mepeyew
//!
//! ## Introduction
//!
//! Mepeyew is a rendering abstraction layer created for [`mewo`](https://github.com/davnotdev/mewo).
//! Essentially, Mepeyew allows you to draw graphics on the GPU without having to
//! worry about the platform specific details.
//! Additionally, Mepeyew has zero unnecessary dependencies, perfect for people who have
//! bundlephobia (like me).
//! For more details, see the [Github page](https://github.com/davnotdev/mepeyew).
//!
//! ## Usage
//!
//! Graphics programming is complicated...
//!
//! For this reason, my best advice for you is to have a look at the examples on the [Github page](https://github.com/davnotdev/mepeyew/tree/main/examples).
//!
//! ## Platform Dependent Nastiness
//!
//! Unfortunately, not everything can be fully abstracted.
//!
//! Here's the list of oddities to look out for.
//!
//! ### Uniform Padding
//!
//! Both Vulkan and WebGPU have very specific alignment requirements for uniform buffers (of any kind).
//! Failing to conform with these requirements leads to strange shader behaviour.
//! You can read [this blog](https://fvcaputo.github.io/2019/02/06/memory-alignment.html)
//! for a good explanation.
//!
//! ### Step Dependencies
//!
//! Certain methods such as [`PassStep::set_wait_for_depth_from_step`]
//! don't have an effect on all APIs.
//! However, on backends that do utilize these methods, they are indispensable.
//! For this reason, you should always use these methods even if you code appears
//! to work without it.

pub(crate) mod alignment;
pub mod context;
mod error;
mod mock;

#[cfg(all(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    feature = "vulkan"
))]
mod vulkan;
#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod webgpu;

pub use context::{
<<<<<<< Updated upstream
    extensions, extensions::*, Api, AttachmentImageColorFormat, AttachmentImageUsage,
    BufferStorageType, ClearColor, ClearDepthStencil, CompilePassExt, CompiledComputePassId,
    CompiledPassId, ComputeProgramId, Context, CubemapTextureUpload, Draw, DrawScissor,
    DrawViewport, DynamicUniformBufferTypeGuard, GetSamplerExt, IndexBufferElement, IndexBufferId,
    MipSamplerFilter, MsaaSampleCount, NewAttachmentImageExt, NewIndexBufferExt, NewPassExt,
    NewProgramExt, NewTextureExt, NewVertexBufferExt, Pass, PassInputLoadOpColorType,
    PassInputLoadOpDepthStencilType, PassStep, PassSubmitData, ProgramId, SamplerFilter, SamplerId,
    SamplerMode, ShaderBlendFactor, ShaderBlendOperation, ShaderCompareOp, ShaderCullFrontFace,
    ShaderCullMode, ShaderPrimitiveTopology, ShaderSet, ShaderStage, ShaderStencilOp, ShaderType,
    ShaderUniform, ShaderUniformType, StepSubmitData, Submit, SubmitExt, TextureFormat,
    UniformBufferId, UniformBufferTypeGuard, UploadCubemapTextureExt, UploadTextureExt,
    VertexBufferElement, VertexBufferId, VertexBufferInput,
=======
    extensions, extensions::*, Api, AttachmentImageColorFormat, AttachmentImageId,
    AttachmentImageUsage, BufferStorageType, ClearColor, ClearDepthStencil, CompilePassExt,
    CompiledComputePassId, CompiledPassId, ComputeProgramId, Context, CubemapTextureUpload, Draw,
    DrawScissor, DrawViewport, DynamicUniformBufferTypeGuard, GetSamplerExt, IndexBufferElement,
    IndexBufferId, MipSamplerFilter, MsaaSampleCount, NewAttachmentImageExt, NewIndexBufferExt,
    NewPassExt, NewProgramExt, NewTextureExt, NewVertexBufferExt, Pass, PassInputLoadOpColorType,
    PassInputLoadOpDepthStencilType, PassLocalAttachment, PassStep, PassSubmitData, ProgramId,
    SamplerFilter, SamplerId, SamplerMode, ShaderBlendFactor, ShaderBlendOperation,
    ShaderCompareOp, ShaderCullFrontFace, ShaderCullMode, ShaderPrimitiveTopology, ShaderSet,
    ShaderStage, ShaderStencilOp, ShaderType, ShaderUniform, ShaderUniformType, StepSubmitData,
    Submit, SubmitExt, TextureFormat, UniformBufferId, UniformBufferTypeGuard,
    UploadCubemapTextureExt, UploadTextureExt, VertexBufferElement, VertexBufferId,
    VertexBufferInput,
>>>>>>> Stashed changes
};
pub use error::{GResult, GpuError};
