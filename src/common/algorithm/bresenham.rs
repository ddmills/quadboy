/// Simple Bresenham line algorithm for creating direct paths between two points
pub fn bresenham_line(from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
    let mut path = Vec::new();

    let mut x0 = from.0 as i32;
    let mut y0 = from.1 as i32;
    let x1 = to.0 as i32;
    let y1 = to.1 as i32;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    loop {
        // Add current position to path (no bounds checking - caller should handle)
        path.push((x0 as usize, y0 as usize));

        if x0 == x1 && y0 == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }

    path
}
