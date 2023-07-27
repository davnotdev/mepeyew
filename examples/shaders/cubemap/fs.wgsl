@group(1) @binding(0) var my_texture: texture_cube<f32>;
@group(1) @binding(1) var my_sampler: sampler;

@fragment
fn main(
    @location(0) texture_coord: vec3<f32>
) -> @location(0) vec4<f32> {
    return textureSample(my_texture, my_sampler, texture_coord);
}
