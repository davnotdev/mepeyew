struct UniformData {
    view : mat4x4<f32>,
    projection : mat4x4<f32>,
    model : array<mat4x4<f32>, 25>,
}

@group(0) @binding(0) var<uniform> mvp: UniformData;
struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) color : vec3<f32>,
}

@vertex
fn main(
    @builtin(instance_index) instance_idx : u32,
    @location(0) position : vec3<f32>,
    @location(1) color : vec3<f32>
) -> VertexOutput {
    var output: VertexOutput;
    output.position = mvp.projection * mvp.view * mvp.model[instance_idx] * vec4<f32>(position, 1.0);
    output.color = color;
    return output;
}
