#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;
uniform float iTime;
uniform vec2 iResolution;

vec2 CRTCurveUV(vec2 uv) {
    vec2 curvature = vec2(9.0, 9.0);
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs(uv.yx) / vec2(curvature.x, curvature.y);
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void DrawVignette(inout vec3 color, vec2 uv) {
    float intensity = 5.75;
    float roundness = 0.05;
    float feather = 0.95;

    vec2 delta = uv - vec2(0.5);

    float radialDist = length(delta);
    float axialDist = max(abs(delta.x), abs(delta.y));

    float shapeDist = mix(axialDist, radialDist, roundness);

    float falloff = pow(shapeDist, intensity);
    float vignette = 1.0 - smoothstep(0.0, 1.0 - feather, falloff);

    color *= vignette;
}

void DrawScanline(inout vec3 color, vec2 uv) {
    float width = 3.0;
    float phase = iTime / 100.0;
    float thickness = 2.6;
    float opacity = 0.2;
    vec3 lineColor = vec3(0.22, 0.25, 0.27);

    float v = 0.5 * (sin((uv.y + phase) * 3.14159 / width * iResolution.y) + 1.0);
    color.rgb -= (lineColor - color.rgb) * (pow(v, thickness) - 1.0) * opacity;
}

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

void DrawFilmGrain(inout vec3 color, vec2 uv) {
    float intensity = 0.03;
    float speed = 10.0;

    vec2 noiseCoord = uv * iResolution.xy * 0.5;
    float timeOffset = floor(iTime * speed) * 0.01;

    float noise = random(noiseCoord + timeOffset);

    noise = (noise - 0.5) * intensity;

    color.rgb += noise;
}

vec3 DrawRGBSeparation(sampler2D tex, vec2 uv) {
    float separation = 0.0002;
    // float separation = 0.0;

    float r = texture2D(tex, uv + vec2(separation, 0.0)).r;
    float g = texture2D(tex, uv).g;
    float b = texture2D(tex, uv - vec2(separation, 0.0)).b;

    return vec3(r, g, b);
}

void DrawBloom(inout vec3 color, vec2 uv) {
    float intensity = 0.3;
    float threshold = 0.7;

    float luma = dot(color, vec3(0.299, 0.587, 0.114));

    if(luma > threshold) {
        float bloom = (luma - threshold) * intensity;
        color += bloom;
    }
}

void DrawColorTemperature(inout vec3 color) {
    vec3 warmTint = vec3(1.05, 1.0, 0.95);
    color *= warmTint;

    color = (color - 0.5) * 1.1 + 0.5;
}

void DrawShadowMask(inout vec3 color, vec2 uv) {
    float intensity = 0.25;
    vec2 maskCoord = uv * iResolution.xy;

    vec3 mask = vec3(1.0);
    float x = mod(maskCoord.x, 3.0);

    if(x < 1.0) {
        mask = vec3(1.0, 0.7, 0.7); // Red phosphor
    } else if(x < 2.0) {
        mask = vec3(0.7, 1.0, 0.7); // Green phosphor  
    } else {
        mask = vec3(0.7, 0.7, 1.0); // Blue phosphor
    }

    color *= mix(vec3(1.0), mask, intensity);
}

void DrawFlicker(inout vec3 color) {
    float flickerIntensity = 0.01;
    float flickerSpeed = 15.0;

    float flicker = 1.0 + sin(iTime * flickerSpeed + random(vec2(iTime * 0.1))) * flickerIntensity;
    color *= flicker;
}

void main() {
    vec2 crtUV = CRTCurveUV(uv);
    // vec2 crtUV = uv;

    if(crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0) {
        discard;
    }

    vec3 texColor = DrawRGBSeparation(Texture, crtUV);
    vec4 tex = vec4(texColor, texture2D(Texture, crtUV).a);

    if(tex.a == 0.0) {
        discard;
    }

    vec3 res = tex.rgb * color.rgb;

    DrawBloom(res, crtUV);
    DrawColorTemperature(res);
    DrawScanline(res, crtUV);
    DrawShadowMask(res, crtUV);
    DrawFilmGrain(res, crtUV);
    DrawFlicker(res);
    DrawVignette(res, crtUV);

    gl_FragColor = vec4(res, 1.0);
}
