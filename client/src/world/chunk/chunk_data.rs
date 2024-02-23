use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::AHasher;
use rayon::prelude::*;
use rclite::Arc;
use unstructured::Document;

use game2::storage::Storage;
use game2::CHUNK_VOLUME;

use crate::world::chunk::grid::ChunkGrid;
use crate::world::chunk::{ChunkRenderStage, RenderingWorldFixedChunk};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct ChunkDataPlugin;

impl Plugin for ChunkDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_neighbor_checksums).in_set(ChunkRenderStage::ChunkPreData),
        );
    }
}

/* Systems */
fn update_neighbor_checksums(
    changes: Query<(&RenderingWorldFixedChunk, &ClientChunkData), Changed<ClientChunkData>>,
    all_chunks: Query<(&ClientChunkData), With<RenderingWorldFixedChunk>>,
    mut neighbors: Query<(&mut ChunkNeighborDataValues)>,
    grid: Res<ChunkGrid>,
) {
    for (pos, data) in changes.iter() {
        let (x, y, z) = (pos.x, pos.y, pos.z);
        let to_update = [
            grid.get(x - 1, y, z),
            grid.get(x + 1, y, z),
            grid.get(x, y - 1, z),
            grid.get(x, y + 1, z),
            grid.get(x, y, z - 1),
            grid.get(x, y, z + 1),
        ];
        for neighbor in to_update.iter().flatten() {
            if let Ok(mut neighbor_data) = neighbors.get_mut(*neighbor) {
                //let neighbor_neighbour_data = all_chunks.get(*neighbor).unwrap();
                //TODO only update if relevant data changed
                neighbor_data.update(Some(data.storage()));
            }
        }
    }
}

/* Components */
pub type ChunkDataStorage = Storage<CHUNK_VOLUME, ChunkDataEntry>;

#[derive(Debug, Clone, Component)]
pub struct ClientChunkData(Arc<ChunkDataStorage>);

impl ClientChunkData {
    pub fn edit(&mut self) -> &mut ChunkDataStorage {
        Arc::make_mut(&mut self.0)
    }

    pub fn storage(&self) -> &ChunkDataStorage {
        &self.0
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ChunkDataEntry {
    #[default]
    Empty,
    //the referenced material + additional data
    Block(Entity, Option<Box<Document>>),
}

impl ChunkDataEntry {
    pub fn empty() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Component)]
pub struct ChunkNeighborDataValues {
    neighbor_summary: u64,
}

impl ChunkNeighborDataValues {
    pub fn new() -> Self {
        Self {
            neighbor_summary: 0,
        }
    }

    pub fn update(&mut self, storage: Option<&ChunkDataStorage>) {
        let mut hasher = AHasher::default();
        hasher.write_u64(self.neighbor_summary);
        storage.hash(&mut hasher);
        self.neighbor_summary = hasher.finish();
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use game2::humanize::humanize_memory;

    use crate::world::chunk::chunk_data::{ChunkDataEntry, ChunkNeighborDataValues};

    #[test]
    fn chunk_data_entry_size() {
        let size = size_of::<ChunkDataEntry>();
        println!("Size of ChunkDataEntry: {}", humanize_memory(size));
    }

    #[test]
    fn test_chunk_data_entry() {
        let neighbors = ChunkNeighborDataValues::new();
        println!("{:?}", neighbors);
    }
}
