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
    float intensity = 4.0;
    float roundness = 0.9;
    float feather = 0.5;

    vec2 delta = uv - vec2(0.5);

    float radialDist = length(delta);
    float axialDist = max(abs(delta.x), abs(delta.y));

    float shapeDist = mix(axialDist, radialDist, roundness);

    float falloff = pow(shapeDist, intensity);
    float vignette = 1.0 - smoothstep(0.0, 1.0 - feather, falloff);

    color *= vignette;
}

void DrawScanline(inout vec3 color, vec2 uv) {
    float width = 4.0;
    float phase = iTime / 100.0;
    float thickness = 2.6;
    float opacity = 0.25;
    vec3 lineColor = vec3(0.22, 0.25, 0.27);

    float v = 0.5 * (sin((uv.y + phase) * 3.14159 / width * iResolution.y) + 1.0);
    color.rgb -= (lineColor - color.rgb) * (pow(v, thickness) - 1.0) * opacity;
}

float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

void DrawFilmGrain(inout vec3 color, vec2 uv) {
    float intensity = 0.06;
    float speed = 10.0;

    vec2 noiseCoord = uv * iResolution.xy * 0.5;
    float timeOffset = floor(iTime * speed) * 0.01;

    float noise = random(noiseCoord + timeOffset);

    noise = (noise - 0.5) * intensity;

    color.rgb += noise;
}

void main() {
    vec2 crtUV = CRTCurveUV(uv);
    // vec2 crtUV = uv;
    vec4 tex = texture2D(Texture, crtUV);

    if (tex.a == 0.0) {
        discard;
    }

    vec3 res = tex.rgb * color.rgb;

    if(crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0) {
        discard;
    }

    DrawScanline(res, crtUV);
    DrawFilmGrain(res, crtUV);
    DrawVignette(res, crtUV);

    gl_FragColor = vec4(res, 1.0);
}
