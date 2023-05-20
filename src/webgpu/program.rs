use super::*;

const BINDING_GROUP_COUNT: usize = 4;

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
    pub bind_groups: Vec<GpuBindGroup>,
    pub bind_group_layouts: Vec<GpuBindGroupLayout>,
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
                    let layout = GpuTextureBindingLayout::new();
                    entry.texture(&layout);
                    GpuShaderStageFlags::FRAGMENT as u8
                }
                ShaderUniformType::Sampler(_) => {
                    let layout = GpuSamplerBindingLayout::new();
                    entry.sampler(&layout);
                    GpuShaderStageFlags::FRAGMENT as u8
                }
                ShaderUniformType::InputAttachment(_) => {
                    let mut layout = GpuTextureBindingLayout::new();
                    layout.sample_type(GpuTextureSampleType::UnfilterableFloat);
                    entry.texture(&layout);
                    GpuShaderStageFlags::FRAGMENT as u8
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
                context
                    .device
                    .create_bind_group_layout(&bind_group_layout_info)
            })
            .collect::<Vec<_>>();

        let mut bind_groups = (0..BINDING_GROUP_COUNT).map(|_| vec![]).collect::<Vec<_>>();

        uniforms.iter().try_for_each(|uniform| {
            let mut entry = GpuBindGroupEntry::new(uniform.binding as u32, &JsValue::null());
            match uniform.ty {
                ShaderUniformType::UniformBuffer(ubo_id) => {
                    let ubo = context.ubos.get(ubo_id.id()).ok_or(gpu_api_err!(
                        "program uniform buffer id {:?} does not exist",
                        ubo_id
                    ))?;
                    let buffer = GpuBufferBinding::new(&ubo.buffer);
                    entry.resource(&buffer);
                }
                ShaderUniformType::Texture(texture_id) => {
                    let texture = context.textures.get(texture_id.id()).ok_or(gpu_api_err!(
                        "program uniform texture id {:?} does not exist",
                        texture_id
                    ))?;
                    entry.resource(&texture.texture_view);
                }
                ShaderUniformType::Sampler(sampler_id) => {
                    let sampler = context.sampler_cache.get(sampler_id).ok_or(gpu_api_err!(
                        "program uniform sampler id {:?} does not exist",
                        sampler_id
                    ))?;
                    entry.resource(&sampler);
                }
                ShaderUniformType::InputAttachment(attachment_image_id) => {
                    let attachment_image = context
                        .attachment_images
                        .get(attachment_image_id.id())
                        .ok_or(gpu_api_err!(
                            "program uniform attachment image id {:?} does not exist",
                            attachment_image_id
                        ))?;
                    entry.resource(&attachment_image.texture_view);
                }
            }

            bind_groups[uniform.frequency as usize].push(entry);

            Ok(())
        })?;

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
                context.device.create_bind_group(&bind_group_info)
            })
            .collect::<Vec<_>>();

        Ok(WebGpuProgram {
            vertex_module: vertex,
            fragment_module: fragment,

            bind_groups,
            bind_group_layouts,
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
