#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 normal;
layout(location = 5) in vec3 bondStart;
layout(location = 6) in vec3 bondEnd;
layout(location = 7) in vec3 color;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 vp;
} ubo;

layout(push_constant) uniform PushConstants {
    layout(offset = 64) float transition_t;
} push;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec3 fragWorldPos;

float smoothstep_ease(float t) {
    return t * t * (3.0 - 2.0 * t);
}

void main() {
    float worldScale = 0.05;
    float t = smoothstep_ease(push.transition_t);

    // bonds shrink to zero radius as t approaches 1.0
    float bondRadius = mix(0.012, 0.0, t);

    vec3 start = bondStart * worldScale;
    vec3 end   = bondEnd   * worldScale;
    vec3 dir   = end - start;
    float len  = length(dir);
    vec3 axis  = normalize(dir);

    vec3 tangent, bitangent;
    if (abs(axis.x) > 0.9) {
        tangent = normalize(cross(axis, vec3(0.0, 1.0, 0.0)));
    } else {
        tangent = normalize(cross(axis, vec3(1.0, 0.0, 0.0)));
    }
    bitangent = normalize(cross(axis, tangent));

    vec3 worldPos = (start + end) * 0.5
        + tangent   * inPosition.x * bondRadius
        + axis      * inPosition.y * len
        + bitangent * inPosition.z * bondRadius;

    gl_Position = ubo.vp * vec4(worldPos, 1.0);
    fragColor = color;
    fragNormal = normalize(tangent * normal.x + axis * normal.y + bitangent * normal.z);
    fragWorldPos = worldPos;
}