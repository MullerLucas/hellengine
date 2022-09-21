#version 450

layout(binding = 0) uniform UBOCamera {
//    mat4 model;
    mat4 view;
    mat4 proj;
    mat4 view_proj;
} camera_data;

layout(location = 0) in vec4 in_pos;
layout(location = 1) in vec4 in_color;
layout(location = 2) in vec2 in_tex_coord;

layout(push_constant) uniform constants {
    vec4 data;
    mat4 model_mat;
} PushConstants;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_tex_coord;



void main() {
//     gl_Position = in_pos;
//    gl_Position = ubo.proj * ubo.view * ubo.model * in_pos;

    mat4 transform_mat = (camera_data.view_proj * PushConstants.model_mat);
    gl_Position = transform_mat * in_pos;

//    gl_Position = PushConstants.model_mat * in_pos;

    out_color = in_color;
    out_tex_coord = in_tex_coord;
}
