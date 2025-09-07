#version 100
precision lowp float;

varying lowp vec2 uv;
varying float idx;
varying float tex_idx;
varying vec4 fg1;
varying vec4 fg2;
varying vec4 bg;
varying vec4 outline;
varying float is_shrouded;
varying vec4 light_rgba;
varying float light_flicker;
varying float ignore_lighting;

uniform sampler2D tex_1;
uniform sampler2D tex_2;
uniform float time;
uniform vec4 ambient;

vec4 apply_shroud(vec4 color) {
    if (color.a == 0.0) return color;

    vec3 shrouded = mix(color.rgb, ambient.rgb, 0.8);

    return vec4(shrouded, color.a);
}

void main() {
    vec2 uv_scaled = uv / 16.0; // atlas is 16x16

    float idx_float = floor(idx + 0.5);
    float x = mod(idx_float, 16.0);
    float y = floor(idx_float / 16.0);
    vec2 uv_offset = vec2(x, y) / 16.0;

    vec2 tex_uv = uv_offset + uv_scaled;

    vec4 v = vec4(0.0);

    bool apply_lighting = ignore_lighting < 0.5;

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

    if (gl_FragColor.r == 1.0 && gl_FragColor.g == 1.0 && gl_FragColor.b == 0.0) { // Clear color
        gl_FragColor = vec4(ambient.rgb, 1.0);
        apply_lighting = false;
    }

    if (gl_FragColor.a == 0.0) {
        discard;
    }

    if (apply_lighting) {
        if (is_shrouded > 0.5) {
            gl_FragColor = apply_shroud(gl_FragColor);
        } else {
            vec3 dynamic_color = light_rgba.rgb;
            float dynamic_intensity = light_rgba.w;
            vec3 ambient_color = ambient.rgb;
            float ambient_intensity = ambient.w;

            // Apply dynamic lighting if present
            if (dynamic_intensity > 0.0) {
                float dynamic_strength = (1.0 - ambient_intensity) * 0.4 + 0.4; // Range: 0.4 to 1.0
                float effective_dynamic = dynamic_intensity * dynamic_strength;
                gl_FragColor.rgb = mix(gl_FragColor.rgb, dynamic_color, effective_dynamic);
            }

            // Bring everything toward ambient color (NOTE: darkens all non-dynamics!)
            float darkness = (1.0 - max(ambient.w, dynamic_intensity)) * 0.5;
            gl_FragColor = mix(gl_FragColor, vec4(ambient_color, 1.0), darkness);
        }
    }
}