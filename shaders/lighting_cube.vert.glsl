#version 330 core
struct Camera {
    mat4 projection;
    mat4 view;
};

layout (location = 0) in vec3 v_pos;

uniform mat4 model;
uniform Camera camera;


void main() {
    gl_Position = camera.projection * camera.view * model * vec4(v_pos, 1.0);
}
