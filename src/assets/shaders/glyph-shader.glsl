#version 400

precision lowp float;

in vec2 uv;

uniform float idx;
uniform vec4 fg1;
uniform vec4 fg2;
uniform vec4 outline;
uniform vec4 bg;
uniform sampler2D Texture;

void main() {
    vec2 uv_scaled = uv / 16.0; // atlas is 16x16
    float x = float(uint(idx) % 16u);
    float y = float(uint(idx) / 16u);
    vec2 uv_offset = vec2(x, y) / 16.0;

    vec2 tex_uv = uv_offset + uv_scaled;

    vec4 tex = texture2D(Texture, tex_uv);

    gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0);

    if (tex.a == 0) { // transparent (background)
        gl_FragColor = bg;
    } else if (tex.r == 0 && tex.g == 0 && tex.b == 0 && fg1.a > 0) { // Black (Primary)
        gl_FragColor = fg1;
    } else if (tex.r == 1 && tex.g == 1 && tex.b == 1 && fg2.a > 0) { // White (Secondary)
        gl_FragColor = fg2;
    } else if (tex.r == 1 && tex.g == 0 && tex.b == 0 && outline.a > 0) { // Red (Outline)
        gl_FragColor = outline;
    } else { // debug
        gl_FragColor = vec4(1.0, 1.0, 0.0, 1.0);
    }
}
