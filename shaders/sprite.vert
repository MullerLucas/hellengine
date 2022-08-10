#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec4 in_pos;
layout(location = 1) in vec4 in_color;
layout(location = 2) in vec2 in_tex_coord;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_tex_coord;


void main() {
    // gl_Position = in_pos;
    gl_Position = ubo.proj * ubo.view * ubo.model * in_pos;
    out_color = in_color;
    out_tex_coord = in_tex_coord;
}
