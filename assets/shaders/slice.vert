#version 450
#pragma shader_stage(vertex)

layout(location = 0) out vec4 world_position;
layout(location = 1) out vec4 normal;
layout(location = 2) out vec4 color;

struct Mix {
    uint a;
    uint b;
    uint p1; // padding, since array elements in std140 layout are 16-btye aligned
    uint p2; // padding
};

layout(std140, set = 0, binding = 0) uniform MixtableUniform {
    // naga GLSL front end is incapable of parsing multidimensional arrays,
    // so we'll index this manually
    Mix[567] mixtable;
};

struct Camera {
    mat4 view;
    vec4 position;
    mat4 projection;
};

layout(std140, set = 1, binding = 0) uniform CameraUniform {
    Camera camera;
};

struct Tetrahedron {
    mat4 positions;
    mat4 normals;
    mat4 colors;
};

layout(std140, set = 2, binding = 0) buffer readonly Tetrahedra {
    Tetrahedron[] tetrahedra;
};

struct Transforms {
    mat4 linear;
    vec4 translation;
};

layout(std140, set = 2, binding = 1) uniform TransformUniform {
    Transforms transforms;
};

void main()
{
    Tetrahedron tetrahedron = tetrahedra[gl_VertexIndex / 7];

    // get world positions of each vertex of tetrahedron
    mat4 world_positions = mat4(transforms.translation, transforms.translation, transforms.translation, transforms.translation) + transforms.linear * tetrahedron.positions;

    // dot product of the component of the camera orientation facing into the 4th dimension with the camera-space positions of the vertices
    vec4 dots = transpose(world_positions - mat4(camera.position, camera.position, camera.position, camera.position)) * transpose(camera.view)[3];

    // Equivalent to mixtable[tetrahedral situation][vertex id] if this was a 2D array
    Mix m = mixtable[(7 * uint(dot(1 + sign(dots), vec4(27, 9, 3, 1)))) + (gl_VertexIndex % 7)];

    float s = (m.a == m.b) ? 1 : dots[m.a] / (dots[m.a] - dots[m.b]);

    world_position = mix(world_positions[m.a], world_positions[m.b], s);
    normal = mix(tetrahedron.normals[m.a], tetrahedron.normals[m.b], s);
    color = mix(tetrahedron.colors[m.a], tetrahedron.colors[m.b], s);
    gl_Position = camera.projection * vec4((camera.view * (world_position - camera.position)).xyz, 1.0);
}
