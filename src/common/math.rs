pub fn max_3<T: Ord>(a: T, b: T, c: T) -> T {
    std::cmp::max(std::cmp::max(a, b), c)
}

pub fn min_max_3<T: Ord + Copy>(a: T, b: T, c: T) -> [T; 3] {
    let mut values = [a, b, c];
    values.sort();
    values
}

// remap a number v that is between 0-1 to be between min and max
pub fn remap(v: f32, min: f32, max: f32) -> f32 {
    (v * (max - min)) + min
}
