use bevy::app::{App, Plugin};
use bevy::hierarchy::Parent;
use bevy::prelude::{
    apply_deferred, Changed, Commands, Component, Entity, IntoSystemConfigs, Query,
    RemovedComponents, SystemSet, With, Without,
};
use bimap::BiHashMap;

use crate::world::chunk::Chunk;
use crate::world::Dimension;
use crate::world::WorldSchedules::WorldTick;

#[derive(Debug, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, SystemSet)]
struct ChunkGridSystems;

#[derive(Debug, Default)]
pub struct ChunkGridPlugin;

impl Plugin for ChunkGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            WorldTick,
            (
                chunk_grid_attach_system,
                apply_deferred,
                (chunk_grid_construct_system, chunk_grid_destruct_system),
            )
                .chain()
                .in_set(ChunkGridSystems),
        );
    }
}

fn chunk_grid_attach_system(
    mut command: Commands,
    dimensions: Query<Entity, (With<Dimension>, Without<ChunkGrid>)>,
) {
    for dimension in dimensions.iter() {
        command.entity(dimension).insert(ChunkGrid::default());
    }
}

fn chunk_grid_construct_system(
    mut dimensions: Query<&mut ChunkGrid, With<Dimension>>,
    updated_chunks: Query<(Entity, &Parent, &Chunk), Changed<Chunk>>,
) {
    if updated_chunks.is_empty() {
        return;
    }
    for (entity_id, dim_id, coords) in updated_chunks.iter() {
        if let Ok(mut grid) = dimensions.get_mut(dim_id.get()) {
            grid.0.insert((coords.x, coords.y, coords.z), entity_id);
        }
    }
}

fn chunk_grid_destruct_system(
    mut dimensions: Query<&mut ChunkGrid, With<Dimension>>,
    mut removed_chunks: RemovedComponents<Chunk>,
) {
    if removed_chunks.is_empty() {
        return;
    }

    let mut removed = vec![];
    for entity in removed_chunks.read() {
        removed.push(entity);
    }
    removed.shrink_to_fit();

    dimensions.par_iter_mut().for_each(|grid| {
        let grid = grid.into_inner();
        for entity in &removed {
            grid.0.remove_by_right(entity);
        }
    });
}

//TODO respect dimensions
#[derive(Debug, Component, Default)]
pub struct ChunkGrid(Box<BiHashMap<(i64, i64, i64), Entity>>);
