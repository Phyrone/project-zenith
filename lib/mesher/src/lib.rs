use std::hash::Hash;
use std::ops::{BitAndAssign, Not};

use hashbrown::HashMap;
use smallvec::SmallVec;

pub mod b16;
pub mod b32;
pub mod b64;
#[cfg(feature = "bevy")]
pub mod meshing;

pub type MeshingResult<S> = HashMap<S, SmallVec<[GreedyQuad; 256]>>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FaceDirection {
    ZPos = 0,
    ZNeg = 1,
    YPos = 2,
    YNeg = 3,
    XPos = 4,
    XNeg = 5,
}

impl std::fmt::Display for FaceDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FaceDirection::ZPos => write!(f, "ZPos"),
            FaceDirection::ZNeg => write!(f, "ZNeg"),
            FaceDirection::YPos => write!(f, "YPos"),
            FaceDirection::YNeg => write!(f, "YNeg"),
            FaceDirection::XPos => write!(f, "XPos"),
            FaceDirection::XNeg => write!(f, "XNeg"),
        }
    }
}

impl FaceDirection {
    pub fn to_index(&self) -> usize {
        *self as usize
    }
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => FaceDirection::ZPos,
            1 => FaceDirection::ZNeg,
            2 => FaceDirection::YPos,
            3 => FaceDirection::YNeg,
            4 => FaceDirection::XPos,
            5 => FaceDirection::XNeg,
            _ => panic!("Invalid index"),
        }
    }

    pub const ALL: [FaceDirection; 6] = [
        FaceDirection::ZPos,
        FaceDirection::ZNeg,
        FaceDirection::YPos,
        FaceDirection::YNeg,
        FaceDirection::XPos,
        FaceDirection::XNeg,
    ];

    pub fn opposite(&self) -> Self {
        match self {
            FaceDirection::ZPos => FaceDirection::ZNeg,
            FaceDirection::ZNeg => FaceDirection::ZPos,
            FaceDirection::YPos => FaceDirection::YNeg,
            FaceDirection::YNeg => FaceDirection::YPos,
            FaceDirection::XPos => FaceDirection::XNeg,
            FaceDirection::XNeg => FaceDirection::XPos,
        }
    }

    pub fn is_positive(&self) -> bool {
        match self {
            FaceDirection::ZPos | FaceDirection::YPos | FaceDirection::XPos => true,
            FaceDirection::ZNeg | FaceDirection::YNeg | FaceDirection::XNeg => false,
        }
    }
    #[inline]
    pub fn is_negative(&self) -> bool {
        !self.is_positive()
    }

    pub fn axis_rel_to_absolute(&self, i: usize, j: usize, k: usize) -> (usize, usize, usize) {
        match self {
            FaceDirection::XPos | FaceDirection::XNeg => (k, j, i),
            FaceDirection::YPos | FaceDirection::YNeg => (i, k, j),
            FaceDirection::ZPos | FaceDirection::ZNeg => (i, j, k),
        }
    }

    pub fn absolute_to_axis_rel(&self, x: usize, y: usize, z: usize) -> (usize, usize, usize) {
        match self {
            FaceDirection::XPos | FaceDirection::XNeg => (z, y, x),
            FaceDirection::YPos | FaceDirection::YNeg => (x, z, y),
            FaceDirection::ZPos | FaceDirection::ZNeg => (x, y, z),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GreedyQuad {
    pub direction: FaceDirection,
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub w: usize,
    pub h: usize,
}

impl GreedyQuad {
    pub fn vertex_positions(&self, scaling: f32) -> [[f32; 3]; 4] {
        let (x, y, z) = match self.direction {
            FaceDirection::ZPos => (self.x as f32, self.y as f32, self.z as f32 + 1f32),
            FaceDirection::YPos => (self.x as f32, self.y as f32 + 1f32, self.z as f32),
            FaceDirection::XPos => (self.x as f32 + 1f32, self.y as f32, self.z as f32),
            _ => (self.x as f32, self.y as f32, self.z as f32),
        };

        let (w, h) = (self.w as f32, self.h as f32);
        let (x, y, z) = (x * scaling, y * scaling, z * scaling);
        let (w, h) = (w * scaling, h * scaling);
        match self.direction {
            FaceDirection::ZPos => {
                [
                    [x + w, y, z],
                    [x + w, y + h, z],
                    [x, y, z],
                    [x, y + h, z],
                ]
            }
            FaceDirection::ZNeg => {
                [
                    [x, y, z],
                    [x, y + h, z],
                    [x + w, y, z],
                    [x + w, y + h, z],
                ]
            }
            FaceDirection::XPos => {
                [
                    [x, y, z],
                    [x, y + h, z],
                    [x, y, z + w],
                    [x, y + h, z + w],
                ]
            }
            FaceDirection::XNeg => {
                [
                    [x, y, z + w],
                    [x, y + h, z + w],
                    [x, y, z],
                    [x, y + h, z],
                ]
            }
            FaceDirection::YPos => {
                [
                    [x, y, z],
                    [x, y, z + h],
                    [x + w, y, z],
                    [x + w, y, z + h],
                ]
            }
            FaceDirection::YNeg => {
                [
                    [x, y, z + h],
                    [x, y, z],
                    [x + w, y, z + h],
                    [x + w, y, z],
                ]
            }
        }
    }
}

enum VoxelCubeSize {
    WellKnown8,
    WellKnown16,
    WellKnown32,
    WellKnown64,
    WellKnown128,
    Dynamic(usize),
}

impl VoxelCubeSize {
    pub fn from_size(size: usize) -> Self {
        match size {
            8 => VoxelCubeSize::WellKnown8,
            16 => VoxelCubeSize::WellKnown16,
            32 => VoxelCubeSize::WellKnown32,
            64 => VoxelCubeSize::WellKnown64,
            128 => VoxelCubeSize::WellKnown128,
            _ => VoxelCubeSize::Dynamic(size),
        }
    }
    pub fn from_volume(volume: usize) -> Self {
        match volume {
            0..=7 => panic!("Volume too small"),
            8 => VoxelCubeSize::WellKnown8,
            16 => VoxelCubeSize::WellKnown16,
            32 => VoxelCubeSize::WellKnown32,
            64 => VoxelCubeSize::WellKnown64,
            128 => VoxelCubeSize::WellKnown128,
            _ => VoxelCubeSize::Dynamic((volume as f64).powf(1.0 / 3.0) as usize),
        }
    }
    pub fn size(&self) -> usize {
        match self {
            VoxelCubeSize::WellKnown8 => 8,
            VoxelCubeSize::WellKnown16 => 16,
            VoxelCubeSize::WellKnown32 => 32,
            VoxelCubeSize::WellKnown64 => 64,
            VoxelCubeSize::WellKnown128 => 128,
            VoxelCubeSize::Dynamic(size) => *size,
        }
    }
    pub fn slice_size(&self) -> usize {
        let size = self.size();
        size * size
    }
}

#[inline]
pub fn volume_to_len(volume: usize) -> usize {
    match volume {
        //well-known and frequent used values
        64 => 4,
        512 => 8,
        4096 => 16,
        32768 => 32,
        262144 => 64,
        2097152 => 128,
        _ => (volume as f64).powf(1.0 / 3.0) as usize,
    }
}
