use macroquad::math::Vec2;
use crate::common::lerp_u32_colors;

#[derive(Clone, Debug)]
pub enum AnimationCurve<T> {
    Constant(T),
    Linear { start: T, end: T },
    EaseOut { start: T, end: T },
}

pub trait CurveEvaluator<T> {
    fn evaluate(&self, progress: f32) -> T;
}

impl<T: Clone> CurveEvaluator<T> for AnimationCurve<T>
where
    T: Lerpable,
{
    fn evaluate(&self, progress: f32) -> T {
        let t = progress.clamp(0.0, 1.0);
        match self {
            AnimationCurve::Constant(value) => value.clone(),
            AnimationCurve::Linear { start, end } => start.lerp(end, t),
            AnimationCurve::EaseOut { start, end } => {
                let eased_t = easing::ease_out(t);
                start.lerp(end, eased_t)
            }
        }
    }
}

pub trait Lerpable {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerpable for f32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Lerpable for Vec2 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Vec2::new(
            self.x.lerp(&other.x, t),
            self.y.lerp(&other.y, t)
        )
    }
}

impl Lerpable for u32 {
    fn lerp(&self, other: &Self, t: f32) -> Self {
        lerp_u32_colors(*self, *other, t)
    }
}

pub mod easing {
    pub fn ease_out(t: f32) -> f32 {
        1.0 - (1.0 - t).powi(2)
    }
}

// Type aliases for common curves
pub type ColorCurve = AnimationCurve<u32>;
pub type VelocityCurve = AnimationCurve<Vec2>;
pub type AlphaCurve = AnimationCurve<f32>;