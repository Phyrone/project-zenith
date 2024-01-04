use bevy::prelude::{
    apply_deferred, Added, App, Changed, Component, Entity, IntoSystemConfigs, ParallelCommands,
    Plugin, Query, Res, Update, Without,
};

use game2::chunk::ChunkStorage;
use game2::material::{Block, Material};
use game2::CHUNK_SIZE;

use crate::world::chunk::grid::ChunkGrid;
use crate::world::chunk::voxel::{Voxel, VoxelBlock, VOXEL_BLOCK_VOLUME};
use crate::world::chunk::{ChunkRenderStage, RenderingWorldFixedChunk};

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

pub type ClientChunkStorage = ChunkStorage<ChunkDataEntry>;

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ChunkDataEntry {
    Block(Block),
}

impl Default for ChunkDataEntry {
    fn default() -> Self {
        Self::Block(Block::AIR)
    }
}

impl ChunkDataEntry {
    pub fn create_voxel_block(&self) -> VoxelBlock {
        match self {
            //ChunkDataEntry::Empty => [Voxel::air(); 16 * 16 * 16],
            ChunkDataEntry::Block(block) => {
                [Voxel::new(Material::from(*block)); VOXEL_BLOCK_VOLUME]
            }
        }
    }
}

pub type ChunkEdge = [[ChunkDataEntry; CHUNK_SIZE]; CHUNK_SIZE];

#[derive(Debug, Clone, Component)]
pub struct ClientChunkEdgeData {
    pub edges: Box<[ChunkEdge; 6]>,
}

#[allow(clippy::type_complexity)]
fn add_edges_system(
    mut commands: ParallelCommands,
    grid: Res<ChunkGrid>,
    added_chunks: Query<
        (Entity, &RenderingWorldFixedChunk),
        (Added<ClientChunkStorage>, Without<ClientChunkEdgeData>),
    >,
    all_chunks: Query<&ClientChunkStorage>,
) {
    if added_chunks.is_empty() {
        return;
    }

    added_chunks.par_iter().for_each(|(entity, chunk)| {
        let mut edges = [[ChunkDataEntry::default(); CHUNK_SIZE]; CHUNK_SIZE];
    });
}

fn update_edges_system(
    grid: Res<ChunkGrid>,
    updated_chunks: Query<
        (&RenderingWorldFixedChunk, &ClientChunkStorage),
        Changed<ClientChunkStorage>,
    >,
    mut edges: Query<&mut ClientChunkEdgeData>,
) {
    if updated_chunks.is_empty() {
        return;
    }
}
