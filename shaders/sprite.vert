#version 450

layout(location = 0) in vec4 in_pos;


void main() {
    gl_Position = in_pos;
}
