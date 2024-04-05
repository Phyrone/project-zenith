use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy::utils::AHasher;
use rayon::prelude::*;
use rclite::Arc;
use unstructured::Document;

use game2::CHUNK_VOLUME;
use game2::storage::Storage;

use crate::world::chunk::{ChunkRenderStage, VoxelWorldFixedChunkPosition};
use crate::world::chunk::grid::ChunkGrid;
use crate::world::material::MaterialDescription;

use crate::world::chunk::{ChunkRenderStage, VoxelWorldFixedChunkPosition};

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
    changes: Query<(&VoxelWorldFixedChunkPosition, &ClientChunkData), Changed<ClientChunkData>>,
    _all_chunks: Query<&ClientChunkData, With<VoxelWorldFixedChunkPosition>>,
    mut neighbors: Query<&mut ChunkNeighborDataValues>,
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
                //neighbor_data.update(Some(data.storage()));
            }
        }
    }
}

/* Components */
pub type ChunkDataStorage = Storage<CHUNK_VOLUME, ChunkDataEntry>;

#[derive(Debug, Clone, Component)]
pub struct ClientChunkData(Arc<ChunkDataStorage>);

impl ClientChunkData {
    pub fn new(data: ChunkDataStorage) -> Self {
        Self(Arc::new(data))
    }
    pub fn empty() -> Self {
        Self(Arc::new(ChunkDataStorage::empty()))
    }
}

impl Deref for ClientChunkData {
    type Target = ChunkDataStorage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClientChunkData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
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
    Block(MaterialDescription),
}

impl ChunkDataEntry {
    pub fn empty() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Component)]
pub struct ChunkNeighborDataValues {
    neighbours_check: u64,
}

impl Default for ChunkNeighborDataValues {
    fn default() -> Self {
        let mut hasher = AHasher::default();
        [Option::<&ChunkDataStorage>::None; 6].hash(&mut hasher);
        Self {
            neighbours_check: hasher.finish(),
        }
    }
}

impl ChunkNeighborDataValues {
    pub fn new(neighbors: [Option<&ChunkDataStorage>; 6]) -> Self {
        let mut hasher = AHasher::default();
        neighbors.hash(&mut hasher);
        Self {
            neighbours_check: hasher.finish(),
        }
    }

    pub fn update(&mut self, neighbors: [Option<&ChunkDataStorage>; 6]) -> bool {
        let mut hasher = AHasher::default();
        neighbors.hash(&mut hasher);
        let new_check = hasher.finish();
        if new_check != self.neighbours_check {
            self.neighbours_check = new_check;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use std::hash::Hash;
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
        let neighbors = ChunkNeighborDataValues::default();
        println!("{:?}", neighbors);
    }
}
