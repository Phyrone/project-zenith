use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use common::TaskContainer;
use futures_lite::FutureExt;

use crate::world::camera::lod::{LODLevel, LodLevelStage};
use crate::world::chunk::voxel::{EntityGroupedMesh, VoxelRenderData};

pub struct VoxelRenderPlugin;

impl Plugin for VoxelRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            create_errand_voxel_buffer_system
                .after(LodLevelStage)
                .before(apply_deferred)
                .in_set(VoxelRenderStage::VoxelRenderInit),
        );

        app.add_systems(
            Update,
            start_voxel_mesh_generation_system
                .in_set(VoxelRenderStage::VoxelRenderStartMeshGeneration)
                .after(VoxelRenderStage::VoxelDataFeed)
                .after(apply_deferred),
        );
    }
}

#[derive(Debug, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct VoxelRenderErrand;

type VoxelRenderProcess = TaskContainer<EntityGroupedMesh>;

fn create_errand_voxel_buffer_system(
    commands: ParallelCommands,
    errands: Query<(Entity, &LODLevel), (With<VoxelRenderErrand>, Without<VoxelRenderData>)>,
) {
    errands.par_iter().for_each(|(entity, lod)| {
        let size = match lod {
            LODLevel::FULL => 64,
            LODLevel::Far1 => 32,
            LODLevel::Far2 => 16,
            LODLevel::Fartest => 8,
        };

        let data = VoxelRenderData::new(size);
        commands.command_scope(|mut commands| {
            commands.entity(entity).insert(data);
        })
    });
}

fn apply_and_kill_existing_mesh_gen_tasks(
    mut commands: ParallelCommands,
    mut errands: Query<(Entity, &mut VoxelRenderProcess), Added<VoxelRenderErrand>>,
) {
    errands.par_iter_mut().for_each(|(entity, mut process)| {
        process.cancel();
    });
}

fn start_voxel_mesh_generation_system(
    mut commands: ParallelCommands,
    mut errands: Query<(Entity, &VoxelRenderData, &LODLevel), Added<VoxelRenderErrand>>,
) {
    let pool = AsyncComputeTaskPool::get();
    errands.par_iter_mut().for_each(|(entity, data, lod)| {
        let data = data.clone();
        let usage = if *lod == LODLevel::FULL {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };

        let task = pool.spawn(async move { data.generate_mesh(usage) });

        let process = VoxelRenderProcess::new(task);
        commands.command_scope(|mut commands| {
            commands.entity(entity).insert(process);
        });
    });
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, SystemSet)]
pub enum VoxelRenderStage {
    VoxelRenderInit,
    /// Apply necessary data to the voxel matrix neessary for generating the mesh
    VoxelDataFeed,
    /// start task for generating the mesh from the voxel matrix
    VoxelRenderStartMeshGeneration,
    /// check if the mesh generation task is done
    VoxelApplyMesh,

    VoxelRenderAppend,
}
