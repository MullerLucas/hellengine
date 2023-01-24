#version 460

layout(set = 1, binding = 0) uniform SharedUbo {
    vec4 shared_color;
} shared_ubo;

layout(set = 0, binding = 1) uniform sampler2D global_tex;

layout(set = 2, binding = 0) uniform InstanceUbo {
    vec4 instance_color;
} instance_ubo;

layout(set = 2, binding = 1) uniform sampler2D instance_tex;

// layout(set = 3, binding = 0) uniform LocalUbo {
//     vec4 local_color;
// } local_ubo;


layout(location = 0) in vec2 in_tex_coord;
layout(location = 0) out vec4 out_color;

void main() {
    out_color = texture(instance_tex, in_tex_coord);
    out_color *= shared_ubo.shared_color;
    out_color *= instance_ubo.instance_color;
    // out_color *= local_ubo.local_color;
    out_color = vec4(1.0, 0.0, 0.0, 1.0);
}
