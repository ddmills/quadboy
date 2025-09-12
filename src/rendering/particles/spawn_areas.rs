use macroquad::math::Vec2;
use macroquad::rand::{gen_range, rand};

#[derive(Clone, Debug)]
pub enum SpawnArea {
    Point,
    Circle { radius: f32, distribution: Distribution },
    Rectangle { width: f32, height: f32, distribution: Distribution },
    Line { start: Vec2, end: Vec2, spacing: SpawnSpacing },
    Arc { radius: f32, angle_start: f32, angle_end: f32, radial_distribution: Distribution },
}

#[derive(Clone, Debug)]
pub enum Distribution {
    Uniform,        // Even spread throughout area
    EdgeOnly,       // Only on perimeter
    Gaussian,       // Clustered toward center
}

#[derive(Clone, Debug)]
pub enum SpawnSpacing {
    Uniform,        // Even spacing along line
    Random,         // Random positions along line
    Count(u32),     // Specific number of evenly spaced points
}

impl SpawnArea {
    pub fn generate_position(&self, base_position: Vec2) -> Vec2 {
        match self {
            SpawnArea::Point => base_position,
            
            SpawnArea::Circle { radius, distribution } => {
                base_position + generate_circle_position(*radius, distribution)
            }
            
            SpawnArea::Rectangle { width, height, distribution } => {
                base_position + generate_rectangle_position(*width, *height, distribution)
            }
            
            SpawnArea::Line { start, end, spacing } => {
                base_position + generate_line_position(*start, *end, spacing)
            }
            
            SpawnArea::Arc { radius, angle_start, angle_end, radial_distribution } => {
                base_position + generate_arc_position(*radius, *angle_start, *angle_end, radial_distribution)
            }
        }
    }
}

fn generate_circle_position(radius: f32, distribution: &Distribution) -> Vec2 {
    match distribution {
        Distribution::Uniform => {
            // Random point within circle using rejection sampling for uniform distribution
            let r = radius * (rand() as f32).sqrt(); // sqrt for uniform area distribution
            let theta = gen_range(0.0, 2.0 * std::f32::consts::PI);
            Vec2::new(r * theta.cos(), r * theta.sin())
        }
        
        Distribution::EdgeOnly => {
            // Random point on circle circumference
            let theta = gen_range(0.0, 2.0 * std::f32::consts::PI);
            Vec2::new(radius * theta.cos(), radius * theta.sin())
        }
        
        Distribution::Gaussian => {
            // Gaussian distribution toward center (using Box-Muller transform approximation)
            let r = (gen_range(0.0, 1.0) + gen_range(0.0, 1.0)) * 0.5 * radius;
            let theta = gen_range(0.0, 2.0 * std::f32::consts::PI);
            Vec2::new(r * theta.cos(), r * theta.sin())
        }
    }
}

fn generate_rectangle_position(width: f32, height: f32, distribution: &Distribution) -> Vec2 {
    match distribution {
        Distribution::Uniform => {
            Vec2::new(
                gen_range(-width * 0.5, width * 0.5),
                gen_range(-height * 0.5, height * 0.5)
            )
        }
        
        Distribution::EdgeOnly => {
            // Random point on rectangle perimeter
            let perimeter = 2.0 * (width + height);
            let t = gen_range(0.0, perimeter);
            
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
            let x = (gen_range(0.0, 1.0) + gen_range(0.0, 1.0) - 1.0) * width * 0.5;
            let y = (gen_range(0.0, 1.0) + gen_range(0.0, 1.0) - 1.0) * height * 0.5;
            Vec2::new(x, y)
        }
    }
}

fn generate_line_position(start: Vec2, end: Vec2, spacing: &SpawnSpacing) -> Vec2 {
    let t = match spacing {
        SpawnSpacing::Uniform => gen_range(0.0, 1.0),
        SpawnSpacing::Random => gen_range(0.0, 1.0),
        SpawnSpacing::Count(_count) => {
            // For count spacing, we need context of which particle this is
            // For now, treat as uniform random
            gen_range(0.0, 1.0)
        }
    };
    
    start + (end - start) * t
}

fn generate_arc_position(radius: f32, angle_start: f32, angle_end: f32, distribution: &Distribution) -> Vec2 {
    let angle_range = angle_end - angle_start;
    let angle = angle_start + gen_range(0.0, angle_range);
    
    let r = match distribution {
        Distribution::Uniform => gen_range(0.0, radius),
        Distribution::EdgeOnly => radius,
        Distribution::Gaussian => {
            // Gaussian distribution toward center
            (gen_range(0.0, 1.0) + gen_range(0.0, 1.0)) * 0.5 * radius
        }
    };
    
    let angle_rad = angle.to_radians();
    Vec2::new(r * angle_rad.cos(), r * angle_rad.sin())
}