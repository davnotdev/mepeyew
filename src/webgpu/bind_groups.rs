use super::*;
use std::collections::HashMap;

const WEBGPU_BIND_GROUP_COUNT: usize = 4;

pub struct WebGpuBindGroups {
    pub bind_groups: Vec<GpuBindGroup>,
    pub bind_group_layouts: Vec<GpuBindGroupLayout>,
    bind_groups_dynamic_indices: Vec<Vec<DynamicGenericBufferId>>,
}

impl WebGpuBindGroups {
    pub fn new(
        context: &WebGpuContext,
        uniforms: &[ShaderUniform],
        is_compute: bool,
    ) -> GResult<Self> {
        let mut bind_group_layouts = (0..WEBGPU_BIND_GROUP_COUNT)
            .map(|_| vec![])
            .collect::<Vec<_>>();

        let mut bind_groups_dynamic_indices = vec![vec![]; WEBGPU_BIND_GROUP_COUNT];

        let maybe_vertex_flag = if !is_compute {
            GpuShaderStageFlags::Vertex as u8
        } else {
            0
        };
        uniforms.iter().for_each(|uniform| {
            let mut dynamic_indices = vec![];
            let mut entry = GpuBindGroupLayoutEntry::new(uniform.binding as u32, 0);

            let visibility = match uniform.ty {
                ShaderUniformType::UniformBuffer(_) => {
                    let mut layout = GpuBufferBindingLayout::new();
                    layout.type_(GpuBufferBindingType::Uniform);
                    entry.buffer(&layout);
                    maybe_vertex_flag
                        | GpuShaderStageFlags::Fragment as u8
                        | GpuShaderStageFlags::Compute as u8
                }
                ShaderUniformType::DynamicUniformBuffer(id) => {
                    dynamic_indices.push(DynamicGenericBufferId::Uniform(id));
                    let mut layout = GpuBufferBindingLayout::new();
                    layout
                        .type_(GpuBufferBindingType::Uniform)
                        .has_dynamic_offset(true);
                    entry.buffer(&layout);
                    maybe_vertex_flag
                        | GpuShaderStageFlags::Fragment as u8
                        | GpuShaderStageFlags::Compute as u8
                }
                ShaderUniformType::ShaderStorageBuffer(_) => {
                    let mut layout = GpuBufferBindingLayout::new();
                    layout.type_(GpuBufferBindingType::Storage);
                    entry.buffer(&layout);
                    maybe_vertex_flag
                        | GpuShaderStageFlags::Fragment as u8
                        | GpuShaderStageFlags::Compute as u8
                }
                ShaderUniformType::ShaderStorageBufferReadOnly(_) => {
                    let mut layout = GpuBufferBindingLayout::new();
                    layout.type_(GpuBufferBindingType::ReadOnlyStorage);
                    entry.buffer(&layout);
                    maybe_vertex_flag
                        | GpuShaderStageFlags::Fragment as u8
                        | GpuShaderStageFlags::Compute as u8
                }
                ShaderUniformType::Texture(_) => {
                    let layout = GpuTextureBindingLayout::new();
                    entry.texture(&layout);
                    GpuShaderStageFlags::Fragment as u8 | GpuShaderStageFlags::Compute as u8
                }
                ShaderUniformType::Sampler(_) => {
                    let layout = GpuSamplerBindingLayout::new();
                    entry.sampler(&layout);
                    GpuShaderStageFlags::Fragment as u8 | GpuShaderStageFlags::Compute as u8
                }
                ShaderUniformType::InputAttachment(_) => {
                    let mut layout = GpuTextureBindingLayout::new();
                    layout.sample_type(GpuTextureSampleType::UnfilterableFloat);
                    entry.texture(&layout);
                    GpuShaderStageFlags::Fragment as u8
                }
            };

            entry.visibility(visibility as u32);

            bind_group_layouts[uniform.set].push(entry);
            bind_groups_dynamic_indices[uniform.set].append(&mut dynamic_indices);
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

        let mut bind_groups = (0..WEBGPU_BIND_GROUP_COUNT)
            .map(|_| vec![])
            .collect::<Vec<_>>();

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
                ShaderUniformType::DynamicUniformBuffer(ubo_id) => {
                    let ubo = context.dyn_ubos.get(ubo_id.id()).ok_or(gpu_api_err!(
                        "program dynamic uniform buffer id {:?} does not exist",
                        ubo_id
                    ))?;
                    let mut buffer = GpuBufferBinding::new(&ubo.buffer.buffer);
                    buffer.offset(0.0).size(ubo.per_index_offset as f64);
                    entry.resource(&buffer);
                }
                ShaderUniformType::ShaderStorageBuffer(ssbo_id)
                | ShaderUniformType::ShaderStorageBufferReadOnly(ssbo_id) => {
                    let ssbo = context.ssbos.get(ssbo_id.id()).ok_or(gpu_api_err!(
                        "program shader storage buffer id {:?} does not exist",
                        ssbo_id
                    ))?;
                    let buffer = GpuBufferBinding::new(&ssbo.buffer);
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

            bind_groups[uniform.set].push(entry);

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
                let bind_group_info = GpuBindGroupDescriptor::new(&entries, layout);
                context.device.create_bind_group(&bind_group_info)
            })
            .collect::<Vec<_>>();

        Ok(WebGpuBindGroups {
            bind_groups,
            bind_group_layouts,
            bind_groups_dynamic_indices,
        })
    }

    pub fn cmd_render_bind_groups(
        &self,
        context: &WebGpuContext,
        pass_encoder: &GpuRenderPassEncoder,
        dynamic_indices: &HashMap<DynamicGenericBufferId, usize>,
    ) -> GResult<()> {
        self.bind_groups
            .iter()
            .enumerate()
            .try_for_each(|(slot_idx, bind_group)| {
                let offsets = self.cmd_bind_generic(context, dynamic_indices, slot_idx)?;
                pass_encoder.set_bind_group_with_u32_sequence(
                    slot_idx as u32,
                    bind_group,
                    &offsets,
                );

                Ok(())
            })
    }

    pub fn cmd_compute_bind_groups(
        &self,
        context: &WebGpuContext,
        pass_encoder: &GpuComputePassEncoder,
        dynamic_indices: &HashMap<DynamicGenericBufferId, usize>,
    ) -> GResult<()> {
        self.bind_groups
            .iter()
            .enumerate()
            .try_for_each(|(slot_idx, bind_group)| {
                let offsets = self.cmd_bind_generic(context, dynamic_indices, slot_idx)?;
                pass_encoder.set_bind_group_with_u32_sequence(
                    slot_idx as u32,
                    bind_group,
                    &offsets,
                );

                Ok(())
            })
    }

    fn cmd_bind_generic(
        &self,
        context: &WebGpuContext,
        dynamic_indices: &HashMap<DynamicGenericBufferId, usize>,
        slot_idx: usize,
    ) -> GResult<Array> {
        let bind_group_dynamic_indices = &self.bind_groups_dynamic_indices[slot_idx];
        let offsets = Array::new();

        dynamic_indices
            .iter()
            .try_for_each(|(id, index)| match id {
                DynamicGenericBufferId::Uniform(id) => {
                    if bind_group_dynamic_indices.iter().any(|p| match p {
                        DynamicGenericBufferId::Uniform(p) => p == id,
                    }) {
                        offsets.push(&JsValue::from(
                            *index * context.dyn_ubos.get(id.id()).unwrap().per_index_offset,
                        ));
                    }
                    Ok(())
                }
            })?;

        Ok(offsets)
    }
}
