pub mod level;

use macroquad::{
    camera::{self, Camera2D},
    color::{Color, colors},
    input::{self, KeyCode, MouseButton},
    shapes,
    window::{self, Conf},
};

use crate::level::Levels;

const START_IN_FULLSCREEN: bool = false;
const SCREEN_WIDTH: f32 = LOGICAL_SCREEN_WIDTH;
const SCREEN_HEIGHT: f32 = LOGICAL_SCREEN_HEIGHT + 0.25;
const SCREEN_ASPECT: f32 = SCREEN_WIDTH / SCREEN_HEIGHT;

const LOGICAL_SCREEN_WIDTH: f32 = Levels::LEVEL_WIDTH as f32;
const LOGICAL_SCREEN_HEIGHT: f32 = Levels::LEVEL_HEIGHT as f32;

fn window_conf() -> Conf {
    Conf {
        window_title: "Inverse".to_owned(),
        fullscreen: START_IN_FULLSCREEN,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut fullscreen = START_IN_FULLSCREEN;

    let mut camera = Camera2D::default();

    let mut levels = Levels::new();

    for x in 0..Levels::LEVEL_WIDTH {
        for y in 0..5 {
            levels[[x, y]] = true;
        }
    }

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            window::set_fullscreen(fullscreen);
        }

        if input::is_mouse_button_pressed(MouseButton::Left) {
            let mouse_position =
                <[f32; 2]>::from(camera.screen_to_world(input::mouse_position().into()));

            let mouse_position = [
                mouse_position[0] + LOGICAL_SCREEN_WIDTH / 2.0,
                mouse_position[1] + LOGICAL_SCREEN_HEIGHT / 2.0,
            ];

            if let Ok(mouse_index) = levels.index_of_position(mouse_position) {
                levels[mouse_index] ^= true;
            }
        }

        update_camera(&mut camera);
        camera::set_camera(&camera);

        window::clear_background(Color::from_hex(0x111111));

        shapes::draw_rectangle(
            -LOGICAL_SCREEN_WIDTH / 2.0,
            LOGICAL_SCREEN_HEIGHT / 2.0,
            LOGICAL_SCREEN_WIDTH,
            (window_height() - LOGICAL_SCREEN_HEIGHT) / 2.0,
            colors::WHITE,
        );

        shapes::draw_rectangle(
            -LOGICAL_SCREEN_WIDTH / 2.0,
            -window_height() / 2.0,
            LOGICAL_SCREEN_WIDTH,
            window_height() - (window_height() - LOGICAL_SCREEN_HEIGHT) / 2.0,
            colors::BLACK,
        );

        for x in 0..Levels::LEVEL_WIDTH {
            for y in 0..Levels::LEVEL_HEIGHT {
                if !levels[[x, y]] {
                    let position = [
                        x as f32 - SCREEN_WIDTH / 2.0,
                        y as f32 - LOGICAL_SCREEN_HEIGHT / 2.0,
                    ];

                    shapes::draw_rectangle(position[0], position[1], 1.0, 1.0, colors::WHITE);
                }
            }
        }

        window::next_frame().await;
    }
}

fn update_camera(camera: &mut Camera2D) {
    camera.zoom.x = 2.0 / window_width();
    camera.zoom.y = -2.0 / window_height();
}

fn window_width() -> f32 {
    let window_aspect = window::screen_width() / window::screen_height();

    if window_aspect < SCREEN_ASPECT {
        SCREEN_WIDTH
    } else {
        SCREEN_HEIGHT * window_aspect
    }
}

fn window_height() -> f32 {
    let window_aspect = window::screen_width() / window::screen_height();

    if window_aspect > SCREEN_ASPECT {
        SCREEN_HEIGHT
    } else {
        SCREEN_WIDTH / window_aspect
    }
}
