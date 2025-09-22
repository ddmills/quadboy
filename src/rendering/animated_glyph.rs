use bevy_ecs::prelude::*;
use quadboy_macros::profiled_system;
use serde::{Deserialize, Serialize};

use super::Glyph;
use crate::engine::{SerializableComponent, Time};

#[derive(Component, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct AnimatedGlyph {
    pub frames: Vec<usize>,
    pub speed_hz: f32,
    pub loop_animation: bool,
    pub current_frame: usize,
    pub timer: f32,
    pub is_playing: bool,
}

impl AnimatedGlyph {
    pub fn new(frames: Vec<usize>, speed_hz: f32) -> Self {
        Self {
            frames,
            speed_hz,
            loop_animation: true,
            current_frame: 0,
            timer: 0.0,
            is_playing: true,
        }
    }

    pub fn with_loop(mut self, loop_animation: bool) -> Self {
        self.loop_animation = loop_animation;
        self
    }
}

#[profiled_system]
pub fn update_animated_glyphs(
    mut q_animated: Query<(&mut AnimatedGlyph, &mut Glyph)>,
    time: Res<Time>,
) {

    for (mut anim_glyph, mut glyph) in q_animated.iter_mut() {
        if !anim_glyph.is_playing || anim_glyph.frames.is_empty() {
            continue;
        }

        let frame_duration = 1.0 / anim_glyph.speed_hz;
        anim_glyph.timer += time.dt;

        if anim_glyph.timer >= frame_duration {
            anim_glyph.timer -= frame_duration;
            anim_glyph.current_frame += 1;

            if anim_glyph.current_frame >= anim_glyph.frames.len() {
                if anim_glyph.loop_animation {
                    anim_glyph.current_frame = 0;
                } else {
                    anim_glyph.current_frame = anim_glyph.frames.len() - 1;
                    anim_glyph.is_playing = false;
                }
            }

            glyph.idx = anim_glyph.frames[anim_glyph.current_frame];
        }
    }
}
