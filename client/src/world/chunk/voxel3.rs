use bevy::prelude::Query;
use rayon::prelude::*;

use game2::CHUNK_SIZE;

use crate::world::chunk::chunk_data::{ChunkDataEntry, ChunkDataStorage};
use crate::world::material::Material;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Voxel<'render> {
    pub id: u32,
    material: &'render Material,
}

impl ChunkDataEntry {
    fn create_voxel<'render>(
        &'render self,
        resolution: usize,
        index: usize,
        materials: &'render Query<(&'render Material)>,
    ) -> Option<Voxel<'render>> {
        match self {
            ChunkDataEntry::Empty => None,
            ChunkDataEntry::Block(entity, _) => {
                let material: &'render Material = materials.get(*entity).ok()?;
                Some(Voxel {
                    id: entity.index(),
                    material,
                })
            }
        }
    }
}

pub fn create_voxel_chunk<'render>(
    data: &ChunkDataStorage,
    neighbors: &[Option<&ChunkDataStorage>; 6],
    materials: &'render Query<(&'render Material)>,
    resolution: usize,
) -> Vec<Option<Voxel<'render>>> {
    let voxel_chunk_size = resolution * CHUNK_SIZE;
    let voxel_chunk_volume = voxel_chunk_size * voxel_chunk_size * voxel_chunk_size;
    let mut voxels = vec![None; voxel_chunk_volume];
    voxels.par_iter_mut().enumerate().for_each(|(i, mut voxel)| {
        let (x, y, z) = (
            i % voxel_chunk_size,
            (i / voxel_chunk_size) % voxel_chunk_size,
            i / (voxel_chunk_size * voxel_chunk_size),
        );
        let mode = (x == 0) as i8
            + (y == 0) as i8
            + (z == 0) as i8
            + (x == voxel_chunk_size - 1) as i8
            + (y == voxel_chunk_size - 1) as i8
            + (z == voxel_chunk_size - 1) as i8;
        //skip edges
        match mode {
            //inner
            0 => get_voxel_inner(x, y, z, resolution, voxel_chunk_size, data, materials, voxel),
            //face
            1 => {}
            //skip corners
            _ => return
        }
    });

    voxels
}

fn get_voxel_inner<'render>(
    x: usize,
    y: usize,
    z: usize,
    resolution: usize,
    voxel_chunk_size: usize,
    chunk: &ChunkDataStorage,
    materials: &'render Query<(&'render Material)>,
    into: &mut Option<Voxel<'render>>,
) {
    let inner_index = (x - 1) + (y - 1) * (voxel_chunk_size - 2) + (z - 1) * (voxel_chunk_size - 2) * (voxel_chunk_size - 2);
    let inner_entry = chunk.get(inner_index);

    let (inner_x, inner_y, inner_z) = (x - 1, y - 1, z - 1);
    let (inner_entry_x, inner_entry_y, inner_entry_z) = (inner_x / resolution, inner_y / resolution, inner_z / resolution);
    let (sub_x, sub_y, sub_z) = (inner_x % resolution, inner_y % resolution, inner_z % resolution);
    let sub_index = sub_x + sub_y * resolution + sub_z * resolution * resolution;
    *into = inner_entry.create_voxel(resolution, sub_index, materials);
}