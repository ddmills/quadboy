use crate::common::lerp_u32_colors;
use macroquad::math::Vec2;

#[derive(Clone, Debug)]
pub enum AnimationCurve<T> {
    Constant(T),
    Linear { values: Vec<T> },
    EaseOut { values: Vec<T> },
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
            AnimationCurve::Linear { values } => interpolate_values(values, t),
            AnimationCurve::EaseOut { values } => {
                let eased_t = easing::ease_out(t);
                interpolate_values(values, eased_t)
            }
        }
    }
}

fn interpolate_values<T: Lerpable + Clone>(values: &[T], progress: f32) -> T {
    if values.is_empty() {
        panic!("AnimationCurve values cannot be empty");
    }

    if values.len() == 1 {
        return values[0].clone();
    }

    let segments = values.len() - 1;
    let segment_progress = progress * segments as f32;
    let segment_index = (segment_progress.floor() as usize).min(segments - 1);
    let local_progress = segment_progress - segment_index as f32;

    let start_value = &values[segment_index];
    let end_value = &values[segment_index + 1];

    start_value.lerp(end_value, local_progress)
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
        Vec2::new(self.x.lerp(&other.x, t), self.y.lerp(&other.y, t))
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
