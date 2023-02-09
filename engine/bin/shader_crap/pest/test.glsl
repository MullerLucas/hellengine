#INFO
{
    version = 0.1.0;
    name = "testing/test_shader";
    pass = "ui";
}

#SCOPE: GLOBAL
{
    uniform buffer {
        vec4 color;
        mat2 model;
    } global_ubo;

    uniform sampler2D global_tex_0;
    uniform sampler2D global_tex_1;
    uniform sampler2D global_tex_2;
    uniform sampler2D global_tex_3;
    uniform sampler2D global_tex_4;
}

#SCOPE: SHARED
{
    uniform buffer {
        float foo;
    } shared_ubo;

    uniform sampler2D shared_tex_0;
    uniform sampler2D shared_tex_1;
    uniform sampler2D shared_tex_2;
}

#SCOPE: INSTANCE
{
    uniform buffer {
        float foo;
        mat3 bar;
        mat2 moo;
        mat2 glatz;
    } instance_ubo;

    uniform sampler2D instance_tex_0;
    uniform sampler2D instance_tex_1;
    uniform sampler2D instance_tex_2;
}

#SCOPE: LOCAL
{
    uniform buffer {
        mat4 model;
        mat4 view;
        mat4 view_model;
    } local_ubo;
}

#SHADER: VERT
{
    uniform GLOBAL::global_ubo;
    uniform GLOBAL::global_tex_0;
    uniform GLOBAL::global_tex_1;
    uniform GLOBAL::global_tex_2;
    uniform GLOBAL::global_tex_3;

    uniform SHARED::shared_ubo;
    uniform SHARED::shared_tex_0;
    uniform SHARED::shared_tex_1;
    uniform SHARED::shared_tex_2;

    uniform INSTANCE::instance_ubo;
    uniform INSTANCE::instance_tex_0;
    uniform INSTANCE::instance_tex_1;
    uniform INSTANCE::instance_tex_2;

    uniform LOCAL::local_ubo;

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
    uniform GLOBAL::global_tex_1;
    uniform SHARED::shared_tex_1;
    uniform SHARED::shared_tex_2;
    uniform INSTANCE::instance_ubo;
    uniform INSTANCE::instance_tex_2;
    uniform LOCAL::local_ubo;

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
