use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::{App, Component, Plugin, SystemSet};

pub mod chunk_data;
pub mod chunk_mesh;
pub mod grid;
pub mod voxel2;

#[derive(Default, Debug)]
pub struct ClientWorldChunksPlugin;

impl Plugin for ClientWorldChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((grid::ChunkGridPlugin, chunk_data::ChunkDataPlugin));
    }
}

/// Describes a chunk and its position into the RederedWorld
/// they are fixed to the grid
/// even chunks in the simulation world are not inteded to move
/// in the rendered world this is intended to recenter world and camera back to near 0,0,0
/// this is to prevent floating point rounding errors
/// that means x,y,z of [RenderingWorldFixedChunk] are likely to change and systems should take it into account
/// render chunks to not care about dimensions
/// render chunks can directly translated into Transforms by multiplying the x,y,z with the chunk size
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RenderingWorldFixedChunk {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, SystemSet)]
pub enum ChunkRenderStage {
    // Determine block states, position chunks etc.
    ChunkPreData,

    // Empty voxel chunks are inserted depeding on lod
    PreBuildVoxels,

    // Voxels are built from the block states
    BuildVoxels,

    // out of the voxels, meshes, textures etc. are built which then can be sent to the GPU
    ApplyVoxels,
}
