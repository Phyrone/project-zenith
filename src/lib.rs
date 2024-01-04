use std::marker::PhantomData;

use bevy::prelude::*;

pub mod chunk;
pub mod compressible;
pub mod humanize;
pub mod material;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Component)]
pub enum Face {
    Top = FACE_TOP as u8,
    Bottom = FACE_BOTTOM as u8,
    East = FACE_EAST as u8,
    West = FACE_WEST as u8,
    North = FACE_NORTH as u8,
    South = FACE_SOUTH as u8,
}

impl Face {
    pub fn get_face_index(&self) -> u32 {
        match self {
            Face::Top => FACE_TOP,
            Face::Bottom => FACE_BOTTOM,
            Face::East => FACE_EAST,
            Face::West => FACE_WEST,
            Face::North => FACE_NORTH,
            Face::South => FACE_SOUTH,
        }
    }
    fn get_opposite_face(&self) -> Face {
        match self {
            Face::Top => Face::Bottom,
            Face::Bottom => Face::Top,
            Face::East => Face::West,
            Face::West => Face::East,
            Face::North => Face::South,
            Face::South => Face::North,
        }
    }

    fn from_index(index: u32) -> Face {
        match index {
            FACE_TOP => Face::Top,
            FACE_BOTTOM => Face::Bottom,
            FACE_EAST => Face::East,
            FACE_WEST => Face::West,
            FACE_NORTH => Face::North,
            FACE_SOUTH => Face::South,
            _ => panic!("there is no face with index {}", index),
        }
    }

    fn opposite_face_index(index: u32) -> u32 {
        match index {
            FACE_TOP => FACE_BOTTOM,
            FACE_BOTTOM => FACE_TOP,
            FACE_EAST => FACE_WEST,
            FACE_WEST => FACE_EAST,
            FACE_NORTH => FACE_SOUTH,
            FACE_SOUTH => FACE_NORTH,
            _ => panic!("there is no face with index {}", index),
        }
    }
}

pub const FACE_TOP: u32 = 0;
pub const FACE_BOTTOM: u32 = 1;
pub const FACE_EAST: u32 = 2;
pub const FACE_WEST: u32 = 3;
pub const FACE_NORTH: u32 = 4;
pub const FACE_SOUTH: u32 = 5;

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
pub struct BlockRotation(u8);

impl Default for BlockRotation {
    fn default() -> Self {
        Self::new(Face::Top, AxialRotation::Zero)
    }
}

impl BlockRotation {
    pub fn new(face: Face, axial_rotation: AxialRotation) -> BlockRotation {
        BlockRotation((face.get_face_index() | (axial_rotation.to_index() << 3)) as u8)
    }

    pub fn get_face(&self) -> Face {
        Face::from_index((self.0 & 0b00000111) as u32)
    }
    pub fn get_axial_rotation(&self) -> AxialRotation {
        AxialRotation::from_index(((self.0 & 0b00011000) >> 3) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
