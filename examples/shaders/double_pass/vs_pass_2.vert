#version 450

layout(location = 0) in vec3 a_pos;

void main() {

    gl_Position = vec4(a_pos, 1.0);

}

