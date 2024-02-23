use bevy::app::App;
use bevy::prelude::{Component, Plugin};
use block_mesh::VoxelVisibility;

use game2::material::Block;

#[derive(Debug, Default)]
pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, _app: &mut App) {}
}

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

#[derive(Debug, Clone, Eq, PartialEq, Component)]
pub struct BlockMaterial {
    pub transparent: bool,
}
