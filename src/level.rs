use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Levels {
    pub tiles: Vec<bool>,
    pub num_levels: usize,
    pub level_index: usize,
    pub x_offset: usize,
    pub limited_gem: Option<usize>,
    pub full_gem: Option<usize>,
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
            limited_gem: None,
            full_gem: None,
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

    pub fn position_of_tile_index(&self, tile_index: usize) -> Option<[f32; 2]> {
        let x = tile_index / Self::LEVEL_HEIGHT;
        let y = tile_index % Self::LEVEL_HEIGHT;

        if x >= self.x_offset && x < self.x_offset + Self::LEVEL_WIDTH {
            Some([x as f32, y as f32])
        } else if x == 0 && self.level_index == self.num_levels - 1 {
            Some([(Self::LEVEL_WIDTH - 1) as f32, y as f32])
        } else {
            None
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

    pub fn remove_level(&mut self, index: usize) {
        assert!(index < self.num_levels);

        self.num_levels -= 1;

        if self.level_index > index {
            self.previous_level();
        }

        let offset = Self::offset_of_level(index);

        for _ in 0..(Self::LEVEL_WIDTH - 1) * Self::LEVEL_HEIGHT {
            self.tiles.remove(offset);
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

impl Display for Levels {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..Self::LEVEL_HEIGHT).rev() {
            for x in 0..(Self::LEVEL_WIDTH - 1) * self.num_levels {
                let tile_index = x * Self::LEVEL_HEIGHT + y;

                if Some(tile_index) == self.limited_gem {
                    write!(f, "e")?;
                    continue;
                }

                if Some(tile_index) == self.full_gem {
                    write!(f, "E")?;
                    continue;
                }

                let tile = self.tiles[x * Self::LEVEL_HEIGHT + y];

                write!(
                    f,
                    "{}",
                    match tile {
                        true => 'x',
                        false => ' ',
                    }
                )?;
            }

            write!(f, "|\n")?;
        }

        Ok(())
    }
}

impl FromStr for Levels {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tiles = Vec::new();

        let mut limited_gem = None;
        let mut full_gem = None;

        let mut lines = s
            .lines()
            .map(|line| line.chars().peekable())
            .collect::<Box<[_]>>();

        if lines.len() != Self::LEVEL_HEIGHT {
            return Err(ParseLevelError::InvalidHeight);
        }

        loop {
            for (i, line) in lines.iter_mut().enumerate().rev() {
                let Some(character) = line.next() else {
                    return Err(ParseLevelError::LineEndsEarly(i));
                };

                let tile = match character {
                    ' ' => false,
                    'x' => true,
                    'e' => {
                        if limited_gem.is_none() {
                            limited_gem = Some(tiles.len());
                        } else {
                            return Err(ParseLevelError::DuplicateGem('e'));
                        }

                        false
                    }
                    'E' => {
                        if full_gem.is_none() {
                            full_gem = Some(tiles.len());
                        } else {
                            return Err(ParseLevelError::DuplicateGem('E'));
                        }

                        false
                    }
                    character => {
                        return Err(ParseLevelError::InvalidTileCharacter(character));
                    }
                };

                tiles.push(tile);
            }

            if lines[0].peek() == Some(&'|') {
                for (i, mut line) in lines.into_iter().enumerate() {
                    let next = line.next();

                    match next {
                        Some('|') => {
                            if line.next().is_some() {
                                return Err(ParseLevelError::InvalidTileCharacter('|'));
                            }
                        }
                        Some(character) => {
                            return Err(ParseLevelError::InvalidEndingCharacter(character));
                        }
                        None => {
                            return Err(ParseLevelError::LineEndsEarly(i));
                        }
                    }
                }

                break;
            }
        }

        const LEVEL_TILES: usize = (Levels::LEVEL_WIDTH - 1) * Levels::LEVEL_HEIGHT;

        if tiles.len() % LEVEL_TILES != 0 {
            return Err(ParseLevelError::InvalidWidth);
        }

        let num_levels = tiles.len() / LEVEL_TILES;

        Ok(Self {
            tiles,
            num_levels,
            level_index: 0,
            x_offset: 0,
            limited_gem,
            full_gem,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ParseLevelError {
    InvalidHeight,
    InvalidWidth,
    InvalidTileCharacter(char),
    InvalidEndingCharacter(char),
    LineEndsEarly(usize),
    DuplicateGem(char),
}
