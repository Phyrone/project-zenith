use bevy::app::App;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use crate::world::chunk::chunk_data::ClientChunkData;
use crate::world::chunk::grid::ChunkGrid;
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

fn build_meshes_system(
    commands: ParallelCommands,
    chunk_grid: Res<ChunkGrid>,
    chunks : Query<(&ClientChunkData, &RenderingWorldFixedChunk)>,
) {
    let pool = AsyncComputeTaskPool::get();
    
    chunks.par_iter().for_each(|(data,pos)|{
        
        
        commands.command_scope(|commands|{
            
        });
    });
    
}



#[derive(Debug, Component)]
struct ChunkMeshBuildProcess {
    task: Task<Mesh>,
}
