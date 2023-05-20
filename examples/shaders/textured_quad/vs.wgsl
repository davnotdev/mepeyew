struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) texture_coord : vec2<f32>,
}

@vertex
fn main(
    @location(0) position : vec3<f32>,
    @location(1) texture_coord : vec2<f32>
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 1.0);
    output.texture_coord = texture_coord;
    return output;
}
