#version 460 core
layout(location = 0) in vec3 vert_in;
layout(location = 1) uniform float n_frac;
layout(location = 0) out float magnitude;
void main()
{
    magnitude = vert_in.z;

    float x = mod(vert_in.x + 1.0 - n_frac, 1.0);
    float y = log2(vert_in.y * 0.50 + 0.008) * 0.15 + 1.0;

    // gl_Position = vec4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 2.0 - min(1.0, magnitude / 50.0));
    gl_Position = vec4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    // gl_Position = vec4((vert_in.x + 1.0 - n_frac) * 2.0 - 1.0, vert_in.y * 2.0 - 1.0, 0.0, 1.0);
}