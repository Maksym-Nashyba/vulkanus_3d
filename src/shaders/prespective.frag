#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(gl_Position.xyz, 1.0);
}