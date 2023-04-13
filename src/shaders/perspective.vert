#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Data {
    mat4 transformation;
} uniforms;

void main() {
    gl_Position = uniforms.transformation * vec4(position, 1.0);
}