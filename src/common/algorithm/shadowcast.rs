pub struct ShadowcastSettings<F1, F2>
where
    F1: Fn(i32, i32) -> bool,
    F2: FnMut(i32, i32, f64),
{
    pub start_x: i32,
    pub start_y: i32,
    pub distance: i32,
    pub is_blocker: F1,
    pub on_light: F2,
}

static QUADRANTS: [(i32, i32); 4] = [(-1, -1), (1, -1), (-1, 1), (1, 1)];

pub fn shadowcast<F1, F2>(mut settings: ShadowcastSettings<F1, F2>)
where
    F1: Fn(i32, i32) -> bool,
    F2: FnMut(i32, i32, f64),
{
    (settings.on_light)(settings.start_x, settings.start_y, 0.0);

    for q in QUADRANTS.iter() {
        cast_light(1, 1.0, 0.0, 0, q.0, q.1, 0, &mut settings);
        cast_light(1, 1.0, 0.0, q.0, 0, 0, q.1, &mut settings);
    }
}

fn cast_light<F1, F2>(
    row: i32,
    start: f64,
    end: f64,
    xx: i32,
    xy: i32,
    yx: i32,
    yy: i32,
    settings: &mut ShadowcastSettings<F1, F2>,
) where
    F1: Fn(i32, i32) -> bool,
    F2: FnMut(i32, i32, f64),
{
    let mut iter_start = start;
    let mut new_start = 0.0;

    if start < end {
        return;
    }

    let mut is_blocked = false;

    for distance in row..=settings.distance {
        if is_blocked {
            break;
        }

        let delta_y = -distance;

        for delta_x in -distance..=0 {
            let pos_x = settings.start_x + (delta_x * xx) + (delta_y * xy);
            let pos_y = settings.start_y + (delta_x * yx) + (delta_y * yy);

            let left_slope = (delta_x as f64 - 0.5) / (delta_y as f64 + 0.5);
            let right_slope = (delta_x as f64 + 0.5) / (delta_y as f64 - 0.5);

            if right_slope > iter_start {
                continue;
            }

            if left_slope < end {
                break;
            }

            let delta_distance = ((delta_x * delta_x + delta_y * delta_y) as f64)
                .sqrt()
                .round();

            if delta_distance <= settings.distance as f64 {
                (settings.on_light)(pos_x, pos_y, delta_distance);
            }

            if is_blocked {
                if (settings.is_blocker)(pos_x, pos_y) {
                    new_start = right_slope;
                } else {
                    is_blocked = false;
                    iter_start = new_start;
                }
            } else if distance < settings.distance && (settings.is_blocker)(pos_x, pos_y) {
                is_blocked = true;
                cast_light(
                    distance + 1,
                    iter_start,
                    left_slope,
                    xx,
                    xy,
                    yx,
                    yy,
                    settings,
                );
                new_start = right_slope;
            }
        }
    }
}
