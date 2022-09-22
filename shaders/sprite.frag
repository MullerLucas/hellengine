#version 450

//layout(set = 0, binding = 1) uniform sampler2D texture_sampler;
layout(set = 1, binding = 0) uniform sampler2D texture_sampler;

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec2 in_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    // out_color = in_color;
    // out_color = vec4(in_tex_coord, 0.0, 1.0);
    out_color = texture(texture_sampler, in_tex_coord * 2.0);
}
