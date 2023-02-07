	
// START: buffer 'global_ubo'
layout(set = 0, binding = 0) uniform global_ubo_buffer_type {
	vec4 color;
	mat2 model;
} global_ubo;
// END: buffer 'global_ubo'

// START: sampler 'instance_tex'
sampler2D instance_tex;
// END: sampler 'instance_tex'

// START: code
layout(location = 0) in vec3 in_pos;
    layout(location = 1) in vec2 in_tex_coord;
    layout(location = 0) out vec2 out_tex_coord;

    void main() {
        // gl_Position = global_ubo.view_proj * push_constants.model *  vec4(in_pos, 1.0);
        mat4 model = local_storage.data[push_constants.local_idx].model;
        gl_Position = global_ubo.view_proj * model *  vec4(in_pos, 1.0);
        // gl_Position = global_ubo.view_proj * push_constants.model * vec4(in_pos, 1.0);
    }
// END: code