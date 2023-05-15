#version 450

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec3 a_color;

layout(location = 0) out vec4 o_color;

void main() {

    gl_Position = vec4(a_pos, 1.0);
    o_color = vec4(a_color, 1.0);

}

