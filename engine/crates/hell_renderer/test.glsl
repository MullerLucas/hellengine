#ifdef VERTEX_PROGRAM

#version 450

layout(binding = 0) uniform UniformBufferObject {
	mat4 model;
	mat4 view;
	mat4 proj;
} ubo;


layout(location = 0) in vec4 in_pos;
layout(location = 1) in vec4 in_color;
layout(location = 2) in vec2 in_tex_coord;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec2 frag_tex_coord;


void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * in_pos;
    frag_color = in_color;
	frag_tex_coord = in_tex_coord;
}


#endif





#ifdef FRAGMENT_PROGRAM

#version 450

layout(binding = 1) uniform sampler2D tex_sampler;

layout(location = 0) in vec4 frag_color;
layout(location = 1) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_color;


void main() {
    // out_color = vec4(frag_tex_coord, 0.0, 1.0);
    out_color = frag_color * vec4(texture(tex_sampler, frag_tex_coord));
}


#endif
