use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::{App, Component, Plugin, SystemSet};

use game2::Direction;

pub mod chunk_data;
pub mod chunk_render_mesh;
pub mod grid;
//pub mod voxel2;
mod chunk_apply_block_materials;
pub mod voxel;

#[derive(Default, Debug)]
pub struct ClientWorldChunksPlugin;

impl Plugin for ClientWorldChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((grid::ChunkGridPlugin, chunk_data::ChunkDataPlugin));
        app.add_plugins(chunk_render_mesh::ChunkRenderMeshPlugin);
    }
}

/// Describes a chunk and its position into the RederedWorld
/// they are fixed to the grid
/// even chunks in the simulation world are not inteded to move
/// in the rendered world this is intended to recenter world and camera back to near 0,0,0
/// this is to prevent floating point rounding errors
/// that means x,y,z of [VoxelWorldFixedChunkPosition] are likely to change and systems should take it into account
/// render chunks to not care about dimensions
/// render chunks can directly translated into Transforms by multiplying the x,y,z with the chunk size
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VoxelWorldFixedChunkPosition {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, SystemSet)]
pub enum ChunkRenderStage {
    // Determine block states, position chunks etc.
    ChunkPreData,

    // Voxels are built from the block states
    ConstructMesh,

    //the constructed voxels are now used to build the mesh
    ApplyMaterials,
}

#[repr(u8)]
enum TextureSplit {
    Mono = 1,
    TSB = 2,
    All = 3,
}

pub struct VoxelTextureIden2(u32);
impl VoxelTextureIden2 {
    const SPLIT_OFFSET: u32 = 30;
    pub fn new(texture: u32, split: TextureSplit) -> Self {
        Self(texture | ((split as u32) << Self::SPLIT_OFFSET))
    }

    pub fn texture(&self) -> u32 {
        self.0 & !(0b11 << Self::SPLIT_OFFSET)
    }
    pub fn split(&self) -> TextureSplit {
        match (self.0 >> Self::SPLIT_OFFSET) & 0b11 {
            1 => TextureSplit::Mono,
            2 => TextureSplit::TSB,
            3 => TextureSplit::All,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextureIden(u32, Direction);

impl TextureIden {
    pub fn new(texture: u32, direction: Direction) -> Self {
        Self(texture, direction)
    }

    pub fn texture(&self) -> u32 {
        self.0
    }

    pub fn direction(&self) -> Direction {
        self.1
    }
}
