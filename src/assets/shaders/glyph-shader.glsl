#version 100
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
    
    // Replace uint() conversion with float operations
    float idx_float = floor(idx + 0.5); // round to nearest integer
    float x = mod(idx_float, 16.0);
    float y = floor(idx_float / 16.0);
    vec2 uv_offset = vec2(x, y) / 16.0;

    vec2 tex_uv = uv_offset + uv_scaled;

    vec4 v = vec4(0.0);

    // Use abs() and small epsilon for floating point comparison
    if (abs(tex_idx - 0.0) < 0.5) {
        v = texture2D(tex_1, tex_uv);
    } else {
        v = texture2D(tex_2, tex_uv);
    }

    if (v.a == 0.0) { // transparent (background)
        gl_FragColor = bg;
    } else if (v.r == 0.0 && v.g == 0.0 && v.b == 0.0 && fg1.a > 0.0) { // Black (Primary)
        gl_FragColor = fg1;
    } else if (v.r == 1.0 && v.g == 1.0 && v.b == 1.0 && fg2.a > 0.0) { // White (Secondary)
        gl_FragColor = fg2;
    } else if (v.r == 1.0 && v.g == 0.0 && v.b == 0.0 && outline.a > 0.0) { // Red (Outline)
        gl_FragColor = outline;
    } else { // debug
        gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
    }

    if (gl_FragColor.a == 0.0) {
        discard;
    }
}