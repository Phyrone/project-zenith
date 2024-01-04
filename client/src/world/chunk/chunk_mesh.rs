use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::world::chunk::chunk_data::ClientChunkEdgeData;
use crate::world::chunk::chunk_data::ClientChunkStorage;
use crate::world::chunk::RenderingWorldFixedChunk;

#[derive(Default)]
struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {}
}

fn build_meshes_system(
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
struct ChunkMeshErrand {
    task: Task<Mesh>,
}
