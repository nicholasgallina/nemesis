#version 450

layout(location = 0) in vec3 frag_normal;
layout(location = 0) out vec4 out_color;

void main() {
    vec3 light_dir = normalize(vec3(1.0, -3.0, -1.0));
    float diffuse = max(dot(normalize(frag_normal), -light_dir), 0.0);
    float ambient = 0.1;
    float brightness = ambient + diffuse;
    out_color = vec4(vec3(brightness), 1.0);
}