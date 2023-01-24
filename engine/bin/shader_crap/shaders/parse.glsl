// 460 allows us to use gl_BaseInstance
#version 460

Scope: Global
{

}


scope: global
{
    struct GlobalUbo {
        mat4 view;
        mat4 proj;
        mat4 view_proj;
        mat4 reserved_0;
    } global;

    sampler2D global_tex;
}

scope: local
{
    uniform LocalUbo {
        mat4 model;
    } local_ubo;
}


shader: vert
{
    #hell use-scope global.global
    #hell use-scope global.global_tex

    layout in vec3 in_pos;
    layout in vec2 in_tex_coord;
    layout out vec2 out_tex_coord;

    void main() {
        // gl_Position = global_ubo.view_proj * push_constants.model *  vec4(in_pos, 1.0);
        mat4 model = local_storage.data[push_constants.local_idx].model;
        gl_Position = global_ubo.view_proj * model *  vec4(in_pos, 1.0);
    }
}

shader: frag
{
    layout in vec2 in_tex_coord;
    layout out vec4 out_color;

    void main() {
        out_color = texture(instance_tex, in_tex_coord);
        out_color *= shared_ubo.shared_color;
        out_color *= instance_ubo.instance_color;
        // out_color *= local_ubo.local_color;
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }
}
