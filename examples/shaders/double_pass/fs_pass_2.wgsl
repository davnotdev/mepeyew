@group(0) @binding(0) var pass_color: texture_2d<f32>;

@fragment
fn main(
    @builtin(position) coords: vec4<f32>,
) -> @location(0) vec4<f32> {
    return textureLoad(
        pass_color,
        vec2<i32>(floor(coords.xy)),
        0
    ).xyzw;
}
