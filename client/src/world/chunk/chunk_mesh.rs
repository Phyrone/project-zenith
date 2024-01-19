use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::world::chunk::chunk_data::ClientChunkEdgeData;
use crate::world::chunk::chunk_data::ClientChunkStorage;
use crate::world::chunk::RenderingWorldFixedChunk;

//works as follows:
// 1. when a chunk is changed, its mesh gets updated
//    this version does not use greedy quads, but only occlusion culling (much faster but use more vram)
// 2. at the same time, an async task is spawned that calculates the greedy quads
//    if an task is already running, it gets canceled
// 3. when the task is finished, the mesh gets updated again with the new greedy quads
//
// hopefully this will result in a smooth experience where updates are not delayed too much but greedy quads are still used for rendering

#[derive(Default)]
struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {}
}

fn build_pre_meshes_system(
    changed_chunks: Query<
        (&RenderingWorldFixedChunk, &ClientChunkStorage),
        (Changed<ClientChunkStorage>, Changed<ClientChunkEdgeData>),
    >,
) {
    let pool = AsyncComputeTaskPool::get();

    changed_chunks.par_iter().for_each(|(pos, data)| {
        let task = pool.spawn(async move {});
    });
}

#[derive(Debug, Component)]
struct ChunkMeshBuildProcess {
    task: Task<Mesh>,
}
