use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct Levels {
    pub tiles: Vec<bool>,
    pub num_levels: usize,
    pub level_index: usize,
    pub x_offset: usize,
}

impl Levels {
    pub const LEVEL_WIDTH: usize = 15;
    pub const LEVEL_HEIGHT: usize = 11;

    pub fn new() -> Self {
        Self {
            tiles: vec![false; (Self::LEVEL_WIDTH - 1) * Self::LEVEL_HEIGHT],
            num_levels: 1,
            level_index: 0,
            x_offset: 0,
        }
    }

    pub fn get_from_position(&self, position: [f32; 2]) -> Option<bool> {
        match self.index_of_position(position) {
            Ok(index) => Some(*self.get(index).unwrap()),
            Err([None, Some(IndexingError::TooBig)]) => Some(false),
            Err([None, Some(IndexingError::TooSmall)]) => Some(true),
            _ => None,
        }
    }

    pub fn index_of_position(
        &self,
        position: [f32; 2],
    ) -> Result<[usize; 2], [Option<IndexingError>; 2]> {
        let mut error = [None; 2];

        if position[0] < 0.0 {
            error[0] = Some(IndexingError::TooSmall);
        } else if position[0] >= crate::LOGICAL_SCREEN_WIDTH {
            error[0] = Some(IndexingError::TooBig);
        }

        if position[1] < 0.0 {
            error[1] = Some(IndexingError::TooSmall);
        } else if position[1] >= crate::LOGICAL_SCREEN_HEIGHT {
            error[1] = Some(IndexingError::TooBig);
        }

        if let [None, None] = error {
            Ok([
                (position[0] as usize).min(Self::LEVEL_WIDTH - 1),
                (position[1] as usize).min(Self::LEVEL_HEIGHT - 1),
            ])
        } else {
            Err(error)
        }
    }

    pub fn get(&self, index: [usize; 2]) -> Option<&bool> {
        let tile_index = self.index_of(index)?;

        Some(&self.tiles[tile_index])
    }

    pub fn get_mut(&mut self, index: [usize; 2]) -> Option<&mut bool> {
        let tile_index = self.index_of(index)?;

        Some(&mut self.tiles[tile_index])
    }

    pub fn index_of(&self, index: [usize; 2]) -> Option<usize> {
        if self.is_index_in_bounds(index) {
            Some(unsafe { self.index_of_unchecked(index) })
        } else {
            None
        }
    }

    unsafe fn index_of_unchecked(&self, index: [usize; 2]) -> usize {
        let overflowing_index = (index[0] + self.x_offset) * Self::LEVEL_HEIGHT + index[1];

        overflowing_index % self.tiles.len()
    }

    fn is_index_in_bounds(&self, index: [usize; 2]) -> bool {
        index[0] <= Self::LEVEL_WIDTH && index[1] <= Self::LEVEL_HEIGHT
    }

    pub fn next_level(&mut self) {
        self.level_index += 1;
        self.level_index %= self.num_levels;

        self.update_level_offset();
    }

    pub fn previous_level(&mut self) {
        if self.level_index == 0 {
            self.level_index = self.num_levels - 1;
        } else {
            self.level_index -= 1;
        }

        self.update_level_offset();
    }

    pub fn insert_level(&mut self, index: usize) {
        self.num_levels += 1;

        assert!(index < self.num_levels);

        if self.level_index >= index {
            self.next_level();
        }

        let mut offset = Self::offset_of_level(index);

        const _: () = assert!(Levels::LEVEL_HEIGHT >= 5);

        for _ in 0..(Self::LEVEL_WIDTH - 1) {
            for _ in 0..5 {
                self.tiles.insert(offset, true);
                offset += 1;
            }

            for _ in 0..Self::LEVEL_HEIGHT - 5 {
                self.tiles.insert(offset, false);
                offset += 1;
            }
        }
    }

    pub fn update_level_offset(&mut self) {
        self.x_offset = self.level_index * (Self::LEVEL_WIDTH - 1);
    }

    fn offset_of_level(level_index: usize) -> usize {
        level_index * (Self::LEVEL_WIDTH - 1) * Self::LEVEL_HEIGHT
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IndexingError {
    TooBig,
    TooSmall,
}

impl Index<[usize; 2]> for Levels {
    type Output = bool;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl IndexMut<[usize; 2]> for Levels {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
