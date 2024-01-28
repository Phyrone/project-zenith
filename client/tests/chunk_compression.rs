use std::mem::size_of;

use client::world::chunk::chunk_data::ChunkDataEntry;
use game2::chunk::ChunkStorage;
use game2::compressible::Compressible;
use game2::humanize::humanize_memory;
use game2::material::{Block, WoodData, WoodPlanksData};

#[inline]
fn create_test_chunk() -> ChunkStorage<ChunkDataEntry> {
    let mut chunk_storage = ChunkStorage::empty();

    //fill half with stone
    chunk_storage.set_many(0..32, 0..20, 0..32, ChunkDataEntry::Block(Block::STONE));
    //cover with grass
    chunk_storage.set_many(0..32, 20..20, 0..32, ChunkDataEntry::Block(Block::GRASS));
    //set some random blocks
    chunk_storage.set(1, 14, 1, ChunkDataEntry::Block(Block::DIRT));
    chunk_storage.set(1, 1, 1, ChunkDataEntry::Block(Block::LEAVES));
    chunk_storage.set(23, 22, 23, ChunkDataEntry::Block(Block::WOOD(WoodData::default())));
    chunk_storage.set_many(13..20, 5..7, 20..22, ChunkDataEntry::Block(Block::WoodPlanks(WoodPlanksData::default())));

    chunk_storage
}

#[test]
fn test_chunk_compression() {
    let chunk = create_test_chunk();

    let entry_size = size_of::<ChunkDataEntry>();
    println!("entry size: {} bytes", humanize_memory(entry_size));

    println!("creating compressed versions...");
    let lz4_compressed = chunk.compress_lz4();
    let snappy_compressed = chunk.compress_snappy();
    let zstd_compressed = chunk.compress_zstd();
    let zstd_best_compressed = chunk.compress_zstd_best();

    println!("results:");
    println!(
        "uncompressed:theoretical: {} bytes",
        humanize_memory(entry_size * 32 * 32 * 32)
    );
    println!("raw: {}", humanize_memory(lz4_compressed.len_data()));
    println!("lz4: {}", humanize_memory(lz4_compressed.len_compressed()));
    println!(
        "snappy: {}",
        humanize_memory(snappy_compressed.len_compressed())
    );
    println!(
        "zstd: {}",
        humanize_memory(zstd_compressed.len_compressed())
    );
    println!(
        "zstd - best: {}",
        humanize_memory(zstd_best_compressed.len_compressed())
    );
}
