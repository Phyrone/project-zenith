use block_mesh::VoxelVisibility;

use game2::material::{Block, Material};

pub trait BlockClientData {
    fn get_visibility(&self) -> VoxelVisibility;
}

impl BlockClientData for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            Block::AIR => VoxelVisibility::Empty,
            _ => VoxelVisibility::Opaque,
        }
    }
}
