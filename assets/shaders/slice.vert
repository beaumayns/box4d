#version 450
#pragma shader_stage(vertex)

layout(location = 0) in vec4 position0;
layout(location = 1) in vec4 position1;
layout(location = 2) in vec4 position2;
layout(location = 3) in vec4 position3;
layout(location = 4) in vec4 normal0;
layout(location = 5) in vec4 normal1;
layout(location = 6) in vec4 normal2;
layout(location = 7) in vec4 normal3;
layout(location = 8) in vec4 color0;
layout(location = 9) in vec4 color1;
layout(location = 10) in vec4 color2;
layout(location = 11) in vec4 color3;

layout(location = 0) out vec4 world_position;
layout(location = 1) out vec4 normal;
layout(location = 2) out vec4 color;
layout(location = 3) out vec3 barycentric;

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

struct Transforms {
    mat4 linear;
    vec4 translation;
};

layout(std140, set = 2, binding = 0) uniform TransformUniform {
    Transforms transforms;
};

void main()
{
    mat4 positions = mat4(position0, position1, position2, position3);
    mat4 colors = mat4(color0, color1, color2, color3);
    mat4 normals = mat4(normal0, normal1, normal2, normal3);

    // get world positions of each vertex of tetrahedron
    mat4 world_positions = mat4(transforms.translation, transforms.translation, transforms.translation, transforms.translation) + transforms.linear * positions;

    // dot product of the component of the camera orientation facing into the 4th dimension with the camera-space positions of the vertices
    vec4 dots = transpose(world_positions - mat4(camera.position, camera.position, camera.position, camera.position)) * transpose(camera.view)[3];

    // Equivalent to mixtable[tetrahedral situation][vertex id] if this was a 2D array
    Mix m = mixtable[(7 * uint(dot(1 + sign(dots), vec4(27, 9, 3, 1)))) + (gl_VertexIndex % 7)];

    float s = (m.a == m.b) ? 1 : dots[m.a] / (dots[m.a] - dots[m.b]);

    world_position = mix(world_positions[m.a], world_positions[m.b], s);
    normal = mix(normals[m.a], normals[m.b], s);
    color = mix(colors[m.a], colors[m.b], s);
    barycentric = vec3(0);
    barycentric[gl_VertexIndex % 3] = 1;
    gl_Position = camera.projection * vec4((camera.view * (world_position - camera.position)).xyz, 1.0);
}
