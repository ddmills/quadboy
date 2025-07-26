#version 400
precision lowp float;

varying lowp vec2 uv;
varying float idx;
varying float tex_idx;
varying vec4 fg1;
varying vec4 fg2;
varying vec4 bg;
varying vec4 outline;

uniform sampler2D tex_1;
uniform sampler2D tex_2;

void main() {
    vec2 uv_scaled = uv / 16.0; // atlas is 16x16
    float x = float(uint(idx) % 16u);
    float y = float(uint(idx) / 16u);
    vec2 uv_offset = vec2(x, y) / 16.0;

    vec2 tex_uv = uv_offset + uv_scaled;

    vec4 v = vec4(0);

    if (tex_idx == 0) {
        v = texture2D(tex_1, tex_uv);
    } else {
        v = texture2D(tex_2, tex_uv);
    }

    if (v.a == 0) { // transparent (background)
        gl_FragColor = bg;
    } else if (v.r == 0 && v.g == 0 && v.b == 0 && fg1.a > 0) { // Black (Primary)
        gl_FragColor = fg1;
    } else if (v.r == 1 && v.g == 1 && v.b == 1 && fg2.a > 0) { // White (Secondary)
        gl_FragColor = fg2;
    } else if (v.r == 1 && v.g == 0 && v.b == 0 && outline.a > 0) { // Red (Outline)
        gl_FragColor = outline;
    } else { // debug
        gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
    }

    if (gl_FragColor.a == 0) {
        discard;
    }
}
