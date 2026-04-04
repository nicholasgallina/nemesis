#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 transform;
} ubo;

layout(push_constant) uniform Push {
    mat4 transform;
} push;


void main() {
    gl_Position = ubo.transform * push.transform * vec4(position, 1.0);
}