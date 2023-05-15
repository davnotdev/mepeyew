#version 450

layout(location = 0) in vec2 tex_coord;

layout(location = 0) out vec4 o_color;

layout(input_attachment_index = 0, binding = 0) uniform subpassInput u_pass_color;

void main() {

    vec4 color = subpassLoad(u_pass_color).rgba;
    o_color = color;

}

