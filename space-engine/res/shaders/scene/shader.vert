#version 460

layout(row_major, push_constant) uniform PushConstants {
    mat4 mvp;
    vec4 color;
};

layout(location = 0) out vec4 faceColor;

vec3 vertices[8] = vec3[](
    vec3(-0.5, -0.5, -0.5),
    vec3(0.5, -0.5, -0.5),
    vec3(0.5, 0.5, -0.5),
    vec3(-0.5, 0.5, -0.5),
    vec3(-0.5, -0.5, 0.5),
    vec3(0.5, -0.5, 0.5),
    vec3(0.5, 0.5, 0.5),
    vec3(-0.5, 0.5, 0.5)
);

uint indices[36] = uint[](
    0, 1, 2,
    2, 3, 0,
    4, 0, 3,
    3, 7, 4,
    1, 5, 6,
    6, 2, 1,
    3, 2, 6,
    6, 7, 3,
    4, 5, 1,
    1, 0, 4,
    5, 4, 7,
    7, 6, 5
);

vec4 faceColors[6] = vec4[](
    vec4(1.0, 0.0, 0.0, 1.0),
    vec4(0.0, 1.0, 0.0, 1.0),
    vec4(0.0, 0.0, 1.0, 1.0),
    vec4(1.0, 1.0, 0.0, 1.0),
    vec4(1.0, 0.0, 1.0, 1.0),
    vec4(0.0, 1.0, 1.0, 1.0)
);

void main() {
    vec3 position = vertices[indices[gl_VertexIndex]];
    gl_Position = mvp * vec4(position, 1);
    uint face = gl_VertexIndex / 6u;
    faceColor = faceColors[face] * color;
}