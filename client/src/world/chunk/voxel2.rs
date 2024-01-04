use block_mesh::VoxelVisibility;

use game2::chunk::ChunkStorage;
use game2::material::Material;

use crate::world::material::MaterialClientData;

struct Voxel {
    material: Material,
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        self.material.get_visibility()
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = Material;

    fn merge_value(&self) -> Self::MergeValue {
        self.material
    }
}

struct VoxelChunk {
    storage: ChunkStorage<Voxel>,
}
