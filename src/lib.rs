use std::marker::PhantomData;

use bevy::prelude::*;

pub mod bundle;
pub mod compressible;
pub mod humanize;
pub mod lzw;
pub mod material;
pub mod mono_bundle;
pub mod network;
pub mod proto;
pub mod registry;
pub mod storage;
pub mod utils;

pub const CHUNK_SIZE: usize = 32;

// CHUNK_VOLUME=32768
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const REGION_SIZE: usize = CHUNK_SIZE * 32;

pub const REGION_VOLUME: usize = REGION_SIZE * REGION_SIZE * REGION_SIZE;

pub const TICKS_PER_SECOND: f64 = 50.0;
pub const DELAY_BETWEEN_TICKS: f64 = 1.0 / TICKS_PER_SECOND;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Expectations<T> {
    _expecatations: PhantomData<T>,
}

impl Expectations<()> {
    pub fn of<T>() -> Expectations<T> {
        Expectations {
            _expecatations: PhantomData,
        }
    }
}

pub trait ItMustSend {
    fn it_must_send(&self) -> &Self;
}

impl<T> ItMustSend for Expectations<T>
where
    T: Send,
{
    fn it_must_send(&self) -> &Self {
        self
    }
}

#[repr(u8)]
pub enum AxialRotation {
    Zero = 0,
    By90 = 1,
    By180 = 2,
    By270 = 3,
}

impl AxialRotation {
    fn from_index(index: u32) -> AxialRotation {
        match index {
            0 => AxialRotation::Zero,
            1 => AxialRotation::By90,
            2 => AxialRotation::By180,
            3 => AxialRotation::By270,
            _ => panic!("there is no axial rotation with index {}", index),
        }
    }

    fn to_index(&self) -> u32 {
        match self {
            AxialRotation::Zero => 0,
            AxialRotation::By90 => 1,
            AxialRotation::By180 => 2,
            AxialRotation::By270 => 3,
        }
    }

    fn flip(&self) -> AxialRotation {
        match self {
            AxialRotation::Zero => AxialRotation::Zero,
            AxialRotation::By90 => AxialRotation::By270,
            AxialRotation::By180 => AxialRotation::By180,
            AxialRotation::By270 => AxialRotation::By90,
        }
    }

    fn clockwise(&self) -> AxialRotation {
        //likely faster than using 'index + 1 % 4'
        match self {
            AxialRotation::Zero => AxialRotation::By90,
            AxialRotation::By90 => AxialRotation::By180,
            AxialRotation::By180 => AxialRotation::By270,
            AxialRotation::By270 => AxialRotation::Zero,
        }
    }
    fn counter_clockwise(&self) -> AxialRotation {
        //likely faster than using 'index + 3 % 4'
        match self {
            AxialRotation::Zero => AxialRotation::By270,
            AxialRotation::By90 => AxialRotation::Zero,
            AxialRotation::By180 => AxialRotation::By90,
            AxialRotation::By270 => AxialRotation::By180,
        }
    }
}

/// Indicates the face of a block, chunk or some other block-like object.
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Component)]
pub enum Direction {
    #[default]
    East = DIRECTION_EAST as u8,
    West = DIRECTION_WEST as u8,
    Up = DIRECTION_UP as u8,
    Down = DIRECTION_DOWN as u8,
    North = DIRECTION_NORTH as u8,
    South = DIRECTION_SOUTH as u8,
}

impl Direction {
    pub fn get_index(&self) -> u32 {
        match self {
            Direction::Up => DIRECTION_UP,
            Direction::Down => DIRECTION_DOWN,
            Direction::East => DIRECTION_EAST,
            Direction::West => DIRECTION_WEST,
            Direction::South => DIRECTION_SOUTH,
            Direction::North => DIRECTION_NORTH,
        }
    }
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::South => Direction::North,
            Direction::North => Direction::South,
        }
    }

    pub fn from_index(index: u32) -> Direction {
        match index {
            DIRECTION_UP => Direction::Up,
            DIRECTION_DOWN => Direction::Down,
            DIRECTION_EAST => Direction::East,
            DIRECTION_WEST => Direction::West,
            DIRECTION_SOUTH => Direction::South,
            DIRECTION_NORTH => Direction::North,
            _ => panic!("there is no direction with index {}", index),
        }
    }
    pub fn is_negative(&self) -> bool {
        match self {
            Direction::East | Direction::Up | Direction::North => false,
            Direction::West | Direction::Down | Direction::South => true,
        }
    }

    pub fn get_vector_values(&self) -> (i32, i32, i32) {
        match self {
            Direction::East => (1, 0, 0),
            Direction::West => (-1, 0, 0),
            Direction::Up => (0, 1, 0),
            Direction::Down => (0, -1, 0),
            Direction::North => (0, 0, 1),
            Direction::South => (0, 0, -1),
        }
    }

    pub fn inner_perspective_faced_index(&self, index: usize, dimension_size: usize) -> usize {
        assert!(
            index < dimension_size * dimension_size,
            "index must not exceed 2D ({}^2 = {}) but was {}",
            dimension_size,
            dimension_size * dimension_size,
            index
        );
        if dimension_size == 1 {
            return 0;
        }
        let (surface_x, surface_y) = (index % dimension_size, index / dimension_size);
        let (x, y, z) = match self {
            Direction::East => (surface_x, surface_y, 0),
            Direction::West => (
                dimension_size - 1 - surface_x,
                surface_y,
                dimension_size - 1,
            ),
            Direction::Up => (
                surface_x,
                dimension_size - 1,
                dimension_size - 1 - surface_y,
            ),
            Direction::Down => (surface_x, 0, surface_y),
            Direction::North => (surface_x, surface_y, dimension_size - 1),
            Direction::South => (dimension_size - 1 - surface_x, surface_y, 0),
        };
        //result (cords back to index)
        x + y * dimension_size + z * dimension_size * dimension_size
    }

    #[inline]
    pub fn outer_perspective_faced_index(&self, size: usize, index: usize) -> usize {
        self.opposite().inner_perspective_faced_index(size, index)
    }
}

pub const DIRECTION_EAST: u32 = 0;
pub const DIRECTION_WEST: u32 = 1;
pub const DIRECTION_UP: u32 = 2;
pub const DIRECTION_DOWN: u32 = 3;
pub const DIRECTION_NORTH: u32 = 4;
pub const DIRECTION_SOUTH: u32 = 5;

//TODO store mirroring bits
/// represents all possible rottations of a block in a single byte
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    Component,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Positioning(u8);

impl Default for Positioning {
    fn default() -> Self {
        Self::new(Direction::Up, AxialRotation::Zero)
    }
}

impl Positioning {
    pub fn new(direction: Direction, axial_rotation: AxialRotation) -> Positioning {
        Positioning((direction.get_index() | (axial_rotation.to_index() << 3)) as u8)
    }

    pub fn direction(&self) -> Direction {
        Direction::from_index((self.0 & 0b00000111) as u32)
    }
    pub fn get_axial_rotation(&self) -> AxialRotation {
        AxialRotation::from_index(((self.0 & 0b00011000) >> 3) as u32)
    }
}

pub trait WithFixedSizeExt<T> {
    fn into_fixed_size<const SIZE: usize>(self) -> [T; SIZE];
}

impl<T> WithFixedSizeExt<T> for Vec<T> {
    #[inline]
    fn into_fixed_size<const SIZE: usize>(self) -> [T; SIZE] {
        self.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Expected a Vec of length {} but it was {}", SIZE, v.len())
        })
    }
}

#[cfg(test)]
mod test_perspective_faced_index {
    use std::ops::Range;

    const DIMENSION_SIZE: usize = 4;
    const DIMENSION_FACE_SURFACE: usize = DIMENSION_SIZE * DIMENSION_SIZE;
    const DIMENSION_VOLUME: usize = DIMENSION_SIZE * DIMENSION_SIZE * DIMENSION_SIZE;
    const INPUT_RANGE: Range<usize> = 0..DIMENSION_FACE_SURFACE;

    //@formatter:off
    #[rustfmt::skip]
    const EXPECTED_EAST: [usize; DIMENSION_FACE_SURFACE] = [
        0,  1,  2,  3,
        4,  5,  6,  7,
        8,  9,  10, 11,
        12, 13, 14, 15,
    ];
    #[rustfmt::skip]
    const EXPECTED_WEST: [usize; DIMENSION_FACE_SURFACE] = [
        48, 49, 50, 51,
        52, 53, 54, 55,
        56, 57, 58, 59,
        60, 61, 62, 63,
    ];
    #[rustfmt::skip]
    const EXPECTED_BOTTOM: [usize; DIMENSION_FACE_SURFACE] = [
        0,  1,  2,  3,
        16, 17, 18, 19,
        32, 33, 34, 35,
        48, 49, 50, 51,
    ];

    #[rustfmt::skip]
    const EXPECTED_TOP: [usize; DIMENSION_FACE_SURFACE] = [
        12, 13, 14, 15,
        28, 29, 30, 31,
        44, 45, 46, 47,
        60, 61, 62, 63,
    ];

    #[rustfmt::skip]
    const EXPECTED_NORTH: [usize; DIMENSION_FACE_SURFACE] = [
        0,  4,  8,  12,
        16, 20, 24, 28,
        32, 36, 40, 44,
        48, 52, 56, 60,
    ];

    #[rustfmt::skip]
    const EXPECTED_SOUTH: [usize; DIMENSION_FACE_SURFACE] = [
        48, 49, 50, 51,
        52, 53, 54, 55,
        56, 57, 58, 59,
        60, 61, 62, 63
    ];

    //@formatter:on

    /*
    #[test]
    fn test_east() {
        let face = Direction::East;
        let result: Vec<usize> = INPUT_RANGE
            .map(|index| face.inner_perspective_faced_index(index, DIMENSION_SIZE))
            .collect();
        assert_eq!(result, EXPECTED_EAST);
    }

    #[test]
    fn test_west() {
        let face = Direction::West;
        let result: Vec<usize> = INPUT_RANGE
            .map(|index| face.inner_perspective_faced_index(index, DIMENSION_SIZE))
            .collect();
        assert_eq!(result, EXPECTED_WEST);
    }

    #[test]
    fn test_bottom() {
        let face = Direction::Down;
        let result: Vec<usize> = INPUT_RANGE
            .map(|index| face.inner_perspective_faced_index(index, DIMENSION_SIZE))
            .collect();
        assert_eq!(result, EXPECTED_BOTTOM);
    }

    #[test]
    fn test_top() {
        let face = Direction::Up;
        let result: Vec<usize> = INPUT_RANGE
            .map(|index| face.inner_perspective_faced_index(index, DIMENSION_SIZE))
            .collect();
        assert_eq!(result, EXPECTED_TOP);
    }

    #[test]
    fn test_north() {
        let face = Direction::South;
        let result: Vec<usize> = INPUT_RANGE
            .map(|index| face.inner_perspective_faced_index(index, DIMENSION_SIZE))
            .collect();
        assert_eq!(result, EXPECTED_NORTH);
    }

    #[test]
    fn test_south() {
        let face = Direction::North;
        let result: Vec<usize> = INPUT_RANGE
            .map(|index| face.inner_perspective_faced_index(index, DIMENSION_SIZE))
            .collect();
        assert_eq!(result, EXPECTED_SOUTH);
    }
     */
}
