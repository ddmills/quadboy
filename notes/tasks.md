Tasks:
- [x] implement item stacking
- [x] fix common directory (random)
- [X] animated glyphs
- [] ui elements (buttons, lists)
- [] dialog
- [] item interactions
- [] cleanup glyph.rs


lets start planning a lighting system. We will have a LightSource component that has "intensity", "color", "range" and "isEnabled". Lighting will need to be recomputed for every tile in the game_systems loop whenever "!clock.is_frozen()". We do not need to save lighting data, as it's computed when the game loop updates. We will also have a "LightBlocker" component. We can use the existing Shadowcast implementation to compute lighting. Lighting values will need to be passed into the glyph-shader. Lets plan this feature in detail.

  1. Lighting Computation Scope: Should lighting be computed for:
    - For now, do all loaded zones. (All LightSource components that isEnabled=true)
  2. Light Falloff Model: What type of light falloff do you prefer:
    - Use Quadratic (realistic: 1/distance²)
  3. Light Combination: When multiple light sources overlap:
    - Custom blending, intensity capped at 1.0
  4. Ambient Light: Should there be:
    - Eventually there will be an ambient light based on time of day and biome. For now, just use a constant.
  5. Dynamic Light Colors: Should light color:
    - Blend/mix when multiple colored lights overlap? YES
    - Affect glyph colors in the shader? YES
    - Have separate RGB channels or single intensity? Each light will have it's own RGB, and a single intensity.
  6. Performance Considerations:
    - recompute every frame when !clock.is_frozen()

instead of fading to Clear, we should fade to the shroud color.

