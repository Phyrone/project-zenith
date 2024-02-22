use std::hash::{Hash, Hasher};

use bevy::prelude::*;
use bevy::utils::AHasher;
use rayon::prelude::*;
use rclite::Arc;
use unstructured::Document;

use game2::CHUNK_VOLUME;
use game2::storage::Storage;

use crate::world::chunk::{ChunkRenderStage, RenderingWorldFixedChunk};
use crate::world::chunk::grid::ChunkGrid;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct ChunkDataPlugin;

impl Plugin for ChunkDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_neighbor_checksums)
                .in_set(ChunkRenderStage::ChunkPreData),
        );
    }
}

/* Systems */
fn update_neighbor_checksums(
    changes: Query<(&RenderingWorldFixedChunk, &ClientChunkData), Changed<ClientChunkData>>,
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
        for (i, neighbor) in to_update.iter().enumerate() {
            if let Some(neighbor) = neighbor {
                if let Ok(mut neighbor_data) = neighbors.get_mut(*neighbor) {
                    neighbor_data.update(i, Some(data.storage()));
                }
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize, )]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[derive(Component)]
pub struct ChunkNeighborDataValues {
    neighbor_summaries: [u64; 6],
}

impl ChunkNeighborDataValues {
    pub fn new() -> Self {
        Self {
            neighbor_summaries: [Self::checksum(None); 6]
        }
    }

    fn checksum(storage: Option<&ChunkDataStorage>) -> u64 {
        let mut hasher = AHasher::default();
        storage.hash(&mut hasher);
        hasher.finish()
    }
    pub fn needs_update(
        &self,
        index: usize,
        against: Option<&ChunkDataStorage>,
    ) -> bool {
        let expected = Self::checksum(against);
        self.neighbor_summaries[index] != expected
    }

    pub fn update(&mut self, index: usize, storage: Option<&ChunkDataStorage>) {
        self.neighbor_summaries[index] = Self::checksum(storage);
    }
}


#[cfg(test)]
mod test {
    use std::mem::size_of;

    use game2::humanize::humanize_memory;

    use crate::world::chunk::chunk_data::ChunkDataEntry;

    #[test]
    fn chunk_data_entry_size() {
        let size = size_of::<ChunkDataEntry>();
        println!("Size of ChunkDataEntry: {}", humanize_memory(size));
    }
}