pub mod level;
pub mod player;

use std::fs;

use macroquad::{
    camera::{self, Camera2D},
    color::{Color, colors},
    input::{self, KeyCode, MouseButton},
    shapes,
    window::{self, Conf},
};

use crate::level::Levels;
use crate::player::Player;

const START_IN_FULLSCREEN: bool = false;
const SCREEN_WIDTH: f32 = LOGICAL_SCREEN_WIDTH;
const SCREEN_HEIGHT: f32 = LOGICAL_SCREEN_HEIGHT + 0.25;
const SCREEN_ASPECT: f32 = SCREEN_WIDTH / SCREEN_HEIGHT;

const LOGICAL_SCREEN_WIDTH: f32 = Levels::LEVEL_WIDTH as f32;
const LOGICAL_SCREEN_HEIGHT: f32 = Levels::LEVEL_HEIGHT as f32;

const PATH_TO_LEVELS: &str = "levels.txt";

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

    let mut levels = fs::read_to_string(PATH_TO_LEVELS)
        .unwrap()
        .parse::<Levels>()
        .unwrap();
    let mut player = Player::new();

    let mut editor = Editor::Limited {
        last_selected: None,
    };

    let mut editor_enabled = false;

    let mut update_time = 0.0;

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            window::set_fullscreen(fullscreen);
        }

        if editor_enabled {
            if input::is_mouse_button_pressed(MouseButton::Left) {
                let mouse_position =
                    <[f32; 2]>::from(camera.screen_to_world(input::mouse_position().into()));

                let mouse_position = [
                    mouse_position[0] + LOGICAL_SCREEN_WIDTH / 2.0,
                    mouse_position[1] + LOGICAL_SCREEN_HEIGHT / 2.0,
                ];

                if let Ok(mouse_index) = levels.index_of_position(mouse_position) {
                    let tile_index = levels.index_of(mouse_index).unwrap();

                    if editor.toggle_tile_index(tile_index, &mut levels, &mut player) {
                        fs::write(PATH_TO_LEVELS, levels.to_string()).unwrap();
                    }
                }
            }

            if input::is_key_pressed(KeyCode::M) {
                editor = match editor {
                    Editor::Limited { .. } => {
                        editor.force_undo_temporary_actions(&mut levels);
                        Editor::Full
                    }
                    Editor::Full => Editor::Limited {
                        last_selected: None,
                    },
                }
            }
        }

        if input::is_key_pressed(KeyCode::N) {
            editor_enabled ^= true;
        }

        if input::is_key_down(KeyCode::RightShift) || input::is_key_down(KeyCode::LeftShift) {
            if input::is_key_pressed(KeyCode::I) {
                levels.insert_level(levels.level_index + 1);

                fs::write(PATH_TO_LEVELS, levels.to_string()).unwrap();
            }

            if input::is_key_pressed(KeyCode::R) && levels.num_levels > 1 {
                levels.remove_level((levels.level_index + 1) % levels.num_levels);

                fs::write(PATH_TO_LEVELS, levels.to_string()).unwrap();
            }
        }

        update_time += macroquad::time::get_frame_time() * Player::UPDATES_PER_SECOND;
        let updates = (update_time as usize).min(Player::MAXIMUM_UPDATES_PER_FRAME);

        player.update_input();

        for _ in 0..updates {
            player.update(&mut levels);
        }

        update_time -= updates as f32;
        update_time = update_time.min(1.0);

        let [_, window_height] = update_camera(&mut camera);
        camera::set_camera(&camera);

        window::clear_background(Color::from_hex(0x111111));

        shapes::draw_rectangle(
            -LOGICAL_SCREEN_WIDTH / 2.0,
            LOGICAL_SCREEN_HEIGHT / 2.0,
            LOGICAL_SCREEN_WIDTH,
            (window_height - LOGICAL_SCREEN_HEIGHT) / 2.0,
            colors::WHITE,
        );

        shapes::draw_rectangle(
            -LOGICAL_SCREEN_WIDTH / 2.0,
            -window_height / 2.0,
            LOGICAL_SCREEN_WIDTH,
            window_height - (window_height - LOGICAL_SCREEN_HEIGHT) / 2.0,
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

        shapes::draw_rectangle(
            player.position[0] - Player::SIZE / 2.0 - LOGICAL_SCREEN_WIDTH / 2.0,
            player.position[1] - Player::SIZE / 2.0 - LOGICAL_SCREEN_HEIGHT / 2.0,
            Player::SIZE,
            Player::SIZE,
            match player.air_kind {
                true => colors::WHITE,
                false => colors::BLACK,
            },
        );

        window::next_frame().await;
    }
}

#[derive(Clone, Debug)]
pub enum Editor {
    Limited { last_selected: Option<usize> },
    Full,
}

impl Editor {
    /// Returns whether or not to write the changes made
    #[must_use]
    pub fn toggle_tile_index(
        &mut self,
        tile_index: usize,
        levels: &mut Levels,
        player: &mut Player,
    ) -> bool {
        if let Editor::Limited { .. } = self {
            if levels.level_index == levels.num_levels - 1 || tile_index < Levels::LEVEL_HEIGHT {
                return false;
            }
        }

        levels.tiles[tile_index] ^= true;

        if player.is_intersecting(levels) {
            levels.tiles[tile_index] ^= true;
            return false;
        }

        match self {
            Editor::Limited { last_selected } => {
                if *last_selected == Some(tile_index) {
                    *last_selected = None;
                } else if let Some(last_selected) = last_selected {
                    levels.tiles[*last_selected] ^= true;

                    if player.is_intersecting(levels) {
                        levels.tiles[tile_index] ^= true;
                        levels.tiles[*last_selected] ^= true;
                        return false;
                    }

                    *last_selected = tile_index;
                } else {
                    *last_selected = Some(tile_index);
                }

                false
            }
            Editor::Full => true,
        }
    }

    pub fn force_undo_temporary_actions(&mut self, levels: &mut Levels) {
        match self {
            Editor::Limited { last_selected } => {
                if let Some(tile_index) = *last_selected {
                    levels.tiles[tile_index] ^= true;
                    *last_selected = None;
                }
            }
            Editor::Full => {}
        }
    }
}

fn update_camera(camera: &mut Camera2D) -> [f32; 2] {
    let window_width = get_window_width();
    let window_height = get_window_height();

    camera.zoom.x = 2.0 / window_width;
    camera.zoom.y = -2.0 / window_height;

    [window_width, window_height]
}

fn get_window_width() -> f32 {
    let window_aspect = window::screen_width() / window::screen_height();

    if window_aspect < SCREEN_ASPECT {
        SCREEN_WIDTH
    } else {
        SCREEN_HEIGHT * window_aspect
    }
}

fn get_window_height() -> f32 {
    let window_aspect = window::screen_width() / window::screen_height();

    if window_aspect > SCREEN_ASPECT {
        SCREEN_HEIGHT
    } else {
        SCREEN_WIDTH / window_aspect
    }
}
