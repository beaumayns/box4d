#version 450
#pragma shader_stage(fragment)

layout(std140, set = 0, binding = 0) uniform Sprite {
    vec2 scale;
    vec2 position;
    vec4 tint;
};

layout(set = 0, binding = 1) uniform texture2D sprite_texture;
layout(set = 0, binding = 2) uniform sampler sprite_sampler;

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 final_color;

void main() {
    vec4 color = texture(sampler2D(sprite_texture, sprite_sampler), uv);
    final_color = vec4(mix(color.xyz, tint.xyz, tint.w), color.w);
}
