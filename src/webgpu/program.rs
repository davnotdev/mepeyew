use super::*;

impl WebGpuContext {
    pub fn new_program(
        &mut self,
        shaders: &ShaderSet,
        uniforms: &[ShaderUniform],
        ext: Option<NewProgramExt>,
    ) -> GResult<ProgramId> {
        let program = WebGpuProgram::new(self, shaders, uniforms, ext)?;
        self.programs.push(program);
        Ok(ProgramId::from_id(self.programs.len() - 1))
    }
}

pub struct WebGpuProgram {
    pub vertex_module: GpuShaderModule,
    pub fragment_module: Option<GpuShaderModule>,
    pub bind_groups: WebGpuBindGroups,
    pub vertex_buffer_layout: GpuVertexBufferLayout,
    pub ext: NewProgramExt,
}

impl WebGpuProgram {
    pub fn new(
        context: &WebGpuContext,
        shaders: &ShaderSet,
        uniforms: &[ShaderUniform],
        ext: Option<NewProgramExt>,
    ) -> GResult<Self> {
        let (ty, vertex) = take_single_shader(&context.device, shaders, |ty| {
            matches!(ty, ShaderType::Vertex(_))
        })?
        .ok_or(gpu_api_err!("webgpu did not get a vertex shader"))?;

        let fragment = take_single_shader(&context.device, shaders, |ty| {
            matches!(ty, ShaderType::Fragment)
        })?
        .map(|(_, shader)| shader);

        let vertex_buffer_layout_attributes = Array::new();

        let vertex_buffer_layout_array_stride = if let ShaderType::Vertex(vertex_data) = ty {
            let mut accum_stride = 0;
            let vertex_size = std::mem::size_of::<VertexBufferElement>();
            assert_eq!(vertex_size, std::mem::size_of::<f32>());

            for (location, arg) in vertex_data.args.iter().enumerate() {
                let format = match arg.0 {
                    1 => GpuVertexFormat::Float32,
                    2 => GpuVertexFormat::Float32x2,
                    3 => GpuVertexFormat::Float32x3,
                    4 => GpuVertexFormat::Float32x4,
                    _ => Err(gpu_api_err!(
                        "webgpu an argument count of {} is invalid for vertex buffers",
                        arg.0
                    ))?,
                };
                let vertex_attr =
                    GpuVertexAttribute::new(format, accum_stride as f64, location as u32);
                accum_stride += arg.0 * vertex_size;
                vertex_buffer_layout_attributes.push(&vertex_attr);
            }

            accum_stride as f64
        } else {
            unreachable!()
        };

        let vertex_buffer_layout = GpuVertexBufferLayout::new(
            vertex_buffer_layout_array_stride,
            &vertex_buffer_layout_attributes,
        );

        let bind_groups = WebGpuBindGroups::new(context, uniforms)?;

        Ok(WebGpuProgram {
            vertex_module: vertex,
            fragment_module: fragment,

            bind_groups,
            vertex_buffer_layout,
            ext: ext.unwrap_or_default(),
        })
    }
}

fn take_single_shader<F>(
    device: &GpuDevice,
    shaders: &ShaderSet,
    compare_shader_ty: F,
) -> GResult<Option<(ShaderType, GpuShaderModule)>>
where
    F: Fn(&ShaderType) -> bool,
{
    let mut ret_ty = None;
    let list = shaders
        .0
        .iter()
        .filter_map(|(ty, src)| {
            if compare_shader_ty(ty) {
                let shader_module_info =
                    GpuShaderModuleDescriptor::new(std::str::from_utf8(src).unwrap());
                let shader_module = device.create_shader_module(&shader_module_info);
                ret_ty = Some(ty.clone());
                Some(shader_module)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if list.len() > 1 {
        Err(gpu_api_err!("webgpu got multiple of the same shader type"))?;
    }
    Ok(list.get(0).cloned().map(|s| (ret_ty.unwrap(), s)))
}
