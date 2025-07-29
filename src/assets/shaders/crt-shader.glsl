#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;
uniform float iTime;
uniform vec2 iResolution;

vec2 CRTCurveUV(vec2 uv) {
    vec2 curvature = vec2(8.0, 6.0);
    vec2 curved = uv * 2.0 - 1.0;
    vec2 offset = abs(uv.yx) / curvature;
    curved = curved + curved * offset * offset;
    curved = curved * 0.5 + 0.5;
    return curved;
}

void DrawVignette(inout vec3 color, vec2 uv) {
    float str = 0.5;
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    vignette = clamp(pow(16.0 * vignette, 0.3), 0., 1.0);
    color *= vignette;
}

void DrawScanline(inout vec3 color, vec2 uv) {
    float width = 2.;
    float phase = iTime / 100.;
    float thickness = 2.4;
    float opacity = 0.2;
    vec3 lineColor = vec3(0.27, 0.31, 0.33);

    float v = .5 * (sin((uv.y + phase) * 3.14159 / width * iResolution.y) + 1.);
    color.rgb -= (lineColor - color.rgb) * (pow(v, thickness) - 1.0) * opacity;
}

void main() {
    // vec2 crtUV = CRTCurveUV(uv);
    vec2 crtUV = uv;
    vec4 tex = texture2D(Texture, crtUV);

    if (tex.a == 0) {
        discard;
    }

    vec3 res = tex.rgb * color.rgb;

    if(crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0) {
        discard;
    }

    // DrawVignette(res, uv);
    DrawScanline(res, uv);

    gl_FragColor = vec4(res, 1.0);
}
