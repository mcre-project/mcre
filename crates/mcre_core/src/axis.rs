use serde::{Deserialize, Serialize};

use crate::VecN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignedAxis {
    Xn,
    Xp,
    Yn,
    Yp,
    Zn,
    Zp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5,
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::Down,
        Direction::Up,
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ];

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }

    pub fn is_positive(self) -> bool {
        match self {
            Direction::Down | Direction::North | Direction::West => false,
            Direction::Up | Direction::South | Direction::East => true,
        }
    }

    pub fn axis(self) -> Axis {
        match self {
            Direction::Down | Direction::Up => Axis::Y,
            Direction::North | Direction::South => Axis::Z,
            Direction::West | Direction::East => Axis::X,
        }
    }
}

impl Axis {
    pub fn select<T: Copy>(self, vec: VecN<T, 3>) -> T {
        match self {
            Axis::X => vec[0],
            Axis::Y => vec[1],
            Axis::Z => vec[2],
        }
    }

    pub fn select_mut<T: Copy>(self, vec: &mut VecN<T, 3>) -> &mut T {
        match self {
            Axis::X => &mut vec[0],
            Axis::Y => &mut vec[1],
            Axis::Z => &mut vec[2],
        }
    }

    pub fn complementary_axes(self) -> [Axis; 2] {
        match self {
            Axis::X => [Axis::Y, Axis::Z],
            Axis::Y => [Axis::X, Axis::Z],
            Axis::Z => [Axis::X, Axis::Y],
        }
    }
}
