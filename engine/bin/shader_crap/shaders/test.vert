// 460 allows us to use gl_BaseInstance
#version 460


layout(set = 0, binding = 0) uniform GlobalUbo {
    mat4 view;
    mat4 proj;
    mat4 view_proj;
    mat4 reserved_0;
} global_ubo;


struct LocalUbo {
    mat4 model;
};

// std140 enforces cpp memory layout
layout(std140, set = 3, binding = 0) readonly buffer LocalStorage {
    LocalUbo data[];
} local_storage;

layout(push_constant) uniform PushConstants {
    uint local_idx;
} push_constants;


layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 in_tex_coord;

layout(location = 0) out vec2 out_tex_coord;



void main() {
    // gl_Position = global_ubo.view_proj * push_constants.model *  vec4(in_pos, 1.0);
    mat4 model = local_storage.data[push_constants.local_idx].model;
    gl_Position = global_ubo.view_proj * model *  vec4(in_pos, 1.0);

    // gl_Position = global_ubo.view_proj * push_constants.model * vec4(in_pos, 1.0);
}
