#![feature(test)]
extern crate test;

use bevy::asset::AssetContainer;
use bevy::prelude::{Added, App, apply_deferred, Changed, Commands, Component, Entity, IntoSystemConfigs, ParallelCommands, Plugin, Query, Res, SystemSet, Update, Without};

use game2::chunk::ChunkStorage;
use game2::CHUNK_SIZE;
use game2::compressible::Compressible;

use crate::world::block_data::ClientBlockState;
use crate::world::chunk::{ChunkRenderStage, RenderingWorldFixedChunk};
use crate::world::chunk::grid::{ChunkGrid, GRID_NEIGHBOUR_MAP};
use crate::world::chunk::voxel::{Voxel, VoxelBlock};

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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
enum ChunkDataEntry {
    #[default]
    Empty,
    Block(ClientBlockState),
    //
}

impl ChunkDataEntry {
    fn create_voxel_block(&self) -> VoxelBlock {
        match self {
            ChunkDataEntry::Empty => [Voxel::air(); 16 * 16 * 16],
            ChunkDataEntry::Block(block) => [Voxel::new(block.material as u32, false); 16 * 16 * 16]
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
        let mut edges = [[ChunkDataEntry::Empty; CHUNK_SIZE]; CHUNK_SIZE];


    });
}

fn update_edges_system(
    grid: Res<ChunkGrid>,
    updated_chunks: Query<(&RenderingWorldFixedChunk, &ClientChunkStorage), Changed<ClientChunkStorage>>,
    mut edges: Query<&mut ClientChunkEdgeData>,
) {
    if updated_chunks.is_empty() {
        return;
    }
}

#[cfg(test)]
mod test_chunkdata {
    use super::*;
    use test::Bencher;
    use test::black_box;


    use bevy::utils::petgraph::visit::Walker;
    use game2::compressible::Compressible;

    use crate::world::chunk::chunk_data::ClientChunkStorage;

    #[test]
    fn test_chunk_compression() {
        let chunk = ClientChunkStorage::empty();
        let compressed = chunk.compress_zstd_best();
        println!(
            "compressed: {} bytes, uncompressed: {} bytes",
            compressed.len_compressed(),
            compressed.len_data()
        );

        let decompressed = compressed.decompress();

        assert_eq!(chunk, decompressed);
    }

    #[bench]
    fn bench_pow(b: &mut Bencher) {
        // Optionally include some setup
        let x: f64 = 211.0 * 11.0;
        let y: f64 = 301.0 * 103.0;

        b.iter(|| {
            // Inner closure, the actual test
            for i in 1..100 {
                black_box(x.powf(y).powf(x));
            }
        });
    }
}
