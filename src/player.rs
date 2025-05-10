use std::{array, sync::LazyLock};

use macroquad::input::{self, KeyCode};

use crate::level::Levels;

pub struct Player {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub air_kind: bool,
    pub on_ground: bool,
    pub cyote_time: u8,
    pub inputs_down: [bool; 4],
    pub inputs_ready: [bool; 4],
}

impl Player {
    pub const UPDATES_PER_SECOND: f32 = 60.0;
    pub const UPS_SCALE: f32 = Self::UPDATES_PER_SECOND / 30.0;

    pub const SIZE: f32 = 0.5;
    pub const GRAVITY: f32 = 1.0 / 32.0 / Self::UPS_SCALE / Self::UPS_SCALE;

    pub const MAXIMUM_UPDATES_PER_FRAME: usize = 5;

    pub const CYOTE_FRAMES: u8 = (0.05 * Self::UPDATES_PER_SECOND) as u8;

    pub fn new() -> Self {
        Self {
            position: [
                crate::LOGICAL_SCREEN_WIDTH / 2.0,
                crate::LOGICAL_SCREEN_HEIGHT / 2.0,
            ],
            velocity: [0.0, 0.0],
            air_kind: false,
            on_ground: false,
            cyote_time: 0,
            inputs_down: [false; 4],
            inputs_ready: [false; 4],
        }
    }

    pub fn update_input(&mut self) {
        static KEYBINDS: LazyLock<[Vec<KeyCode>; 4]> = LazyLock::new(|| {
            [
                vec![KeyCode::W, KeyCode::Up, KeyCode::Space],
                vec![KeyCode::A, KeyCode::Left],
                vec![KeyCode::S, KeyCode::Down],
                vec![KeyCode::D, KeyCode::Right],
            ]
        });

        fn is_down(keys: &[KeyCode]) -> bool {
            keys.iter().any(|key| input::is_key_down(*key))
        }

        fn is_pressed(keys: &[KeyCode]) -> bool {
            keys.iter().any(|key| input::is_key_pressed(*key))
        }

        self.inputs_down = array::from_fn(|i| self.inputs_down[i] || is_down(&KEYBINDS[i]));
        self.inputs_ready = array::from_fn(|i| {
            (self.inputs_ready[i] || is_pressed(&KEYBINDS[i])) && self.inputs_down[i]
        });
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

        if self.cyote_time > 0 {
            self.cyote_time -= 1;
        }

        if y_collision {
            if self.velocity[1] * self.gravity() > 0.0 {
                self.on_ground = true;
                self.cyote_time = Self::CYOTE_FRAMES;
            }

            self.velocity[1] = 0.0;
        }

        if self.inputs_ready[0] && (self.cyote_time > 0 || self.on_ground) {
            self.inputs_ready[0] = false;

            self.velocity[1] = -7.5 * Self::UPS_SCALE * self.gravity();
        }

        let x_input = self.inputs_down[3] as isize - self.inputs_down[1] as isize;

        self.velocity[0] *= 1.0 - 0.2 / Self::UPS_SCALE;
        self.velocity[0] += x_input as f32 / 32.0 / Self::UPS_SCALE / Self::UPS_SCALE;

        if self.on_ground && self.inputs_ready[2] {
            self.inputs_ready[2] = false;

            let old_position = self.position;

            match self.air_kind {
                true => self.position[1] += Self::SIZE,
                false => self.position[1] -= Self::SIZE,
            }

            self.air_kind ^= true;

            if self.is_intersecting(levels) {
                self.position = old_position;
                self.air_kind ^= true;
            }
        }

        self.inputs_down = [false; 4];
    }

    pub fn gravity(&self) -> f32 {
        match self.air_kind {
            true => Self::GRAVITY,
            false => -Self::GRAVITY,
        }
    }

    pub fn is_intersecting(&mut self, levels: &Levels) -> bool {
        match self.move_by(levels, [0.0, 0.0]) {
            Some(collision) => collision,
            None => true,
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
