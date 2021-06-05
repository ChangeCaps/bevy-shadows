#version 450

layout(location = 0) in vec3 Vertex_Position;

layout(set = 0, binding = 0) uniform ViewProj {
    mat4 view_proj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 model;
};

void main() {
    gl_Position = view_proj * model * vec4(Vertex_Position, 1.0);
}