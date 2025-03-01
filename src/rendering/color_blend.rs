use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};

pub fn alpha_blend() -> BlendState {
    BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    )
}
