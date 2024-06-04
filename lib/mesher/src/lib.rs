use std::hash::Hash;
use std::ops::{BitAnd, BitAndAssign, Not};

use bitvec::store::BitStore;
use hashbrown::HashMap;
use num_traits::{Num, NumAssign, NumOps, PrimInt, Unsigned};
use rayon::prelude::*;

pub use t1::greedy_mesh_binary_plane;

#[cfg(not(test))]
use crate::cube_slice::*;
//public for benches
#[cfg(test)]
pub use crate::cube_slice::*;
use crate::occlusion_culling::VoxelCubeOcclusionMatrix64;

mod cube_slice;
#[cfg(feature = "bevy")]
pub mod meshing;
pub mod occlusion_culling;
mod t1;

type GetSurfaceFn<'a, S> = &'a dyn Fn(usize, &FaceDirection) -> Option<S>;
type IsSolidFn = dyn Fn(usize) -> bool;

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

    //I ->Right, J -> Up, K -> Close
    //Z: I->X, J->Y, K->Z
    //Y: I->X, J->Z, K->Y
    //X: I->Z, J->Y, K->X
    pub fn axis_rel_to_absolute(&self, i: usize, j: usize, k: usize) -> (usize, usize, usize) {
        match self {
            FaceDirection::ZPos | FaceDirection::ZNeg => (i, j, k),
            FaceDirection::YPos | FaceDirection::YNeg => (i, k, j),
            FaceDirection::XPos | FaceDirection::XNeg => (k, j, i),
        }
    }

    pub fn absolute_to_axis_rel(&self, x: usize, y: usize, z: usize) -> (usize, usize, usize) {
        match self {
            FaceDirection::ZPos | FaceDirection::ZNeg => (x, y, z),
            FaceDirection::YPos | FaceDirection::YNeg => (x, z, y),
            FaceDirection::XPos | FaceDirection::XNeg => (z, y, x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GreedyQuad {
    pub direction: FaceDirection,
    pub i: usize,
    pub j: usize,
    pub k: usize,
    pub w: usize,
    pub h: usize,
}

impl GreedyQuad {
    
    pub fn corner_positions(){
        
    }
    
    
}

pub fn build_mesh64<'a, S>(
    matrix: &VoxelCubeOcclusionMatrix64,
    get_surface: impl Fn(usize, usize, usize, &FaceDirection) -> Option<S>,
) -> HashMap<S, Vec<GreedyQuad>>
where
    S: Hash + Eq + PartialEq + 'a,
{
    //Oclussion culling and grouping by type
    //direction -> surface -> slice
    let mut slice_by_axis_by_group: [HashMap<S, HashMap<usize, [u64; u64::BITS as usize]>>; 6] =
        Default::default();
    matrix.find_surfaces(|x, y, z, face| {
        let surface = get_surface(x, y, z, &face);
        if let Some(surface) = surface {
            let (i, j, k) = face.absolute_to_axis_rel(x, y, z);
            let slices = slice_by_axis_by_group[face.to_index()]
                .entry(surface)
                .or_default();
            let slice = slices.entry(k).or_insert_with(|| [0; u64::BITS as usize]);
            slice[i] |= 1 << j;
        }
    });

    //Greedy meshing
    let mut grouped_quads = HashMap::new();
    for (axis, entries) in slice_by_axis_by_group.into_iter().enumerate() {
        let axis = FaceDirection::from_index(axis);
        for (surface, slices) in entries {
            let mut quads = Vec::new();
            for (k, slice) in slices.into_iter() {
                greedy_mesh_slice_64_no_alloc(slice, |i, j, w, h| {
                    quads.push(GreedyQuad {
                        direction: axis,
                        i,
                        j,
                        k,
                        w,
                        h,
                    });
                });
            }
            grouped_quads
                .entry(surface)
                .or_insert_with(Vec::new)
                .extend(quads);
        }
    }
    grouped_quads
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

#[cfg(test)]
pub mod test {
    #[allow(unused)]
    use rayon::iter::*;
    use rstest::rstest;
    use std::time::Duration;

    use crate::occlusion_culling::VoxelCubeOcclusionMatrix64;

    #[rstest]
    #[timeout(Duration::from_secs(3))]
    fn mesh_worst_case() {
        let cube = build_worst_case_voxel_cube();
        let matrix = build_occlusion_matrix::<true>(&cube);
        let _ = super::build_mesh64(&matrix, |x, y, z, face| {
            if cube[x + y * 64 + z * 64 * 64].solid {
                Some(1)
            } else {
                None
            }
        });
    }

    /* Helper and Utilities */

    #[derive(Debug, Copy, Clone)]
    pub struct TestVoxel {
        pub solid: bool,
    }

    pub type VoxelCube64 = [TestVoxel; 64 * 64 * 64];

    pub fn build_worst_case_voxel_cube() -> Box<VoxelCube64> {
        let mut voxels = vec![TestVoxel { solid: false }; 64 * 64 * 64];
        voxels.par_iter_mut().enumerate().for_each(|(i, voxel)| {
            let solid = i % 2 == 0;
            voxel.solid = solid;
        });
        voxels.try_into().unwrap()
    }

    pub fn build_filled_voxel_cube() -> Box<VoxelCube64> {
        let mut voxels = vec![TestVoxel { solid: false }; 64 * 64 * 64];
        voxels.par_iter_mut().for_each(|voxel| {
            voxel.solid = true;
        });
        voxels.try_into().unwrap()
    }

    pub fn build_occlusion_matrix<const PAR: bool>(
        cube: &VoxelCube64,
    ) -> VoxelCubeOcclusionMatrix64 {
        let mut matrix = VoxelCubeOcclusionMatrix64::new();
        if (PAR) {
            matrix.par_import(|x, y, z| cube[x + y * 64 + z * 64 * 64].solid);
        } else {
            matrix.import(|x, y, z| cube[x + y * 64 + z * 64 * 64].solid);
        }

        matrix
    }

    const DATA_8: [u8; 8] = [
        0b1100_0011,
        0b1100_1111,
        0b0011_1111,
        0b0001_0000,
        0b1000_0000,
        0b1000_1000,
        0b1000_0100,
        0b1000_0010,
    ];
    const DATA_16: [u16; 16] = [
        0b1100_0011_1100_1111,
        0b0011_1111_0001_0000,
        0b1000_0000_1000_1000,
        0b1000_0100_1000_0010,
        0b0000_0000_0000_0000,
        0b0000_0000_0000_0000,
        0b0000_0000_0000_0000,
        0b1000_1100_1000_0010,
        0b1000_0100_1000_0010,
        0b1111_1111_1111_1111,
        0b1111_1111_1111_1111,
        0b1111_1111_1111_1111,
        0b0011_1111_0001_0000,
        0b1000_0000_1000_1000,
        0b1000_0100_1000_0010,
        0b1000_0100_1000_0010,
    ];

    const DATA_32: [u32; 32] = [
        0b1100_0011_1100_1111_0011_1111_0001_0000,
        0b1000_0000_1000_1000_1000_0100_1000_0010,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b1000_1100_1000_0010_1000_0100_1000_0010,
        0b1111_1111_1111_1111_1111_1111_1111_1111,
        0b1111_1111_1111_1111_1111_1111_1111_1111,
        0b0011_1111_0001_0000_1000_0000_1000_1000,
        0b1000_0100_1000_0010_1000_0100_1000_0010,
        0b1000_0100_1000_0010_1000_0100_1000_0010,
        0b1100_0011_1100_1111_0011_1111_0001_0000,
        0b1000_0000_1000_1000_1000_0100_1000_0010,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b1000_1100_1000_0010_1000_0100_1000_0010,
        0b1111_1111_1111_1111_1111_1111_1111_1111,
        0b1111_1111_1111_1111_1111_1111_1111_1111,
        0b0011_1111_0001_0000_1000_0000_1000_1000,
        0b1000_0100_1000_0010_1000_0100_1000_0010,
        0b1000_0100_1000_0010_1000_0100_1000_0010,
        0b1100_0011_1100_1111_0011_1111_0001_0000,
        0b1000_0000_1000_1000_1000_0100_1000_0010,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b0000_0000_0000_0000_0000_0000_0000_0000,
        0b1000_1100_1000_0010_1000_0100_1000_0010,
        0b1111_1111_1111_1111_1111_1111_1111_1111,
        0b1111_1111_1111_1111_1111_1111_1111_1111,
        0b0011_1111_0001_0000_1000_0000_1000_1000,
        0b1000_0100_1000_0010_1000_0100_1000_0010,
        0b1000_0100_1000_0010_1000_0100_1000_0010,
        0b1100_0011_1100_1111_0011_1111_0001_0000,
        0b1000_0000_1000_1000_1000_0100_1000_0010,
    ];

    //64 lines of randome 64 bit integers
    const DATA_64: [u64; 64] = [
        0b1100_0011_1100_1111_0011_1111_0001_0000_1100_0011_1100_1111_0011_1111_0001_0000,
        0b0110_111111111001110010111100000111111000000001100011111111000000,
        0b0111_100001110000111000110100101011101011110100001101100001111000,
        0b0000_110111100001100010000000010011011011111100110111110100111101,
        0b0001100010110110011101110111111111100111110111111111001101010100,
        0b1110011101110000111110011010101000110110011010101101101011011001,
        0b1100001010010101101000010000111111100000001010001101010111101001,
        0b0111001011010010011010001011011110001001010011000110100010011100,
        0b0100101010011011001100010011010001110000001000110100101000110010,
        0b1011000111000100101100100110001000001000110001110000011010010111,
        0b0100100101111000010110000011111100000100011011110101101010011110,
        0b0010000101001001100011011111011010010101001000100000001111010001,
        0b1110110001000110110010011011100001111100111110010101100101010100,
        0b1100010110011010110001111100100100000101101000110101011111000000,
        0b1101011111110010110010100000001011100000111011000100100111010110,
        0b0010011000101101111001110010011011000000001110100011111101011011,
        0b1001101111011011001010010101000100111101001101010010001011100000,
        0b0111010010111100010111101100111100011000001111111110111011101111,
        0b0011111000001011111101100111110111001011010110100100110011110111,
        0b1001000110100111011111000100110100000111111001110111011001000100,
        0b0101101111101100011101011110101110011111001000010001010111110101,
        0b1110100000100110010111010110101010001000011011000110110100011011,
        0b1011011010011000100010001011110110100111111001011100001110000000,
        0b0001100101101001010111000101011001110001010101100011111110101100,
        0b1110110100000101001010001111100000100001011100000010110110000100,
        0b0101111001100011010111011001001010110001000100001100101010000100,
        0b1100001111010000001010101001011010001101110110011011001011111111,
        0b1011110110001100010110110001101011100111001110100110000110110001,
        0b1101110011011101100100001101100010100000100011111000111011010100,
        0b1011101111000100010100000010100010111100101011000000000101110010,
        0b1100011001010100001000011000000111101001101110000010000110110000,
        0b0011010111110101101011001001110001011110010010111111111000000000,
        0b0100111111011110101011001100001110101110100110001110010111011100,
        0b1001001101101110101111101101111000101011111111011111001100100110,
        0b1001000110101010110000001001101000010100110011011101110100111100,
        0b0011011000100011001011110111000010010110110110001011100100110000,
        0b1111000100011101110101011011100000111110000011001101011000010001,
        0b1100001100100100110101011011001001111000011001110110100011110001,
        0b1111010001010001101111111010010100101000110000001101000000110001,
        0b1011110000010010100010001110111101100001110001100110100000100100,
        0b1001100011101101101011011100111001111110101010111001101110001100,
        0b0010101000000000011110111110110001111001111101101100110010010001,
        0b1000100000110011001111011111011010000111011001100000110011110001,
        0b0001011111111100110011000110000100010100011101100100001100010111,
        0b1101011010001100110011101010010101001110111110010011110011110100,
        0b1101001100010110010000110001001010001110101111111011011110101101,
        0b0000010000001001101010010011110011101001101101110010001011101000,
        0b0110111001100110111010111010000000110100111001110010011011011001,
        0b0111101101100110000010110000110000001101111011001011010011011100,
        0b0011000010110010110001100101011100001100101111000000111001100000,
        0b1110010011110010110100000100110100110011000100010101010100011111,
        0b0001110001101101011010101110000011010111000000001110100101100011,
        0b0110001000111001000101110010110000100111110101001101000101010001,
        0b1101001000101111000011100101111010010101000111011111111100000111,
        0b1001011101001011000001000010101111101000011111111110101001010000,
        0b0010101011110001111011101001001100111101011111101011101110001100,
        0b1001110101111010110000101111000110000011000011101100010000110011,
        0b1010001101101100010100100010101101000011111110001010101101000011,
        0b0001111011101111111011110110101010111011010111111010000011001111,
        0b0011011000010011101001111110000010100101011000101110110111010000,
        0b1010011100001111001100010011110101111010010010100001100010111010,
        0b1111010111100100100111100010001101011111010000000001010001100111,
        0b1110001101110000011001100010011010000011110111001111001110000101,
        0b1101100100101001011101000000010001010101001110001001100011010001,
    ];
}
