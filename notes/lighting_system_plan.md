# Lighting System Implementation Plan

## Design Requirements

### Answered Questions:
1. **Lighting Computation Scope**: All loaded zones (All LightSource components where isEnabled=true)
2. **Light Falloff Model**: Quadratic (realistic: 1/distance²)
3. **Light Combination**: Custom blending, intensity capped at 1.0
4. **Ambient Light**: Constant for now, eventually based on time of day and biome
5. **Dynamic Light Colors**:
   - Blend/mix when multiple colored lights overlap: YES
   - Affect glyph colors in the shader: YES
   - Each light has RGB color (stored as u32) and single intensity
6. **Performance Considerations**: Recompute every frame when !clock.is_frozen()

## Implementation Details

### 1. Core Components

#### LightSource Component
```rust
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct LightSource {
    pub intensity: f32,      // 0.0 - 1.0 base brightness
    pub color: u32,          // Packed RGB color (0xRRGGBB format)
    pub range: i32,          // Maximum distance light travels
    pub is_enabled: bool,    // Can be toggled on/off
    pub flicker_speed: f32,  // 0.0 = no flicker, >0 = flicker frequency in Hz
    pub flicker_amount: f32, // 0.0-1.0, how much intensity varies
}
```

#### LightBlocker Component
```rust
#[derive(Component)]
pub struct LightBlocker;  // Simple marker component - blocks light completely
```
*Note: LightBlocker is not serializable since lighting is recomputed each frame*

### 2. Lighting Data Storage

```rust
#[derive(Resource)]
pub struct LightingData {
    // Light maps for all loaded zones
    zone_light_maps: HashMap<usize, Grid<LightValue>>,
    // Constant ambient light
    ambient_light: LightValue,
}

#[derive(Clone, Default)]
pub struct LightValue {
    pub rgba: Vec4,  // RGB color (xyz) + intensity (w) packed together
    pub flicker_params: Vec2,  // x = speed (Hz), y = amount (0-1)
}

impl LightValue {
    pub fn new(color: u32, intensity: f32) -> Self {
        let r = ((color >> 16) & 0xFF) as f32 / 255.0;
        let g = ((color >> 8) & 0xFF) as f32 / 255.0;
        let b = (color & 0xFF) as f32 / 255.0;
        Self {
            rgba: Vec4::new(r, g, b, intensity),
        }
    }
}
```

### 3. Lighting Computation System

#### Main Update System
```rust
pub fn update_lighting_system(
    world: &mut World,
    clock: Res<Clock>,
    zones: Res<Zones>,
    mut lighting_data: ResMut<LightingData>,
    q_lights: Query<(&Position, &LightSource)>,
    q_blockers: Query<&Position, With<LightBlocker>>,
    time: Res<Time>,
) {
    if clock.is_frozen() {
        return;
    }
    
    // Clear and apply ambient to all loaded zones
    for &zone_idx in zones.loaded.iter() {
        lighting_data.clear_zone(zone_idx);
        lighting_data.apply_ambient(zone_idx, AMBIENT_LIGHT);
    }
    
    // Process all enabled lights in loaded zones
    for (pos, light) in q_lights.iter() {
        if !light.is_enabled {
            continue;
        }
        
        let zone_idx = pos.zone_idx();
        if !zones.loaded.contains(&zone_idx) {
            continue;
        }
        
        compute_light_propagation(
            pos,
            light,
            &q_blockers,
            &mut lighting_data,
        );
    }
}

// Constant ambient light (for now)
const AMBIENT_LIGHT: LightValue = LightValue {
    rgba: Vec4::new(0.5, 0.5, 0.6, 0.1),  // Slightly blue-tinted, dim
};
```

#### Light Propagation with Shadowcast
```rust
fn compute_light_propagation(
    pos: &Position,
    light: &LightSource,
    blockers: &Query<&Position, With<LightBlocker>>,
    lighting_data: &mut LightingData,
) {
    // Convert u32 color to RGB floats once
    let r = ((light.color >> 16) & 0xFF) as f32 / 255.0;
    let g = ((light.color >> 8) & 0xFF) as f32 / 255.0;
    let b = (light.color & 0xFF) as f32 / 255.0;
    
    let settings = ShadowcastSettings {
        start_x: pos.x as i32,
        start_y: pos.y as i32,
        distance: light.range,
        is_blocker: |x, y| {
            // Check if any blocker exists at this position
            blockers.iter().any(|blocker_pos| {
                blocker_pos.x == x && 
                blocker_pos.y == y && 
                blocker_pos.zone_idx == pos.zone_idx
            })
        },
        on_light: |x, y, distance| {
            // Quadratic falloff (inverse square law)
            let falloff = 1.0 / (1.0 + distance * distance * 0.1);
            let intensity = light.intensity * falloff;
            
            lighting_data.blend_light(
                pos.zone_idx,
                x,
                y,
                r, g, b,
                intensity,
                light.flicker_speed,
                light.flicker_amount,
            );
        },
    };
    
    shadowcast(settings);
}
```

#### Custom Light Blending
```rust
impl LightingData {
    pub fn blend_light(&mut self, zone_idx: usize, x: i32, y: i32, 
                       r: f32, g: f32, b: f32, intensity: f32,
                       flicker_speed: f32, flicker_amount: f32) {
        let light_map = self.zone_light_maps.entry(zone_idx)
            .or_insert_with(|| Grid::new(ZONE_SIZE.0, ZONE_SIZE.1));
        
        if let Some(current) = light_map.get_mut(x as usize, y as usize) {
            let curr_intensity = current.rgba.w;
            let new_total = curr_intensity + intensity;
            
            if new_total > 0.0 {
                // Blend colors weighted by intensity
                let curr_weight = curr_intensity / new_total;
                let new_weight = intensity / new_total;
                
                current.rgba.x = current.rgba.x * curr_weight + r * new_weight;
                current.rgba.y = current.rgba.y * curr_weight + g * new_weight;
                current.rgba.z = current.rgba.z * curr_weight + b * new_weight;
                current.rgba.w = new_total.min(1.0);  // Cap at 1.0
                
                // Blend flicker parameters (take maximum to preserve strongest flicker)
                if flicker_speed > 0.0 {
                    current.flicker_params.x = current.flicker_params.x.max(flicker_speed);
                    current.flicker_params.y = current.flicker_params.y.max(flicker_amount * new_weight);
                }
            }
        }
    }
}
```

### 4. Shader Integration

#### Fragment Shader Updates (glyph-shader.glsl)
```glsl
// Add uniform for time
uniform float time;

// Lighting varyings
varying vec4 light_rgba;  // xyz = color, w = intensity
varying vec2 light_flicker;  // x = speed (Hz), y = amount

void main() {
    // ... existing glyph color selection ...
    
    // Calculate flicker if applicable
    float light_intensity = light_rgba.w;
    if (light_flicker.x > 0.0) {
        // Create flicker using sine wave
        float flicker = sin(time * light_flicker.x * 6.28318) * 0.5 + 0.5;
        // Apply flicker amount
        light_intensity *= mix(1.0 - light_flicker.y, 1.0, flicker);
    }
    
    vec3 light_color = light_rgba.xyz;
    
    if (light_intensity > 0.0) {
        // Apply light color and intensity to glyph
        vec3 lit_color = gl_FragColor.rgb * light_color * light_intensity;
        
        // Mix between dark and lit based on intensity
        gl_FragColor.rgb = mix(
            gl_FragColor.rgb * 0.05,  // Minimum visibility in darkness
            lit_color,
            light_intensity
        );
    }
    
    // ... rest of shader ...
}
```

#### Vertex Shader Updates
```glsl
// Lighting attributes
attribute vec4 in_light_rgba;  // xyz = color, w = intensity
attribute vec2 in_light_flicker;  // x = speed, y = amount

// Lighting varyings to pass to fragment
varying vec4 light_rgba;
varying vec2 light_flicker;

void main() {
    // ... existing vertex shader ...
    
    // Pass lighting data to fragment shader
    light_rgba = in_light_rgba;
    light_flicker = in_light_flicker;
}
```

### 5. GlyphBatch Modifications

```rust
// Update InstanceData struct
struct InstanceData {
    // ... existing fields ...
    light_rgba: Vec4,  // RGB color + intensity in one field
    light_flicker: Vec2,  // Flicker speed and amount
}

// Update vertex attributes in pipeline creation
VertexAttribute::with_buffer("in_light_rgba", VertexFormat::Float4, 1),
VertexAttribute::with_buffer("in_light_flicker", VertexFormat::Float2, 1),

// Add time uniform to shader meta
uniforms: vec![
    UniformDesc::new("projection", UniformType::Mat4),
    UniformDesc::new("time", UniformType::Float1),
],

// In render_glyphs system
fn render_glyphs(
    // ... existing params ...
    lighting_data: Res<LightingData>,
    time: Res<Time>,
) {
    // When batching glyphs, look up lighting value
    let light_value = lighting_data.get_light(
        glyph_pos.zone_idx, 
        glyph_pos.x, 
        glyph_pos.y
    ).unwrap_or_default();
    
    instance.light_rgba = light_value.rgba;
    instance.light_flicker = light_value.flicker_params;
    
    // Update time uniform for shader
    batch.set_uniform("time", time.elapsed);
}
```

### 6. Light Source Presets

```rust
impl LightSource {
    pub fn new(intensity: f32, color: u32, range: i32) -> Self {
        Self {
            intensity,
            color,
            range,
            is_enabled: true,
            flicker_speed: 0.0,
            flicker_amount: 0.0,
        }
    }
    
    pub fn with_flicker(mut self, speed: f32, amount: f32) -> Self {
        self.flicker_speed = speed;
        self.flicker_amount = amount;
        self
    }

    // Common light presets using u32 colors
    pub fn torch() -> Self {
        Self::new(0.8, 0xFFB366, 6)  // Warm orange
            .with_flicker(3.0, 0.3)  // 3Hz flicker, 30% intensity variation
    }

    pub fn campfire() -> Self {
        Self::new(0.9, 0xFF994D, 8)  // Warm orange-red
            .with_flicker(2.5, 0.4)  // 2.5Hz flicker, 40% intensity variation
    }

    pub fn lantern() -> Self {
        Self::new(0.85, 0xF2E6B3, 8)  // Warm white, no flicker
    }

    pub fn player_light() -> Self {
        Self::new(0.4, 0xCCCCFF, 4)  // Cool blue-white, no flicker
    }
}
```

### 7. Integration Points

#### Game Systems Registration
```rust
// In register_game_systems
world.register_system(update_lighting_system);
// Execute after position updates but before rendering
// Run every frame when !clock.is_frozen()
```

#### Prefab Updates
- Campfire: Add `LightSource::campfire()`
- Lantern: Add `LightSource::lantern()`
- Trees/Boulders: Add `LightBlocker` component
- Player: Consider adding dim light source

### 8. File Organization

```
src/domain/components/
  lighting.rs         // LightSource, LightBlocker components
  mod.rs             // Export lighting components

src/domain/systems/
  lighting_system.rs  // Core lighting computation
  mod.rs             // Export lighting system

src/rendering/
  lighting_data.rs    // LightingData resource, LightValue struct
  mod.rs             // Export lighting data

src/assets/shaders/
  glyph-shader.glsl  // Update fragment shader
  (vertex shader embedded in glyph_batch.rs)
```

### 9. Implementation Checklist

- [ ] Create lighting components (LightSource, LightBlocker)
- [ ] Create LightingData resource structure
- [ ] Implement lighting computation system
- [ ] Update fragment shader to accept lighting data
- [ ] Update vertex shader attributes
- [ ] Modify GlyphBatch to pass lighting values
- [ ] Register lighting system in game loop
- [ ] Add light sources to campfire prefab
- [ ] Add LightBlocker to trees/boulders
- [ ] Test performance with multiple light sources
- [ ] Optimize if needed (spatial indexing, dirty tracking)

## Key Design Decisions

1. **u32 Color Storage**: More memory efficient than Vec3, easier to define preset colors
2. **Vec4 in Shader**: Single attribute/varying reduces complexity and improves performance
3. **Simple LightBlocker**: Marker component without opacity keeps it simple
4. **Quadratic Falloff**: Realistic lighting that looks natural
5. **Custom Blending with Cap**: Prevents over-brightening while allowing color mixing
6. **Recompute Every Frame**: Simplifies implementation, allows dynamic lighting

## Future Enhancements

1. **Ambient Light Variations**:
   - Time of day system
   - Biome-specific ambient colors
   - Weather effects on ambient

2. **Performance Optimizations**:
   - Spatial indexing for lights/blockers
   - Dirty tracking (only update when changes occur)
   - LOD system for distant zones
   - Light culling outside view

3. **Advanced Features**:
   - Colored shadows
   - Light occlusion mapping
   - Emissive glyphs
   - Dynamic light source creation (spells, explosions)