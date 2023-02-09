	

// ERROR: failed to write uniform 'GLOBAL::global_tex'


// ERROR: failed to write uniform 'SHARED::shared_tex'

// START: buffer 'instance_ubo'
layout(set = 2, binding = 0) uniform instance_ubo_buffer_type {
	float foo;
	mat3 bar;
	mat2 moo;
	mat2 glatz;
} instance_ubo;
// END: buffer 'instance_ubo'


// ERROR: failed to write uniform 'INSTANCE::instance_tex'


// ERROR: failed to write uniform 'LOCAL::local_ubo'

// START: code
layout(location = 0) in vec2 in_tex_coord;
    layout(location = 0) out vec4 out_color;

    void main() {
        out_color = texture(instance_tex, in_tex_coord);
        out_color *= shared_ubo.shared_color;
        out_color *= instance_ubo.instance_color;
        // out_color *= local_ubo.local_color;
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
// END: code