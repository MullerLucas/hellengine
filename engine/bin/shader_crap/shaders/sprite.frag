#version 460

layout(set = 1, binding = 1) uniform sampler2D instance_tex_0;

layout(location = 0) in vec2 in_tex_coord;
layout(location = 0) out vec4 out_color;



void main() {
    out_color = texture(instance_tex_0, in_tex_coord);
    // out_color = vec4(1.0, 0.0, 0.0, 1.0);
}
