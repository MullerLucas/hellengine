	
// START: sampler 'global_tex'
sampler2D global_tex;
// END: sampler 'global_tex'

// START: buffer 'instance_ubo'
layout(set = 0, binding = 0) uniform instance_ubo_buffer_type {
	float foo;
	mat3 bar;
} instance_ubo;
// END: buffer 'instance_ubo'

// START: sampler 'instance_tex'
sampler2D instance_tex;
// END: sampler 'instance_tex'

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