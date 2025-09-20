/// Simple Bresenham line algorithm for creating direct paths between two points
#[allow(dead_code)]
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

/// Bresenham circle algorithm for creating circles
/// Returns points forming either the outline or filled circle based on the `filled` parameter
/// Implementation based on classic Bresenham circle algorithm with balance/error term
#[allow(dead_code)]
pub fn bresenham_circle(radius: i32, filled: bool) -> Vec<(i32, i32)> {
    let mut points = Vec::new();

    if radius <= 0 {
        points.push((0, 0));
        return points;
    }

    // Use balance-based Bresenham algorithm (like Odyssey implementation)
    let mut balance = -radius;
    let mut dx = 0;
    let mut dy = radius;

    while dx <= dy {
        if filled {
            // Draw horizontal lines for filled circle
            let p0 = -dx;
            let p1 = -dy;
            let w0 = dx + dx + 1;
            let w1 = dy + dy + 1;

            // Add horizontal lines at y+dy and y-dy
            for i in 0..w0 {
                points.push((p0 + i, dy));
                points.push((p0 + i, -dy));
            }
            // Add horizontal lines at y+dx and y-dx (avoid duplicates at dy == dx)
            if dx != dy {
                for i in 0..w1 {
                    points.push((p1 + i, dx));
                    points.push((p1 + i, -dx));
                }
            }
        } else {
            // Add the 8 symmetric points for outline
            add_circle_points(&mut points, dx, dy);
        }

        dx += 1;
        balance += dx + dx;

        if balance >= 0 {
            dy -= 1;
            balance -= dy + dy;
        }
    }

    points
}

/// Helper function to add the 8 symmetric points of a circle
fn add_circle_points(points: &mut Vec<(i32, i32)>, x: i32, y: i32) {
    if x == 0 {
        points.push((0, y));
        points.push((0, -y));
    } else if y == 0 {
        points.push((x, 0));
        points.push((-x, 0));
    } else {
        points.push((x, y));
        points.push((-x, y));
        points.push((x, -y));
        points.push((-x, -y));
        if x != y {
            points.push((y, x));
            points.push((-y, x));
            points.push((y, -x));
            points.push((-y, -x));
        }
    }
}
