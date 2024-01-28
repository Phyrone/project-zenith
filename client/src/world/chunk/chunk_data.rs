use bevy::prelude::*;
use rayon::prelude::*;
use rclite::Arc;

use game2::{BlockFace, CHUNK_SIZE};
use game2::chunk::ChunkStorage;
use game2::material::Block;
use game2::storage::Storage;

use crate::world::chunk::{ChunkRenderStage, RenderingWorldFixedChunk};
use crate::world::chunk::grid::ChunkGrid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct ChunkDataPlugin;

impl Plugin for ChunkDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (add_edges_system, apply_deferred, update_edges_system)
                .chain()
                .in_set(ChunkRenderStage::ChunkPreData),
        );
    }
}

#[derive(
Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ChunkDataEntry {
    Block(Block),
}


#[derive(Debug, Clone, Component)]
pub struct ClientChunkData(Arc<ChunkStorage<ChunkDataEntry>>);

impl ClientChunkData {
    pub fn edit(&mut self) -> &mut ChunkStorage<ChunkDataEntry> {
        Arc::make_mut(&mut self.0)
    }
}

const EDGE_STORAGE_FACE_SIZE: usize = CHUNK_SIZE * CHUNK_SIZE;
const EDGE_STORAGE_SIZE: usize = EDGE_STORAGE_FACE_SIZE * 6;

pub type EdgeStorage = Storage<EDGE_STORAGE_SIZE, ChunkDataEntry>;

#[derive(Debug, Clone, Component)]
pub struct ClientChunkEdge(Arc<EdgeStorage>);

impl ClientChunkEdge {
    pub fn edit(&mut self) -> &mut EdgeStorage {
        Arc::make_mut(&mut self.0)
    }
}


impl ChunkDataEntry {
    pub fn empty() -> Self {
        Self::Block(Block::AIR)
    }
}

impl Default for ChunkDataEntry {
    fn default() -> Self {
        Self::Block(Block::AIR)
    }
}

#[allow(clippy::type_complexity)]
fn add_edges_system(
    mut commands: ParallelCommands,
    grid: Res<ChunkGrid>,
    added_chunks: Query<
        (Entity, &RenderingWorldFixedChunk),
        Without<ClientChunkEdge>,
    >,
    all_chunks: Query<&ClientChunkData>,
) {
    if added_chunks.is_empty() {
        return;
    }

    added_chunks.par_iter().for_each(|(entity, chunk)| {
        let mut storage = EdgeStorage::empty();


        let edge = ClientChunkEdge(Arc::new(storage));

        commands.command_scope(|mut command| {
            command.entity(entity).insert(edge);
        })
    });
}

fn update_edges_system(
    grid: Res<ChunkGrid>,
    updated_chunks: Query<
        (&RenderingWorldFixedChunk, &ClientChunkData),
        Or<(Changed<ClientChunkData>, Changed<RenderingWorldFixedChunk>)>,
    >,
    mut edges: Query<&mut ClientChunkEdge>,
) {
    if updated_chunks.is_empty() {
        return;
    }
    updated_chunks.par_iter().for_each(|(cords, data)| {
        /*
        let chunk = grid.neighbours(cords.x, cords.y, cords.z);
        for (i, neighbour) in chunk.iter().enumerate() {
            if let Some(neighbour) = neighbour {
                let mut neighbour = edges.get_mut(*neighbour).unwrap();
                let neighbour = neighbour.edit();



                //TODO update neighbour
            }
        }

         */
    });
}

fn create_edge_storage(
    neighbours: [Option<&ChunkStorage<ChunkDataEntry>>; 6],
) -> EdgeStorage {
    let mut data = [ChunkDataEntry::empty(); EDGE_STORAGE_SIZE];
    data.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let face_index = i / EDGE_STORAGE_FACE_SIZE;
        let face = BlockFace::from_index(face_index as u32);
        let i = i % EDGE_STORAGE_FACE_SIZE;

        let neighbour = neighbours[face_index];
        if let Some(neighbour) = neighbour {

        }
    });

    EdgeStorage::new(&data.to_vec())
}
