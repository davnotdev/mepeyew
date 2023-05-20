#version 450

layout(location = 0) in vec2 tex_coord;

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0) uniform texture2D photo;
layout(set = 0, binding = 1) uniform sampler photo_sampler;

void main() {

    o_color = texture(sampler2D(photo, photo_sampler), tex_coord);

}

