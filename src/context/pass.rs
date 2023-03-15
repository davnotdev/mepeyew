use super::*;

#[derive(Clone)]
pub enum PassInputType {
    Color(PassInputLoadOpColorType),
    Depth(PassInputLoadOpDepthStencilType),
}

#[derive(Clone)]
pub enum PassInputLoadOpColorType {
    Load,
    Clear,
}

#[derive(Clone)]
pub enum PassInputLoadOpDepthStencilType {
    Load,
    Clear,
}

#[derive(Clone)]
pub struct PassAttachment {
    pub(crate) ty: PassInputType,
    //  Could technically be replaced with `.iter().enumerate()`.
    pub(crate) local_attachment_idx: usize,
    pub(crate) output_image: ImageId,
}

//  `surface_attachment` is always index 0 if set.
#[derive(Default, Clone)]
pub struct Pass {
    pub(crate) steps: Vec<PassStep>,
    pub(crate) attachments: Vec<PassAttachment>,
    pub(crate) surface_attachment: bool,
    pub(crate) depends_on_surface_size: bool,
    pub(crate) render_width: usize,
    pub(crate) render_height: usize,
}

#[derive(Default)]
pub struct NewPassExt {
    pub depends_on_surface_size: Option<()>,
    pub surface_attachment_load_op: Option<PassInputLoadOpColorType>,
}

impl Pass {
    pub fn new(render_width: usize, render_height: usize, ext: Option<NewPassExt>) -> Self {
        let ext = ext.unwrap_or_default();

        let mut pass = Pass {
            render_width,
            render_height,
            depends_on_surface_size: ext.depends_on_surface_size.is_some(),
            ..Default::default()
        };
        if let Some(surface_attachment_load_op) = ext.surface_attachment_load_op {
            pass.surface_attachment = true;
            pass.attachments.push(PassAttachment {
                ty: PassInputType::Color(surface_attachment_load_op),
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

    pub fn get_surface_local_attachment(&self) -> PassLocalAttachment {
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
        self.attachments.push(PassAttachment {
            ty,
            output_image: image,
            local_attachment_idx: self.attachments.len(),
        });
        PassLocalAttachment::from_id(self.attachments.len() - 1)
    }
}

#[derive(Default)]
pub struct CompilePassExt {}

impl Context {
    pub fn compile_pass(
        &mut self,
        pass: &Pass,
        ext: Option<CompilePassExt>,
    ) -> GResult<CompiledPassId> {
        match self {
            Self::Vulkan(vk) => vk.compile_pass(pass, ext),
        }
    }
}
