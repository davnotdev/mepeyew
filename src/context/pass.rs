use super::*;

pub enum PassInputType {
    Color(PassInputLoadOpColorType),
    Depth(PassInputLoadOpDepthStencilType),
}

pub enum PassInputLoadOpColorType {
    Load,
    Clear,
}

pub enum PassInputLoadOpDepthStencilType {
    Load,
    Clear,
}

pub struct PassInput {
    pub(crate) ty: PassInputType,
    //  Could technically be replaced with `.iter().enumerate()`.
    pub(crate) local_attachment_idx: usize,
    pub(crate) output_image: ImageId,
}

//  `output_attachment` is always index 0 if set.
#[derive(Default)]
pub struct Pass {
    pub(crate) steps: Vec<PassStep>,
    pub(crate) inputs: Vec<PassInput>,
    pub(crate) output_attachment: bool,
}

impl Pass {
    pub fn new(output_attachment: Option<PassInputLoadOpColorType>) -> Self {
        let mut pass = Pass::default();
        if let Some(output_attachment_load_op) = output_attachment {
            pass.output_attachment = true;
            pass.inputs.push(PassInput {
                ty: PassInputType::Color(output_attachment_load_op),
                local_attachment_idx: 0,
                //  Will be ignored.
                output_image: ImageId(0),
            })
        }
        pass
    }

    pub fn add_step(&mut self) -> &mut PassStep {
        let dep = PassStepDependency::from_id(self.steps.len());
        let pass_step = PassStep {
            step_dependency: dep,
            ..Default::default()
        };
        self.steps.push(pass_step);
        &mut self.steps[dep.id()]
    }

    pub fn get_output_attachment(&self) -> PassLocalAttachment {
        PassLocalAttachment::from_id(0)
    }

    pub fn add_attachment_color_image(
        &mut self,
        color: ImageId,
        load_op: PassInputLoadOpColorType,
    ) -> PassLocalAttachment {
        self.add_attachment(color, PassInputType::Color(load_op))
    }

    pub fn add_attachment_depth_image(
        &mut self,
        depth: ImageId,
        load_op: PassInputLoadOpDepthStencilType,
    ) -> PassLocalAttachment {
        self.add_attachment(depth, PassInputType::Depth(load_op))
    }

    fn add_attachment(&mut self, image: ImageId, ty: PassInputType) -> PassLocalAttachment {
        self.inputs.push(PassInput {
            ty,
            output_image: image,
            local_attachment_idx: self.inputs.len(),
        });
        PassLocalAttachment::from_id(self.inputs.len() - 1)
    }
}

impl Context {
    pub fn compile_pass(&mut self, pass: &Pass) -> GResult<CompiledPassId> {
        match self {
            Self::Vulkan(vk) => vk.compile_pass(pass),
        }
    }
}
