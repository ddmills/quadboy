use std::f32::consts::PI;

use macroquad::math::Vec2;

use crate::common::Rand;

#[derive(Clone, Debug)]
pub enum SpawnArea {
    Point,
    Circle {
        radius: f32,
        distribution: Distribution,
    },
    Rectangle {
        width: f32,
        height: f32,
        distribution: Distribution,
    },
    Line {
        start: Vec2,
        end: Vec2,
        spacing: SpawnSpacing,
    },
    Arc {
        radius: f32,
        angle_start: f32,
        angle_end: f32,
        radial_distribution: Distribution,
    },
}

#[derive(Clone, Debug)]
pub enum Distribution {
    Uniform,  // Even spread throughout area
    EdgeOnly, // Only on perimeter
    Gaussian, // Clustered toward center
}

#[derive(Clone, Debug)]
pub enum SpawnSpacing {
    Uniform,    // Even spacing along line
    Random,     // Random positions along line
    Count(u32), // Specific number of evenly spaced points
}

impl SpawnArea {
    pub fn generate_position(&self, base_position: Vec2, rand: &mut Rand) -> Vec2 {
        match self {
            SpawnArea::Point => base_position,

            SpawnArea::Circle {
                radius,
                distribution,
            } => base_position + generate_circle_position(*radius, distribution, rand),

            SpawnArea::Rectangle {
                width,
                height,
                distribution,
            } => base_position + generate_rectangle_position(*width, *height, distribution, rand),

            SpawnArea::Line {
                start,
                end,
                spacing,
            } => base_position + generate_line_position(*start, *end, spacing, rand),

            SpawnArea::Arc {
                radius,
                angle_start,
                angle_end,
                radial_distribution,
            } => {
                base_position
                    + generate_arc_position(
                        *radius,
                        *angle_start,
                        *angle_end,
                        radial_distribution,
                        rand,
                    )
            }
        }
    }
}

fn generate_circle_position(radius: f32, distribution: &Distribution, rand: &mut Rand) -> Vec2 {
    match distribution {
        Distribution::Uniform => {
            // Random point within circle using rejection sampling for uniform distribution
            let r = radius * rand.random().sqrt(); // sqrt for uniform area distribution
            let theta = rand.random() * 2.0 * PI;
            Vec2::new(r * theta.cos(), r * theta.sin())
        }

        Distribution::EdgeOnly => {
            // Random point on circle circumference
            let theta = rand.random() * 2.0 * PI;
            Vec2::new(radius * theta.cos(), radius * theta.sin())
        }

        Distribution::Gaussian => {
            // Gaussian distribution toward center (using Box-Muller transform approximation)
            let r = (rand.random() + rand.random()) * 0.5 * radius;
            let theta = rand.random() * 2.0 * PI;
            Vec2::new(r * theta.cos(), r * theta.sin())
        }
    }
}

fn generate_rectangle_position(
    width: f32,
    height: f32,
    distribution: &Distribution,
    rand: &mut Rand,
) -> Vec2 {
    match distribution {
        Distribution::Uniform => Vec2::new(
            rand.random() * width - width * 0.5,
            rand.random() * height - height * 0.5,
        ),

        Distribution::EdgeOnly => {
            // Random point on rectangle perimeter
            let perimeter = 2.0 * (width + height);
            let t = rand.random() * perimeter;

            if t < width {
                // Top edge
                Vec2::new(t - width * 0.5, height * 0.5)
            } else if t < width + height {
                // Right edge
                Vec2::new(width * 0.5, height * 0.5 - (t - width))
            } else if t < 2.0 * width + height {
                // Bottom edge
                Vec2::new(width * 0.5 - (t - width - height), -height * 0.5)
            } else {
                // Left edge
                Vec2::new(-width * 0.5, -height * 0.5 + (t - 2.0 * width - height))
            }
        }

        Distribution::Gaussian => {
            // Gaussian distribution toward center
            let x = (rand.random() + rand.random() - 1.0) * width * 0.5;
            let y = (rand.random() + rand.random() - 1.0) * height * 0.5;
            Vec2::new(x, y)
        }
    }
}

fn generate_line_position(start: Vec2, end: Vec2, spacing: &SpawnSpacing, rand: &mut Rand) -> Vec2 {
    let t = match spacing {
        SpawnSpacing::Uniform => rand.random(),
        SpawnSpacing::Random => rand.random(),
        SpawnSpacing::Count(_count) => {
            // For count spacing, we need context of which particle this is
            // For now, treat as uniform random
            rand.random()
        }
    };

    start + (end - start) * t
}

fn generate_arc_position(
    radius: f32,
    angle_start: f32,
    angle_end: f32,
    distribution: &Distribution,
    rand: &mut Rand,
) -> Vec2 {
    let angle_range = angle_end - angle_start;
    let angle = angle_start + rand.random() * angle_range;

    let r = match distribution {
        Distribution::Uniform => rand.random() * radius,
        Distribution::EdgeOnly => radius,
        Distribution::Gaussian => {
            // Gaussian distribution toward center
            (rand.random() + rand.random()) * 0.5 * radius
        }
    };

    let angle_rad = angle.to_radians();
    Vec2::new(r * angle_rad.cos(), r * angle_rad.sin())
}
