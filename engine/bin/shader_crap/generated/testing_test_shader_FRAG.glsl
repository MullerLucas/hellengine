	
// --- START: sampler 'global_tex_1' ---
layout(set = 0, binding = 2) sampler2D global_tex_1;
// --- END: sampler 'global_tex_1' ---

// --- START: sampler 'shared_tex_1' ---
layout(set = 1, binding = 2) sampler2D shared_tex_1;
// --- END: sampler 'shared_tex_1' ---

// --- START: sampler 'shared_tex_2' ---
layout(set = 1, binding = 3) sampler2D shared_tex_2;
// --- END: sampler 'shared_tex_2' ---

// --- START: buffer 'instance_ubo' ---
layout(set = 2, binding = 0) uniform instance_ubo_buffer_type {
	float foo;
	mat3 bar;
	mat2 moo;
	mat2 glatz;
} instance_ubo;
// --- END: buffer 'instance_ubo' ---

// --- START: sampler 'instance_tex_2' ---
layout(set = 2, binding = 3) sampler2D instance_tex_2;
// --- END: sampler 'instance_tex_2' ---

// --- START: buffer 'local_ubo' ---
struct inner_local_ubo_buffer_type {
	mat4 model;
	mat4 view;
	mat4 view_model;
};
// std140 enforces cpp memory layout
layout(std140, set = 3, binding = 0) readonly buffer local_ubo_buffer_type {
    inner_inner_local_ubo_buffer_type data[];
} local_ubo;
// --- END: buffer 'local_ubo' ---

// --- START: code ---
layout(location = 0) in vec2 in_tex_coord;
    layout(location = 0) out vec4 out_color;

    void main() {
        out_color = texture(instance_tex, in_tex_coord);
        out_color *= shared_ubo.shared_color;
        out_color *= instance_ubo.instance_color;
        // out_color *= local_ubo.local_color;
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
// --- END: code ---