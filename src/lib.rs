use std::marker::PhantomData;

use bevy::prelude::*;

pub mod chunk;
pub mod compressible;

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
    One = 1,
    Two = 2,
    Three = 3,
}

impl AxialRotation {
    fn from_index(index: u32) -> AxialRotation {
        match index {
            0 => AxialRotation::Zero,
            1 => AxialRotation::One,
            2 => AxialRotation::Two,
            3 => AxialRotation::Three,
            _ => panic!("there is no axial rotation with index {}", index),
        }
    }

    fn to_index(&self) -> u32 {
        match self {
            AxialRotation::Zero => 0,
            AxialRotation::One => 1,
            AxialRotation::Two => 2,
            AxialRotation::Three => 3,
        }
    }

    fn flip(&self) -> AxialRotation {
        match self {
            AxialRotation::Zero => AxialRotation::Zero,
            AxialRotation::One => AxialRotation::Three,
            AxialRotation::Two => AxialRotation::Two,
            AxialRotation::Three => AxialRotation::One,
        }
    }

    fn clockwise(&self) -> AxialRotation {
        //likely faster than using 'index + 1 % 4'
        match self {
            AxialRotation::Zero => AxialRotation::One,
            AxialRotation::One => AxialRotation::Two,
            AxialRotation::Two => AxialRotation::Three,
            AxialRotation::Three => AxialRotation::Zero,
        }
    }
    fn counter_clockwise(&self) -> AxialRotation {
        //likely faster than using 'index + 3 % 4'
        match self {
            AxialRotation::Zero => AxialRotation::Three,
            AxialRotation::One => AxialRotation::Zero,
            AxialRotation::Two => AxialRotation::One,
            AxialRotation::Three => AxialRotation::Two,
        }
    }
}

/// Indicates the face of a block, chunk or some other block-like object.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Component)]
pub enum Face {
    Top = FACE_TOP as u8,
    Bottom = FACE_BOTTOM as u8,
    Left = FACE_LEFT as u8,
    Right = FACE_RIGHT as u8,
    Front = FACE_FRONT as u8,
    Back = FACE_BACK as u8,
}

impl Face {
    pub fn get_face_index(&self) -> u32 {
        match self {
            Face::Top => FACE_TOP,
            Face::Bottom => FACE_BOTTOM,
            Face::Left => FACE_LEFT,
            Face::Right => FACE_RIGHT,
            Face::Front => FACE_FRONT,
            Face::Back => FACE_BACK,
        }
    }
    fn get_opposite_face(&self) -> Face {
        match self {
            Face::Top => Face::Bottom,
            Face::Bottom => Face::Top,
            Face::Left => Face::Right,
            Face::Right => Face::Left,
            Face::Front => Face::Back,
            Face::Back => Face::Front,
        }
    }

    fn from_index(index: u32) -> Face {
        match index {
            FACE_TOP => Face::Top,
            FACE_BOTTOM => Face::Bottom,
            FACE_LEFT => Face::Left,
            FACE_RIGHT => Face::Right,
            FACE_FRONT => Face::Front,
            FACE_BACK => Face::Back,
            _ => panic!("there is no face with index {}", index),
        }
    }

    fn opposite_face_index(index: u32) -> u32 {
        match index {
            FACE_TOP => FACE_BOTTOM,
            FACE_BOTTOM => FACE_TOP,
            FACE_LEFT => FACE_RIGHT,
            FACE_RIGHT => FACE_LEFT,
            FACE_FRONT => FACE_BACK,
            FACE_BACK => FACE_FRONT,
            _ => panic!("there is no face with index {}", index),
        }
    }
}

pub const FACE_TOP: u32 = 0;
pub const FACE_BOTTOM: u32 = 1;
pub const FACE_LEFT: u32 = 2;
pub const FACE_RIGHT: u32 = 3;
pub const FACE_FRONT: u32 = 4;
pub const FACE_BACK: u32 = 5;

//TODO store mirroring bits
/// stores the rotation of a block or block-like object. it only needs a single byte because there are only 6 faces and 4 axial rotations.
/// and so 6* 4 = 24 possible combinations. 24 < 256
/// the first 3 bits are used to store the face and the next 2 bits are used to store the axial rotation.
/// the remaining 3 bits are unused yet. but might be used to store mirroring information in the future.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Component)]
#[derive(serde::Serialize, serde::Deserialize)]
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
