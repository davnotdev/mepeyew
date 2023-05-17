use super::*;

const BINDING_GROUP_COUNT: usize = 4;

impl WebGpuContext {
    pub fn new_program(
        &mut self,
        shaders: &ShaderSet,
        uniforms: &[ShaderUniform],
        ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        let program = WebGpuProgram::new(&self.device, shaders, uniforms, ext)?;
        self.programs.push(program);
        Ok(ProgramId::from_id(self.programs.len() - 1))
    }
}

pub struct WebGpuProgram {
    vertex_module: GpuShaderModule,
    fragment_module: Option<GpuShaderModule>,
    bind_groups: Vec<GpuBindGroup>,
    bind_group_layouts: Vec<GpuBindGroupLayout>,
}

impl WebGpuProgram {
    pub fn new(
        device: &GpuDevice,
        shaders: &ShaderSet,
        uniforms: &[ShaderUniform],
        _ext: Option<NewProgramExt>,
    ) -> GResult<Self> {
        let vertex = take_single_shader(device, shaders, |ty| matches!(ty, ShaderType::Vertex(_)))?
            .ok_or(gpu_api_err!("webgpu did not get a vertex shader"))?;

        let fragment =
            take_single_shader(device, shaders, |ty| matches!(ty, ShaderType::Fragment))?;

        let mut bind_group_layouts = (0..BINDING_GROUP_COUNT).map(|_| vec![]).collect::<Vec<_>>();

        uniforms.iter().for_each(|uniform| {
            let mut entry = GpuBindGroupLayoutEntry::new(uniform.binding as u32, 0);

            let visibility = match uniform.ty {
                ShaderUniformType::UniformBuffer(_) => {
                    let mut layout = GpuBufferBindingLayout::new();
                    layout.type_(GpuBufferBindingType::Uniform);
                    entry.buffer(&layout);
                    GpuShaderStageFlags::VERTEX as u8 | GpuShaderStageFlags::FRAGMENT as u8
                }
                ShaderUniformType::Texture(_) => {
                    unimplemented!();
                    GpuShaderStageFlags::FRAGMENT as u8
                }
                ShaderUniformType::InputAttachment(_) => {
                    unimplemented!();
                    GpuShaderStageFlags::VERTEX as u8 | GpuShaderStageFlags::FRAGMENT as u8
                }
            };

            entry.visibility(visibility as u32);

            bind_group_layouts[uniform.frequency as usize].push(entry);
        });

        let bind_group_layouts = bind_group_layouts
            .into_iter()
            .map(|bindings| {
                let entries = Array::new();
                bindings.iter().for_each(|binding| {
                    entries.push(binding);
                });
                let bind_group_layout_info = GpuBindGroupLayoutDescriptor::new(&entries);
                device.create_bind_group_layout(&bind_group_layout_info)
            })
            .collect::<Vec<_>>();

        let bind_groups = (0..BINDING_GROUP_COUNT).map(|_| vec![]).collect::<Vec<_>>();

        uniforms.iter().for_each(|uniform| {
            let entry = GpuBindGroupEntry::new(uniform.binding as u32, &JsValue::null());
            match uniform.ty {
                ShaderUniformType::UniformBuffer(_) => {
                    unimplemented!()
                }
                ShaderUniformType::Texture(_) => {
                    unimplemented!()
                }
                ShaderUniformType::InputAttachment(_) => {
                    unimplemented!()
                }
            }

            bind_groups[uniform.frequency as usize].push(entry);
        });

        let bind_groups = bind_groups
            .into_iter()
            .enumerate()
            .map(|(idx, bindings)| {
                let entries = Array::new();
                bindings.iter().for_each(|binding| {
                    entries.push(binding);
                });
                let layout = &bind_group_layouts[idx];
                let bind_group_info = GpuBindGroupDescriptor::new(&entries, &layout);
                device.create_bind_group(&bind_group_info)
            })
            .collect::<Vec<_>>();

        Ok(WebGpuProgram {
            vertex_module: vertex,
            fragment_module: fragment,

            bind_groups,
            bind_group_layouts,
        })
    }
}

fn take_single_shader<F>(
    device: &GpuDevice,
    shaders: &ShaderSet,
    compare_shader_ty: F,
) -> GResult<Option<GpuShaderModule>>
where
    F: Fn(&ShaderType) -> bool,
{
    let list = shaders
        .0
        .iter()
        .filter_map(|(ty, src)| {
            if compare_shader_ty(ty) {
                let shader_module_info =
                    GpuShaderModuleDescriptor::new(std::str::from_utf8(src).unwrap());
                let shader_module = device.create_shader_module(&shader_module_info);
                Some(shader_module)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if list.len() > 1 {
        Err(gpu_api_err!("webgpu got multiple of the same shader type"))?;
    }
    Ok(list.get(0).cloned())
}
