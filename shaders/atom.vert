#version 450

// geometry
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 normal;

// per-instance
layout(location = 2) in vec3 atomPos;
layout(location = 3) in float atomRadius;
layout(location = 4) in vec3 color;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 vp;
} ubo;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = ubo.vp * vec4(inPosition * atomRadius + atomPos, 1.0);
    fragColor = color;
}