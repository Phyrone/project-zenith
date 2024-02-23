use std::sync::Arc;

use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, Task};
use futures_lite::future;

use game2::WithFixedSizeExt;

use crate::world::chunk::{ChunkRenderStage, RenderingWorldFixedChunk};
use crate::world::chunk::chunk_data::{ChunkNeighborDataValues, ClientChunkData};
use crate::world::chunk::grid::ChunkGrid;
use crate::world::chunk::voxel3::{create_voxel_chunk, Voxel, voxels_geedy_mesh, voxels_mesh};
use crate::world::material::BlockMaterial;

type ChangedStatiChunksQuery<'world, 'state, T> = Query<
    'world,
    'state,
    T,
    (
        Changed<ClientChunkData>,
        Changed<ChunkNeighborDataValues>,
        With<RenderingWorldFixedChunk>,
        With<ClientChunkData>,
        With<ChunkNeighborDataValues>,
    ),
>;


//TODO cancel tasks when chunk is removed
//TODO cancel tasks when chunk is updated before the task is finished
//TODO lod support
#[derive(Default)]
struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update, (
                create_build_mesh_tasks,
                apply_calculated_meshes_system
            ).in_set(ChunkRenderStage::BuildVoxels),
        );
    }
}

fn create_build_mesh_tasks(
    commands: ParallelCommands,
    chunk_grid: Res<ChunkGrid>,
    chunks: ChangedStatiChunksQuery<(
        Entity,
        &ClientChunkData,
        &RenderingWorldFixedChunk,
    )>,
    materials: Query<&BlockMaterial>,
) {
    let pool = AsyncComputeTaskPool::get();

    chunks.par_iter().for_each(
        |(entity, data, pos)| {
            let neighbors = [
                chunk_grid.get(pos.x - 1, pos.y, pos.z),
                chunk_grid.get(pos.x + 1, pos.y, pos.z),
                chunk_grid.get(pos.x, pos.y - 1, pos.z),
                chunk_grid.get(pos.x, pos.y + 1, pos.z),
                chunk_grid.get(pos.x, pos.y, pos.z - 1),
                chunk_grid.get(pos.x, pos.y, pos.z + 1),
            ]
                .iter()
                .map(|entity| match entity {
                    None => None,
                    Some(entity) => chunks.get(*entity).ok(),
                })
                .map(|chunk| chunk.map(|(_, data, _)| data.storage()))
                .collect::<Vec<_>>()
                .into_fixed_size::<6>();
            //TODO change resolution depending on LOD
            let resolution = 1;
            let voxels = create_voxel_chunk(data.storage(), &neighbors, &materials, resolution);
            //TODO use Arc<[Voxel]> instead of Vec<Voxel> when we maybe run
            let voxels: Arc<[Voxel]> = voxels.into();
            let voxels2 = voxels.clone();
            let mesh_task = pool.spawn(async move {
                let voxels = voxels.clone();
                voxels_mesh(&voxels, resolution)
            });

            let greedy_mesh_task =
                pool.spawn(async move { voxels_geedy_mesh(&voxels2, resolution) });

            let mesh_task = MeshTask::new(mesh_task);
            let greedy_mesh_task = GreedyMeshTask::new(greedy_mesh_task);

            commands.command_scope(|mut commands| {
                commands
                    .entity(entity)
                    .insert(greedy_mesh_task)
                    .insert(mesh_task);
            });
        },
    );
}

fn apply_calculated_meshes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tasks: Query<(Entity, Option<&mut MeshTask>, &mut GreedyMeshTask)>,
) {
    tasks.iter_mut().for_each(|(entity, mut mesh_task, mut greedy_mesh_task)| {
        let got_mesh = if let Some(mesh_task) = &mut mesh_task {
            let mesh_task = &mut mesh_task.task;
            block_on(future::poll_once(&mut *mesh_task))
        } else { None };

        let mut greedy_mesh_task = &mut greedy_mesh_task.task;
        let got_greedy_mesh = block_on(future::poll_once(&mut *greedy_mesh_task));

        if let Some(mesh) = got_greedy_mesh {
            commands.entity(entity).insert(meshes.add(mesh))
                .remove::<GreedyMeshTask>()
                .remove::<MeshTask>();
        } else if let Some(mesh) = got_mesh {
            commands.entity(entity).insert(meshes.add(mesh))
                .remove::<MeshTask>();
        };
    });
}


#[derive(Debug, Component)]
struct GreedyMeshTask {
    task: Task<Mesh>,
}

impl GreedyMeshTask {
    fn new(task: Task<Mesh>) -> Self {
        Self { task }
    }
}

#[derive(Debug, Component)]
struct MeshTask {
    task: Task<Mesh>,
}

impl MeshTask {
    fn new(task: Task<Mesh>) -> Self {
        Self { task }
    }
}
