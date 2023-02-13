	
// --- START: buffer 'global_ubo' ---
layout(set = 0, binding = 0) uniform global_ubo_buffer_type {
	vec4 color;
	mat2 model;
} global_ubo;
// --- END: buffer 'global_ubo' ---

// --- START: sampler 'global_tex_0' ---
layout(set = 0, binding = 1) sampler2D global_tex_0;
// --- END: sampler 'global_tex_0' ---

// --- START: sampler 'global_tex_1' ---
layout(set = 0, binding = 2) sampler2D global_tex_1;
// --- END: sampler 'global_tex_1' ---

// --- START: sampler 'global_tex_2' ---
layout(set = 0, binding = 3) sampler2D global_tex_2;
// --- END: sampler 'global_tex_2' ---

// --- START: sampler 'global_tex_3' ---
layout(set = 0, binding = 4) sampler2D global_tex_3;
// --- END: sampler 'global_tex_3' ---

// --- START: buffer 'shared_ubo' ---
layout(set = 1, binding = 0) uniform shared_ubo_buffer_type {
	float foo;
} shared_ubo;
// --- END: buffer 'shared_ubo' ---

// --- START: sampler 'shared_tex_0' ---
layout(set = 1, binding = 1) sampler2D shared_tex_0;
// --- END: sampler 'shared_tex_0' ---

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

// --- START: sampler 'instance_tex_0' ---
layout(set = 2, binding = 1) sampler2D instance_tex_0;
// --- END: sampler 'instance_tex_0' ---

// --- START: sampler 'instance_tex_1' ---
layout(set = 2, binding = 2) sampler2D instance_tex_1;
// --- END: sampler 'instance_tex_1' ---

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
layout(location = 0) in vec3 in_pos;
    layout(location = 1) in vec2 in_tex_coord;
    layout(location = 0) out vec2 out_tex_coord;

    void main() {
        // gl_Position = global_ubo.view_proj * push_constants.model *  vec4(in_pos, 1.0);
        mat4 model = local_storage.data[push_constants.local_idx].model;
        gl_Position = global_ubo.view_proj * model *  vec4(in_pos, 1.0);
        // gl_Position = global_ubo.view_proj * push_constants.model * vec4(in_pos, 1.0);
    }
// --- END: code ---