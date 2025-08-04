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
    float intensity = 3.0;
    float roundness = 0.5;
    float feather = 0.25;

    vec2 delta = uv - vec2(0.5);

    // Compute circular and square-like falloffs
    float radialDist = length(delta);
    float axialDist = max(abs(delta.x), abs(delta.y));

    // Blend between circular and square based on roundness
    float shapeDist = mix(axialDist, radialDist, roundness);

    // Apply non-linear falloff for more realistic vignette
    float falloff = pow(shapeDist, intensity);
    float vignette = 1.0 - smoothstep(0.0, 1.0 - feather, falloff);

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

    DrawVignette(res, crtUV);
    DrawScanline(res, crtUV);

    gl_FragColor = vec4(res, 1.0);
}
