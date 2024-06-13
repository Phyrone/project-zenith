use std::fmt::Debug;
use std::hash::Hash;

use bevy::prelude::{App, Changed, Component, Plugin, Query, Reflect, SystemSet, Transform, Update, Vec3};
use uuid::Uuid;

use common::{CHUNK_SIZE, CHUNK_VOLUME};
use common::storage::Storage;

pub mod mesh;

#[derive(Default, Debug)]
pub struct ClientWorldChunksPlugin;

impl Plugin for ClientWorldChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(mesh::VoxelMeshPlugin);
        app.add_systems(Update, set_static_cubes_position_system);
    }
}

/// Describes a chunk and its position into the RederedWorld
/// they are fixed to the grid
/// even chunks in the simulation world are not inteded to move
/// in the rendered world this is intended to recenter world and camera back to near 0,0,0
/// this is to prevent floating point rounding errors
/// that means x,y,z of [RenderWorldFixedVoxelCubePosition] are likely to change and systems should take it into account
/// render chunks to not care about dimensions
/// render chunks can directly translated into Transforms by multiplying the x,y,z with the chunk size
#[derive(Component, Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Reflect)]
pub struct RenderWorldFixedVoxelCubePosition {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

pub const PBR_MATERIAL_IDEN_DOMAIN: Uuid =
    Uuid::from_u128(0xbc90_9df2_66b8_224b_96ff_6b40_77c9_7fde);

#[derive(Component, Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Reflect)]
pub struct SurfaceTextureIden {
    pub domain: Uuid,
    pub iden: Uuid,
}

pub type VoxelCubeStore = Storage<CHUNK_VOLUME, Option<SurfaceTextureIden>>;

pub fn set_static_cubes_position_system(
    mut cubes: Query<(&RenderWorldFixedVoxelCubePosition, &mut Transform),
        Changed<RenderWorldFixedVoxelCubePosition>>,
) {
    cubes.par_iter_mut().for_each(|(position, mut transform)| {
        transform.translation = Vec3::new(
            CHUNK_SIZE as f32 * position.x as f32,
            CHUNK_SIZE as f32 * position.y as f32,
            CHUNK_SIZE as f32 * position.z as f32,
        );
    });
}
