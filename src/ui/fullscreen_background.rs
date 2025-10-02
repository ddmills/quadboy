use bevy_ecs::prelude::*;
use macroquad::prelude::trace;

use crate::{
    common::Palette,
    rendering::{Glyph, Layer, Position, ScreenSize},
};

/// Marker component for full-screen background elements
#[derive(Component)]
pub struct FullScreenBackground;

/// System to create and update full-screen backgrounds
pub fn setup_fullscreen_backgrounds(
    mut cmds: Commands,
    screen: Res<ScreenSize>,
    q_backgrounds: Query<Entity, With<FullScreenBackground>>,
    mut q_glyphs: Query<&mut Glyph>,
    mut q_positions: Query<&mut Position>,
) {
    trace!("setup_fullscreen_backgrounds called - screen size: {}x{}", screen.tile_w, screen.tile_h);

    let color = Palette::Clear;

    // Check if we already have a background
    if let Ok(background_entity) = q_backgrounds.single() {
        trace!("Updating existing background entity");
        // Update existing background to match current screen size
        if let Ok(mut glyph) = q_glyphs.get_mut(background_entity) {
            glyph.scale = (screen.tile_w as f32, screen.tile_h as f32);
        }
        if let Ok(mut position) = q_positions.get_mut(background_entity) {
            position.x = 0.0;
            position.y = 0.0;
        }
    } else {
        // Create new background using the same pattern as UI panels
        trace!("Creating new background entity");
        let entity = cmds.spawn((
            FullScreenBackground,
            Position::new(0, 0, 0),
            Glyph::new(6, color, color)
                .bg(color)
                .scale((screen.tile_w as f32, screen.tile_h as f32))
                .layer(Layer::UiPanels),
        )).id();
        trace!("Created background entity: {:?}", entity);
    }
}