#version 450

layout(location = 0) in vec2 tex_coord;

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0) uniform sampler2D photo;

void main() {

    o_color = texture(photo, tex_coord);

}

