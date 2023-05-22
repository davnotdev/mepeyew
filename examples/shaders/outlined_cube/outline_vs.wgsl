struct UniformData {
    model : mat4x4<f32>,
    view : mat4x4<f32>,
    projection : mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> mvp: UniformData;

@vertex
fn main(
    @location(0) position : vec3<f32>,
) -> @builtin(position) vec4<f32> {
    return mvp.projection * mvp.view * mvp.model * vec4<f32>(position, 1.0);
}
