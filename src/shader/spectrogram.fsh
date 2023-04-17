#version 460 core
out vec4 FragColor;

layout(location = 0) in vec2 uv;
layout(location = 1) uniform sampler2D tex;
layout(location = 2) uniform float nFrac;


vec3 heatmap(float x) {
    vec3 col = 0.8/(vec3(1.0,1.0,1.0)+exp(vec3(3.0,4.0,8.0)*(0.5 - x)-vec3(1.0,0.2,-0.5)));
    col += 0.2/(vec3(1.0,1.0,1.0)+exp(vec3(1.5,4.0,3.0)*(0.5 - x)-vec3(2.0,1.2,5.5)));

    col -= vec3(0.2,0.2,0.1)*(1.0 - 1.0/(1.0+exp((2.0*x-2.0)*(-2.0*x+1.0))));
    return col;
}


void main() {
    float uvx = uv.x;
    bool side = false;
    if (uv.x > 0.925) {
        uvx = nFrac;
        side = uv.x > 0.9625;
    } else if (uv.x > 0.9) {
        discard;
    } else {
        uvx = uvx + 0.1 + nFrac;
        side = mod((uv.y * 500.0), 1.0) > 0.5;
    }
    float k = pow(2.0, 10.0 * (uv.y - 1.0));
    vec4 v = texture(tex, vec2(uvx, k));

    vec3 band = vec3(0.0,0.0,0.0);
    float x = 0.0;
    if (side) {
        x = v.x + v.y / 256.0;
        band = vec3(1.0,0.0,0.0);
    } else {
        x = v.z + v.w / 256.0;
        band = vec3(0.0,0.0,1.0);
    }

    x = (x - 0.5) * 100.0;
    x *= 0.75;
    x -= 0.7;
    x = x + 2.0 / (1.0 + exp(-exp(3.0*x-2.0))) - 1.3;

    vec3 col = heatmap(x);
    
    // col.b = cos(x*200.0);
    
    if (uv.y > 0.95) {
        col = band + vec3(0.0,length(col),0.0);
    }


    FragColor = vec4(col,1.0);
    // FragColor = vec4(col,clamp(x/3.0+0.5,0.0,1.0));
}

/*


void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = fragCoord/iResolution.xy;

    // Time varying pixel color
    //vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    float x = ((uv.x-0.5) * 20.0 + 0.5);
    float p = 1.0/(1.0 + exp(x));
    float p1 = 1.0/(1.0 + exp(-3.0-x));
    float p2 = 1.0/(1.0 + exp(0.5-x/2.0));

    float hue = (1.0-p) * 6.9 + 1.1;
    vec3 puretone = pow(0.5 + 0.5*cos(hue + vec3(0.0,2.0,4.0)),vec3(0.7,0.5,0.9));
    vec3 one = vec3(1.0,1.0,1.0);
    vec3 col = (-1.0+exp(one - (one - p1 * puretone) * (1.0 - p2)))/1.8 + vec3(0.0,0.05,0.1);
    
    if (uv.y > 0.8) {
        col = vec3(x,x,x);
    }
    // Output to screen
    fragColor = vec4(col,1.0);
    
}


void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = fragCoord/iResolution.xy;

    // Time varying pixel color
    //vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    float x = ((uv.x-0.5) * 4.0 + 0.5);
    
    x = x + 2.0 / (1.0 + exp(-exp(3.0*x-2.0))) - 1.3;
    
    //x = 1.0/(1.0 + exp((0.5-x)*6.0));
    
    vec3 col = 0.8/(vec3(1.0,1.0,1.0)+exp(vec3(3.0,4.0,8.0)*(0.5 - x)-vec3(1.0,0.2,-0.5)));
    col += 0.2/(vec3(1.0,1.0,1.0)+exp(vec3(1.5,4.0,3.0)*(0.5 - x)-vec3(2.0,1.2,5.5)));

    col -= vec3(0.2,0.2,0.1)*(1.0 - 1.0/(1.0+exp((2.0*x-2.0)*(-2.0*x+1.0))));
    
    if (uv.y > 0.8) {
        col = vec3(x,x,x);
    }
    // Output to screen
    fragColor = vec4(col,1.0);
    
}



*/