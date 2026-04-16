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
    float worldScale = 0.05;
    float radiusScale = 0.06;
    vec3 worldPos = atomPos * worldScale;
    vec3 sphereVertex = inPosition * (atomRadius * radiusScale);
    gl_Position = ubo.vp * vec4(sphereVertex + worldPos, 1.0);
    fragColor = color;
}