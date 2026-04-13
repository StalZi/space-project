#version 460

struct UIObjectData {
    vec4 position_size; // x, y, width, height
    vec4 color; // r, g, b, a
};

layout(std430, set = 0, binding = 0) readonly buffer UIObjects {
    UIObjectData data[];
};

layout(push_constant) uniform PushConstants {
    vec2 viewport_size; // width, height
};

layout(location = 0) out vec4 fragColor;

vec2 vertex_positions[4] = vec2[](
    vec2(0.0, 0.0), // top-left
    vec2(1.0, 0.0), // top-right
    vec2(0.0, 1.0), // bottom-left
    vec2(1.0, 1.0)  // bottom-right
);

void main() {
    UIObjectData obj = data[gl_InstanceIndex];
    vec2 pos = obj.position_size.xy + obj.position_size.zw * vertex_positions[gl_VertexIndex];
    gl_Position = vec4(2.0 * (pos.x / viewport_size.x) - 1.0, -(2.0 * (pos.y / viewport_size.y) - 1.0), 0.0, 1.0);
    gl_Position.y = -gl_Position.y; 
    fragColor = obj.color;
}