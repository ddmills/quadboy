use crate::common::{max_3, min_max_3};

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DistanceFormula {
    Manhattan,
    Diagonal,
    Euclidean,
    EuclideanSq,
    Chebyshev,
}

pub struct Distance;

#[allow(dead_code)]
impl Distance {
    pub fn get(formula: DistanceFormula, a: [i32; 3], b: [i32; 3]) -> f32 {
        match formula {
            DistanceFormula::Manhattan => Self::manhattan(a, b),
            DistanceFormula::Diagonal => Self::diagonal(a, b),
            DistanceFormula::Euclidean => Self::euclidean(a, b),
            DistanceFormula::EuclideanSq => Self::euclidean_sq(a, b),
            DistanceFormula::Chebyshev => Self::chebyshev(a, b),
        }
    }

    pub fn manhattan(a: [i32; 3], b: [i32; 3]) -> f32 {
        ((a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()) as f32
    }

    pub fn diagonal(a: [i32; 3], b: [i32; 3]) -> f32 {
        let dx = (a[0] - b[0]).abs();
        let dy = (a[1] - b[1]).abs();
        let dz = (a[2] - b[2]).abs();

        let [dmin, dmid, dmax] = min_max_3(dx, dy, dz);

        // From "Benchmarks for Pathfinding in 3D Voxel Space"
        // by Daniel Brewer and Nathan R. Sturtevant
        // (√3 − √2) * dmin + (√2 - 1) * dmid + dmax
        0.32 * dmin as f32 + 0.59 * dmid as f32 + dmax as f32
    }

    pub fn chebyshev(a: [i32; 3], b: [i32; 3]) -> f32 {
        let dx = (a[0] - b[0]).abs();
        let dy = (a[1] - b[1]).abs();
        let dz = (a[2] - b[2]).abs();

        max_3(dx, dy, dz) as f32
    }

    pub fn euclidean_sq(a: [i32; 3], b: [i32; 3]) -> f32 {
        let dx = (a[0] - b[0]).abs();
        let dy = (a[1] - b[1]).abs();
        let dz = (a[2] - b[2]).abs();

        (dx * dx + dy * dy + dz * dz) as f32
    }

    pub fn euclidean(a: [i32; 3], b: [i32; 3]) -> f32 {
        Self::euclidean_sq(a, b).sqrt()
    }
}
