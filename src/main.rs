use common::Palette;
use macroquad::prelude::*;

mod common;

fn window_conf() -> Conf {
    Conf {
        window_title: "Quadboy".to_string(),
        window_width: 800,
        window_height: 600,
        fullscreen: false,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(Palette::Black.into());

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, Palette::Blue.into());
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, Palette::Green.into());
        draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, Palette::Yellow.into());

        let fps = get_fps();
        draw_text(fps.to_string().as_str(), 20.0, 20.0, 30.0, Palette::Orange.into());

        next_frame().await;
    }
}
