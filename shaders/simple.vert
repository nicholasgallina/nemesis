#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 transform;
} ubo;

layout(push_constant) uniform Push {
    vec2 offset;
    vec2 scale;
} push;


void main() {
    gl_Position = ubo.transform * vec4(position, 1.0);
}