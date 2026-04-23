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
    float radiusScale = 0.018;

    float t = smoothstep_ease(push.transition_t);

    float bns_radius = atomRadius * radiusScale * 0.8;
    float sf_radius  = atomRadius * radiusScale;
    float finalRadius = mix(bns_radius, sf_radius, t);

    vec3 worldPos = atomPos * worldScale;
    vec3 sphereVertex = inPosition * finalRadius;

    gl_Position = ubo.vp * vec4(sphereVertex + worldPos, 1.0);
    fragColor = color;
    fragNormal = normalize(inPosition);
    fragWorldPos = sphereVertex + worldPos;
}