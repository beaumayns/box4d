#version 450
#pragma shader_stage(fragment)

struct Camera {
    mat4 view;
    vec4 position;
    mat4 projection;
};

layout(std140, set = 1, binding = 0) uniform CameraUniform {
    Camera camera;
};

layout(std140, set = 2, binding = 1) uniform State {
    uint contacts;
    uint hollow;
    uvec2 padding;
};

const vec4 light_position = vec4(2, 2, 0, 0);
const vec3 light_color = 5 * vec3(0.5, 0.5, 0.5);
const vec3 ambient_light = 4 * vec3(0.4, 0.4, 0.45);

const float albedo = 0.7f;
const float shininess = 40.0f;

layout(location = 0) in vec4 world_position;
layout(location = 1) in vec4 normal;
layout(location = 2) in vec4 color;
layout(location = 3) in vec3 barycentric;

layout(location = 0) out vec4 final_color;

void main() {
    vec4 n = normalize(normal);

    vec4 L = normalize(light_position - world_position);
    vec4 R = normalize(reflect(-L, n));
    vec4 V = normalize(camera.position - world_position);
    vec4 H = normalize(L + V);

    vec3 ambient = albedo * ambient_light;
    vec3 diffuse = albedo * light_color * max(dot(L, n), 0.0);
    vec3 specular = light_color * pow(clamp(dot(n, H), 0.0, 1.0), shininess);

    final_color = vec4(pow((ambient + diffuse + specular) * color.xyz, vec3(2.2)), color.w);

    vec3 a3 = step(2.0 * fwidth(barycentric), barycentric);
    if(contacts > 0 || hollow != 0) {
        float m = min(min(a3.x, a3.y), a3.z);
        if(m > 0.0 && hollow != 0) {
            discard;
        }

        vec3 outlineColor = vec3(0.0, contacts > 0 ? 1.0 : 0.0, 0.0);
        final_color = vec4(mix(mix(vec3(0), outlineColor, 1.0), final_color.xyz, m), final_color.w);
    }
}
