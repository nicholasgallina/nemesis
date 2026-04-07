#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 frag_normal;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 vp;
} ubo;

layout(push_constant) uniform Push {
    mat4 transform;
} push;


void main() {
    gl_Position = ubo.vp * push.transform * vec4(position, 1.0);
    frag_normal = mat3(push.transform) * vec3(normal);
}