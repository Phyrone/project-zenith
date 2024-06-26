use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

use bevy::prelude::{
    App, Changed, Component, IntoSystemSetConfigs, Plugin, Query, Reflect, SystemSet, Transform,
    Update, Vec3,
};
use uuid::Uuid;

use common::storage::Storage;
use common::{CHUNK_SIZE, CHUNK_VOLUME};

pub mod mesh;
pub mod pbr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Reflect, SystemSet)]
pub enum ChunkRenderStage {
    ComputeOccupied,
    ComputeMesh,
    ApplyMaterial,
}

#[derive(Default, Debug)]
pub struct CubeRenderWorldPlugin;

impl Plugin for CubeRenderWorldPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                ChunkRenderStage::ComputeOccupied.before(ChunkRenderStage::ComputeMesh),
                ChunkRenderStage::ComputeMesh.before(ChunkRenderStage::ApplyMaterial),
            ),
        );
        app.add_plugins((mesh::VoxelMeshPlugin, pbr::ChunkPbrPlugin));
        app.add_systems(Update, set_static_cubes_position_system);
    }
}

/// Describes a cubes and its position into the RederedWorld
/// they are fixed to the grid
/// even chunks in the simulation world are not inteded to move
/// in the rendered world this is intended to recenter world and camera back to near 0,0,0
/// this is to prevent floating point rounding errors
/// that means x,y,z of [RenderWorldFixedVoxelCubePosition] are likely to change and systems should take it into account
/// render chunks to not care about dimensions
/// render chunks can directly translated into Transforms by multiplying the x,y,z with the cubes size
#[derive(Component, Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Reflect)]
pub struct RenderWorldFixedVoxelCubePosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub struct RenderBlock {}

pub type VoxelCubeStore = Storage<CHUNK_VOLUME, Option<SurfaceMaterial>>;

pub fn set_static_cubes_position_system(
    mut cubes: Query<
        (&RenderWorldFixedVoxelCubePosition, &mut Transform),
        Changed<RenderWorldFixedVoxelCubePosition>,
    >,
) {
    cubes.par_iter_mut().for_each(|(position, mut transform)| {
        transform.translation = Vec3::new(
            CHUNK_SIZE as f32 * position.x as f32,
            CHUNK_SIZE as f32 * position.y as f32,
            CHUNK_SIZE as f32 * position.z as f32,
        );
    });
}
