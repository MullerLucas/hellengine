#INFO
{
    crap_ver = 0.1.0;
    name = "testing/test_shader";
}

#SCOPE: GLOBAL
{
    uniform buffer {
        vec4 color;
        mat2 model;
    } global;

    uniform sampler2D global_tex;
}

#SCOPE: INSTANCE
{
    uniform sampler2D instance_tex;
}

#SHADER: VERT
{
    uniform GLOBAL::global;
    uniform INSTANCE::instance_tex;

    #HELLPROGRAM

    layout(location = 0) in vec3 in_pos;
    layout(location = 1) in vec2 in_tex_coord;
    layout(location = 0) out vec2 out_tex_coord;

    void main() {
        // gl_Position = global_ubo.view_proj * push_constants.model *  vec4(in_pos, 1.0);
        mat4 model = local_storage.data[push_constants.local_idx].model;
        gl_Position = global_ubo.view_proj * model *  vec4(in_pos, 1.0);
        // gl_Position = global_ubo.view_proj * push_constants.model * vec4(in_pos, 1.0);
    }

    #ENDHELL
}

#SHADER: FRAG
{
    uniform GLOBAL::global_tex;

    #HELLPROGRAM

    layout(location = 0) in vec2 in_tex_coord;
    layout(location = 0) out vec4 out_color;

    void main() {
        out_color = texture(instance_tex, in_tex_coord);
        out_color *= shared_ubo.shared_color;
        out_color *= instance_ubo.instance_color;
        // out_color *= local_ubo.local_color;
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }

    #ENDHELL
}
