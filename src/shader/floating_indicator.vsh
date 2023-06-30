#version 460 core
layout(location = 0) in vec2 aPos;
layout(location = 1) uniform float aspect;
void main() {
    gl_Position = vec4(aPos.x, (aPos.y + 1.0) / aspect - 1.0, 0.0, 1.0);
}