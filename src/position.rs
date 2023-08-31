use std::{
    cmp::Ordering,
    hash::Hash,
    ops::{Add, Sub},
};

use super::Buffer;

#[derive(Copy, Clone, Debug, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    pub fn as_uposition(&self) -> UPosition {
        UPosition {
            x: self.x as usize,
            y: self.y as usize,
        }
    }

    pub fn from_index(buf: &Buffer, i: i32) -> Self {
        Position {
            x: i % buf.get_width() as i32,
            y: i / buf.get_width() as i32,
        }
    }

    pub fn with_y(self, y: i32) -> Position {
        Position { x: self.x, y }
    }

    pub fn with_x(self, x: i32) -> Position {
        Position { x, y: self.y }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Position {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Position {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.y < other.y {
            return Some(Ordering::Less);
        }
        if self.y > other.y {
            return Some(Ordering::Greater);
        }
        if self.x < other.x {
            return Some(Ordering::Less);
        }
        if self.x > other.x {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}

impl From<(usize, usize)> for UPosition {
    fn from(value: (usize, usize)) -> Self {
        UPosition {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<(i32, i32)> for UPosition {
    fn from(value: (i32, i32)) -> Self {
        UPosition {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<(u32, u32)> for UPosition {
    fn from(value: (u32, u32)) -> Self {
        UPosition {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<(u8, u8)> for UPosition {
    fn from(value: (u8, u8)) -> Self {
        UPosition {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}

impl From<Position> for UPosition {
    fn from(value: Position) -> Self {
        UPosition {
            x: value.x as usize,
            y: value.y as usize,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq)]
pub struct UPosition {
    pub x: usize,
    pub y: usize,
}

impl Hash for UPosition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl std::fmt::Display for UPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl UPosition {
    pub fn new(x: usize, y: usize) -> Self {
        UPosition { x, y }
    }

    pub fn from_index(buf: &Buffer, i: usize) -> Self {
        UPosition {
            x: i % buf.get_width(),
            y: i / buf.get_width(),
        }
    }

    pub fn with_y(self, y: usize) -> UPosition {
        UPosition { x: self.x, y }
    }

    pub fn with_x(self, x: usize) -> UPosition {
        UPosition { x, y: self.y }
    }
}

impl Default for UPosition {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Add<UPosition> for UPosition {
    type Output = UPosition;

    fn add(self, rhs: UPosition) -> UPosition {
        UPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<UPosition> for UPosition {
    type Output = UPosition;

    fn sub(self, rhs: UPosition) -> UPosition {
        UPosition {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<Position> for UPosition {
    type Output = UPosition;

    fn sub(self, rhs: Position) -> UPosition {
        let x = (self.x as i32) - rhs.x;
        let y = (self.y as i32) - rhs.y;
        assert!(!(x < 0 || y < 0), "Negative position");
        UPosition {
            x: x as usize,
            y: y as usize,
        }
    }
}

impl PartialEq for UPosition {
    fn eq(&self, other: &UPosition) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl PartialOrd for UPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.y < other.y {
            return Some(Ordering::Less);
        }
        if self.y > other.y {
            return Some(Ordering::Greater);
        }
        if self.x < other.x {
            return Some(Ordering::Less);
        }
        if self.x > other.x {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}
