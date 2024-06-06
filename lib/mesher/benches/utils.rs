/* Helper and Utilities */

use std::convert::TryInto;

use rayon::prelude::*;

use mesher::b16::VoxelCubeOcclusionMatrix16;
use mesher::b32::VoxelCubeOcclusionMatrix32;
use mesher::b64::VoxelCubeOcclusionMatrix64;

#[derive(Debug, Default, Copy, Clone)]
pub struct TestVoxel {
    pub id: u64,
}

pub type VoxelCube<const VOLUME: usize> = [TestVoxel; VOLUME];

pub type VoxelCube16 = VoxelCube<{ 16 * 16 * 16 }>;
pub type VoxelCube32 = VoxelCube<{ 32 * 32 * 32 }>;
pub type VoxelCube64 = VoxelCube<{ 64 * 64 * 64 }>;

pub fn build_bad_case_voxel_cube<const ELEMENTS: u64, const VOLUME: usize>(
) -> Box<VoxelCube<VOLUME>> {
    let mut voxels = vec![TestVoxel::default(); VOLUME];
    voxels.par_iter_mut().enumerate().for_each(|(i, voxel)| {
        let solid = i % 2 == 0;
        if solid {
            voxel.id = 1;
        } else {
            voxel.id = 0;
        }
    });
    voxels.try_into().unwrap()
}

pub fn build_filled_voxel_cube<const VOLUME: usize>() -> Box<VoxelCube<VOLUME>> {
    let mut voxels = vec![TestVoxel { id: 1 }; VOLUME];
    voxels.try_into().unwrap()
}

pub fn build_empty_voxel_cube<const VOLUME: usize>() -> Box<VoxelCube<VOLUME>> {
    let voxels = vec![TestVoxel { id: 0 }; VOLUME];
    voxels.try_into().unwrap()
}

pub fn build_occlusion_matrix64<const PAR: bool>(cube: &VoxelCube64) -> VoxelCubeOcclusionMatrix64 {
    let mut matrix = VoxelCubeOcclusionMatrix64::new();
    if (PAR) {
        matrix.par_import(|x, y, z| cube[x + y * 64 + z * 64 * 64].id != 0);
    } else {
        matrix.import(|x, y, z| cube[x + y * 64 + z * 64 * 64].id != 0);
    }

    matrix
}

pub fn build_occlusion_matrix32<const PAR: bool>(cube: &VoxelCube32) -> VoxelCubeOcclusionMatrix32 {
    let mut matrix = VoxelCubeOcclusionMatrix32::new();
    if (PAR) {
        matrix.par_import(|x, y, z| cube[x + y * 32 + z * 32 * 32].id != 0);
    } else {
        matrix.import(|x, y, z| cube[x + y * 32 + z * 32 * 32].id != 0);
    }

    matrix
}

pub fn build_occlusion_matrix16<const PAR: bool>(cube: &VoxelCube16) -> VoxelCubeOcclusionMatrix16 {
    let mut matrix = VoxelCubeOcclusionMatrix16::new();
    if (PAR) {
        matrix.par_import(|x, y, z| cube[x + y * 16 + z * 16 * 16].id != 0);
    } else {
        matrix.import(|x, y, z| cube[x + y * 16 + z * 16 * 16].id != 0);
    }
    matrix
}
