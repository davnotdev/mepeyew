struct UniformData {
    model : mat4x4<f32>,
    view : mat4x4<f32>,
    projection : mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> mvp: UniformData;

struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) texture_coord : vec3<f32>,
}

@vertex
fn main(
    @location(0) position : vec3<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = mvp.projection * mvp.view * mvp.model * vec4<f32>(position, 1.0);
    output.texture_coord = position;
    return output;
}
