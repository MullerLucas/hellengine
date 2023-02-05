	layout(location = 0) in vec2 in_tex_coord;
    layout(location = 0) out vec4 out_color;

    void main() {
        out_color = texture(instance_tex, in_tex_coord);
        out_color *= shared_ubo.shared_color;
        out_color *= instance_ubo.instance_color;
        // out_color *= local_ubo.local_color;
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    }