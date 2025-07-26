#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;
uniform float iTime;
uniform vec2 iResolution;

vec2 CRTCurveUV(vec2 uv) {
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs(uv.yx) / vec2(8.0, 6.0);
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void DrawVignette(inout vec3 color, vec2 uv) {
    float min_str = 0.25;
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    vignette = clamp(pow(16.0 * vignette, 0.3), min_str, 1.0);
    color *= vignette;
}

void DrawScanline(inout vec3 color, vec2 uv) {
    float width = 1.;
    float phase = iTime / 100.;
    float thickness = 1.;
    float opacity = 0.25;
    vec3 lineColor = vec3(0.11, 0.23, 0.19);

    float v = .5 * (sin((uv.y + phase) * 3.14159 / width * iResolution.y) + 1.);
    color.rgb -= (lineColor - color.rgb) * (pow(v, thickness) - 1.0) * opacity;
}

void main() {
    // vec2 crtUV = CRTCurveUV(uv);
    vec2 crtUV = uv;
    vec4 tex = texture2D(Texture, uv);

    if (tex.a == 0) {
        discard;
    }

    vec3 res = tex.rgb * color.rgb;

    if(crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0) {
        res = vec3(0.0, 0.0, 0.0);
    }

    // DrawVignette(res, uv);
    DrawScanline(res, uv);

    gl_FragColor = vec4(res, 1.0);
}
