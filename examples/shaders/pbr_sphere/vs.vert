#version 460

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;

layout(location = 4) out vec3 out_world_position;
layout(location = 5) out vec3 out_normal;

layout(set = 0, binding = 0) uniform SceneData {
    vec3 camera_position;
    vec4 light_positions[4];
    vec4 light_colors[4];
    mat4 view;
    mat4 projection;
};

layout(set = 1, binding = 0) uniform ObjectData {
    vec3 albedo;
    float metallic;
    vec3 _p1;
    float roughness;
    vec3 _p2;
    float ao;
    mat4 model;
    mat4 normal_matrix;
};

void main() {
    out_world_position = vec3(model * vec4(position, 1.0));
    out_normal = mat3(normal_matrix) * normal;

    gl_Position = projection * view * vec4(out_world_position, 1.0);
}
