#version 450
#pragma shader_stage(vertex)

layout(std140, set = 0, binding = 0) uniform Sprite {
    vec2 scale;
    vec2 position;
    vec4 tint;
};

layout(location = 0) out vec2 uv;

void main()
{
    uv = vec2((gl_VertexIndex >> 1) & 1, gl_VertexIndex & 1);

    gl_Position = vec4(((uv * 2) - 1) * scale, 0.0, 1.0);
}
