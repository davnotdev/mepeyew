use super::error::*;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use super::vulkan::*;

mod buffer;
mod image;
mod pass;
mod pass_step;
mod platform;
mod program;
mod submit;

// mod sequence;
// mod pass;
// mod submit;

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
pub struct ProgramId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ImageId(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct PassStepDependency(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct PassLocalAttachment(usize);
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct CompiledPassId(usize);

def_id_ty!(VertexBufferId);
def_id_ty!(IndexBufferId);
def_id_ty!(ProgramId);
def_id_ty!(ImageId);
def_id_ty!(PassStepDependency);
def_id_ty!(PassLocalAttachment);
def_id_ty!(CompiledPassId);

pub enum Api {
    Vulkan,
}

pub enum Context {
    Vulkan(VkContext),
}

pub use buffer::{BufferStorageType, IndexBufferElement, VertexBufferElement};
pub use image::ImageUsage;
pub use pass::{
    Pass, PassInput, PassInputLoadOpColorType, PassInputLoadOpDepthStencilType, PassInputType,
};
pub use pass_step::PassStep;
pub use program::{ShaderSet, ShaderType};
pub use submit::{ClearColor, ClearDepthStencil, Draw, PassSubmitData, StepSubmitData, Submit};
