#version 460

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

layout(location = 4) in vec3 world_position;
layout(location = 5) in vec3 normal;

layout(location = 0) out vec4 out_color;

const float PI = 3.14159265359;

float distribution_ggx(vec3 n, vec3 h, float roughness)
{
    float a = roughness * roughness;
    float a2 = a * a;
    float ndoth = max(dot(n, h), 0.0);
    float ndoth2 = ndoth * ndoth;

    float num = a2;
    float denom = (ndoth2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float geometry_schlick_ggx(float ndotv, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float num = ndotv;
    float denom = ndotv * (1.0 - k) + k;

    return num / denom;
}

float geometry_smith(vec3 n, vec3 v, vec3 l, float roughness) {
    float ndotv = max(dot(n, v), 0.0);
    float ndotl = max(dot(n, l), 0.0);
    float res1 = geometry_schlick_ggx(ndotl, roughness);
    float res2 = geometry_schlick_ggx(ndotv, roughness);
    return res1 * res2;
}

vec3 fresnel_schlick(float cos_theta, vec3 f0) {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

void main() {
    vec3 n = normalize(normal);
    vec3 v = normalize(camera_position - world_position);

    vec3 f0 = vec3(0.04);
    f0 = mix(f0, albedo, metallic);

    vec3 l_out = vec3(0.0);

    for(int i = 0; i < 4; i++) {
        vec3 light_color = vec3(light_colors[i]);
        vec3 light_position = vec3(light_positions[i]);

        vec3 l = normalize(light_position - world_position);
        vec3 h = normalize(v + l);
        float distance = length(light_position - world_position);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance = light_color * attenuation;

        float ndf = distribution_ggx(n, h, roughness);
        float g = geometry_smith(n, v, l, roughness);
        vec3 f = fresnel_schlick(clamp(dot(h, v), 0.0, 1.0), f0);

        vec3 num = f * ndf * g;
        float denom = 4.0 * max(dot(n, v), 0.0) * max(dot(n, l), 0.0) + 0.0001;
        vec3 specular = num / denom;

        vec3 ks = f;
        vec3 kd = vec3(1.0) - ks;
        kd *= 1.0 - metallic;

        float ndotl = max(dot(n, l), 0.0);

        l_out += (kd * albedo / PI + specular) * radiance * ndotl;
    }
    vec3 ambient = vec3(0.03) * albedo * ao;

    l_out = l_out / (l_out + vec3(1.0));
    l_out = pow(l_out, vec3(1.0 / 2.2));

    out_color = vec4(l_out, 1.0);
}
