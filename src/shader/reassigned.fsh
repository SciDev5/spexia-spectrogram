#version 460 core
out vec4 FragColor;

layout(location = 0) in float magnitude;

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
vec3 heatmap(float x) {
    float k = clamp(x, 0.0, 1.0);// max(0.0,min(1.0,fac));
    float h = mod((0.6 - 0.75 * pow(k, 4.0) + 1.0), 1.0);
    float s = 1.0 - pow(k, 7.0);
    float v = pow(k, 0.7);
    return hsv2rgb(vec3(h, s, v));
}

void main() {

    float x = log(magnitude * 0.9 + 0.1) / 4.0;

    vec3 col = heatmap(x);

    FragColor = vec4(col, magnitude * 0.4);// vec4(magnitude * 0.01, magnitude * 0.1, magnitude * 1.0, 1.0);
}

// // vec3 heatmap(float x) {
// //     vec3 col = 0.8/(vec3(1.0,1.0,1.0)+exp(vec3(3.0,4.0,8.0)*(0.5 - x)-vec3(1.0,0.2,-0.5)));
// //     col += 0.2/(vec3(1.0,1.0,1.0)+exp(vec3(1.5,4.0,3.0)*(0.5 - x)-vec3(2.0,1.2,5.5)));

// //     col -= vec3(0.2,0.2,0.1)*(1.0 - 1.0/(1.0+exp((2.0*x-2.0)*(-2.0*x+1.0))));
// //     return col;
// // }

// vec3 hsv2rgb(vec3 c)
// {
//     vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
//     vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
//     return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
// }
// vec3 heatmap(float x) {
//     float k = clamp(x, 0.0, 1.0);// max(0.0,min(1.0,fac));
//     float h = mod((0.6-0.75*pow(k,4.0)+1.0),1.0);
//     float s = 1.0-pow(k,7.0);
//     float v = pow(k,0.7);
//     return hsv2rgb(vec3(h,s,v));
// }

// float samplePower(float uvx, float k, bool side) {
//     float p = 0.0;
//     float m = 0.0;

//     const int N = 50;

//     for (int i = 0; i < N; i++) {
//         vec4 v = texture(tex, vec2(uvx, k * (1.0 + 10.0 * float(i) / float(N) / height)));
//         // vec4 v = texture(tex, vec2(uvx, k));
//         if (side) {
//             p += v.x + v.y / 256.0;
//             m = max(m, v.x + v.y / 256.0);
//         } else {
//             p += v.z + v.w / 256.0;
//             m = max(m, v.z + v.w / 256.0);
//         }
//     }

//     return p / float(N) * 0.5 + m * 0.5;
//     // return p;
// }

// void main() {
//     float uvx = uv.x;
//     bool side = false;
//     bool blank = false;
//     if (uv.x > 0.925) {
//         uvx = nFrac;
//         side = uv.x > 0.9625;
//     } else {
//         if (uv.x > 0.90) {
//             blank = true;
//         }
//         uvx = uvx + 0.1 + nFrac;
//         side = mod((uv.y * height * 0.5), 1.0) > 0.5;
//     }
//     float k = pow(2.0, 10.0 * (uv.y - 1.0));
//     // vec4 v = texture(tex, vec2(uvx, k));

//     vec3 band = vec3(0.0,0.0,0.0);
//     // float x = 0.0;
//     float x = samplePower(uvx, k, side);
//     if (side) {
//         // x = v.x + v.y / 256.0;
//         band = vec3(0.0392156862745098, 0.4392156862745098, 0.4196078431372549);
//     } else {
//         // x = v.z + v.w / 256.0;
//         band = vec3(0.27058823529411763, 0.0392156862745098, 0.4392156862745098 );
//     }

//     x = (x - 0.5) * 100.0;
//     x *= 0.35;
//     x += -0.05;
//     x = x + 2.0 / (1.0 + exp(-exp(3.0*x-2.0))) - 1.3;

//     vec3 col = vec3(0.0,0.0,0.0);
//     if (!blank) {
//         col = heatmap(x);
//     }

//     // col.b = cos(x*200.0);

//     if (uv.y > 0.95) {
//         col = band + vec3(0.0,length(col),0.0);
//     }

//     FragColor = vec4(col,1.0);
//     // FragColor = vec4(col,clamp(x/3.0+0.5,0.0,1.0));
// }

// /*

// void mainImage( out vec4 fragColor, in vec2 fragCoord )
// {
//     // Normalized pixel coordinates (from 0 to 1)
//     vec2 uv = fragCoord/iResolution.xy;

//     // Time varying pixel color
//     //vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

//     float x = ((uv.x-0.5) * 20.0 + 0.5);
//     float p = 1.0/(1.0 + exp(x));
//     float p1 = 1.0/(1.0 + exp(-3.0-x));
//     float p2 = 1.0/(1.0 + exp(0.5-x/2.0));

//     float hue = (1.0-p) * 6.9 + 1.1;
//     vec3 puretone = pow(0.5 + 0.5*cos(hue + vec3(0.0,2.0,4.0)),vec3(0.7,0.5,0.9));
//     vec3 one = vec3(1.0,1.0,1.0);
//     vec3 col = (-1.0+exp(one - (one - p1 * puretone) * (1.0 - p2)))/1.8 + vec3(0.0,0.05,0.1);

//     if (uv.y > 0.8) {
//         col = vec3(x,x,x);
//     }
//     // Output to screen
//     fragColor = vec4(col,1.0);

// }

// void mainImage( out vec4 fragColor, in vec2 fragCoord )
// {
//     // Normalized pixel coordinates (from 0 to 1)
//     vec2 uv = fragCoord/iResolution.xy;

//     // Time varying pixel color
//     //vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

//     float x = ((uv.x-0.5) * 4.0 + 0.5);

//     x = x + 2.0 / (1.0 + exp(-exp(3.0*x-2.0))) - 1.3;

//     //x = 1.0/(1.0 + exp((0.5-x)*6.0));

//     vec3 col = 0.8/(vec3(1.0,1.0,1.0)+exp(vec3(3.0,4.0,8.0)*(0.5 - x)-vec3(1.0,0.2,-0.5)));
//     col += 0.2/(vec3(1.0,1.0,1.0)+exp(vec3(1.5,4.0,3.0)*(0.5 - x)-vec3(2.0,1.2,5.5)));

//     col -= vec3(0.2,0.2,0.1)*(1.0 - 1.0/(1.0+exp((2.0*x-2.0)*(-2.0*x+1.0))));

//     if (uv.y > 0.8) {
//         col = vec3(x,x,x);
//     }
//     // Output to screen
//     fragColor = vec4(col,1.0);

// }

// */