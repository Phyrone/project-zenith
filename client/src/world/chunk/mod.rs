use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::{App, Component, Plugin, SystemSet};

pub mod voxel;
mod voxel_render;

#[derive(Default, Debug)]
pub struct ClientWorldChunksPlugin;

impl Plugin for ClientWorldChunksPlugin {
    fn build(&self, app: &mut App) {}
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
