#version 460 core
layout(location = 0) in vec2 aPos;
layout(location = 1) uniform float xOff;
void main() {
    gl_Position = vec4(aPos.x + xOff, aPos.y * 0.05 + 0.95, 0.0, 1.0);
}