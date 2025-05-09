use std::array;

use macroquad::input::{self, KeyCode};

use crate::level::Levels;

pub struct Player {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub air_kind: bool,
    pub on_ground: bool,
    pub inputs_down: [bool; 4],
    pub inputs_pressed: [bool; 4],
}

impl Player {
    pub const SIZE: f32 = 0.5;
    pub const GRAVITY: f32 = 1.0 / 32.0 / 4.0;
    pub const UPDATES_PER_SECOND: f32 = 60.0;
    pub const MAXIMUM_UPDATES_PER_FRAME: usize = 5;

    pub fn new() -> Self {
        Self {
            position: [
                crate::LOGICAL_SCREEN_WIDTH / 2.0,
                crate::LOGICAL_SCREEN_HEIGHT / 2.0,
            ],
            velocity: [0.0, 0.0],
            air_kind: false,
            on_ground: false,
            inputs_down: [false; 4],
            inputs_pressed: [false; 4],
        }
    }

    pub fn update_input(&mut self) {
        const KEYBINDS: [KeyCode; 4] = [KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D];

        self.inputs_down =
            array::from_fn(|i| self.inputs_down[i] || input::is_key_down(KEYBINDS[i]));
        self.inputs_pressed =
            array::from_fn(|i| self.inputs_pressed[i] || input::is_key_pressed(KEYBINDS[i]));
    }

    pub fn update(&mut self, levels: &mut Levels) {
        self.velocity[1] += self.gravity();

        let Some(x_collision) = self.move_by(levels, [self.velocity[0], 0.0]) else {
            if self.position[0] > crate::LOGICAL_SCREEN_WIDTH / 2.0 {
                levels.next_level();
                self.position[0] = Self::SIZE / 2.0;
            } else {
                levels.previous_level();
                self.position[0] = crate::LOGICAL_SCREEN_WIDTH - Self::SIZE / 2.0;
            }

            return;
        };
        let y_collision = self.move_by(levels, [0.0, self.velocity[1]]).unwrap();

        if x_collision {
            self.velocity[0] = 0.0;
        }

        self.on_ground = false;

        'outer: {
            if y_collision {
                if self.velocity[1] * self.gravity() > 0.0 {
                    self.on_ground = true;

                    if self.inputs_down[0] {
                        self.velocity[1] = -15.0 * self.gravity();
                        break 'outer;
                    }
                }

                self.velocity[1] = 0.0;
            }
        }

        let x_input = self.inputs_down[3] as isize - self.inputs_down[1] as isize;

        self.velocity[0] *= 0.9;
        self.velocity[0] += x_input as f32 / 32.0 / 4.0;

        if self.on_ground && self.inputs_pressed[2] {
            let old_position = self.position;

            match self.air_kind {
                true => self.position[1] += Self::SIZE,
                false => self.position[1] -= Self::SIZE,
            }

            self.air_kind ^= true;

            if self.move_by(levels, [0.0, 0.0]).unwrap() {
                self.position = old_position;
                self.air_kind ^= true;
            }
        }

        self.inputs_down = [false; 4];
        self.inputs_pressed = [false; 4];
    }

    pub fn gravity(&self) -> f32 {
        match self.air_kind {
            true => Self::GRAVITY,
            false => -Self::GRAVITY,
        }
    }

    pub fn move_by(&mut self, levels: &Levels, amount: [f32; 2]) -> Option<bool> {
        self.position[0] += amount[0];
        self.position[1] += amount[1];

        let mut collision = false;

        const CORNERS: [[f32; 2]; 4] = [[1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]];

        for corner in CORNERS {
            // Ensure the corner doesn't clip onto the next tile's bounds
            let corner = corner.map(|x| if x == 1.0 { 1.0 - 10e-6 } else { x });

            let corner_position =
                array::from_fn(|i| self.position[i] + corner[i] * Self::SIZE / 2.0);

            if levels.get_from_position(corner_position)? == self.air_kind {
                continue;
            }

            // There is a collision
            if amount[0] != 0.0 {
                if amount[0] > 0.0 {
                    self.position[0] = corner_position[0].floor() - Self::SIZE / 2.0;
                } else {
                    self.position[0] = corner_position[0].floor() + 1.0 + Self::SIZE / 2.0;
                }

                collision = true;
            } else if amount[1] != 0.0 {
                if amount[1] > 0.0 {
                    self.position[1] = corner_position[1].floor() - Self::SIZE / 2.0;
                } else {
                    self.position[1] = corner_position[1].floor() + 1.0 + Self::SIZE / 2.0;
                }

                collision = true;
            } else {
                return Some(true);
            }
        }

        Some(collision)
    }
}
