use std::mem::size_of;

use client::world::chunk::chunk_data::{ChunkDataEntry, ChunkStorage};
use game2::compressible::Compressible;
use game2::humanize::humanize_memory;
use game2::material::Block;

#[inline]
fn create_test_chunk() -> ChunkStorage {
    let mut chunk_storage = ChunkStorage::empty();

    //fill half with stone
    chunk_storage.set_many(0..32 * 32 * 10, ChunkDataEntry::Block(Block::STONE));
    //cover with grass
    chunk_storage.set_many(
        32 * 32 * 10..32 * 32 * 11,
        ChunkDataEntry::Block(Block::GRASS),
    );
    //set some random blocks
    chunk_storage.set(
        32 * 32 * 12 + 32 * 2 + 14,
        ChunkDataEntry::Block(Block::DIRT),
    );

    chunk_storage
}

#[test]
fn test_chunk_compression() {
    let chunk = create_test_chunk();
    //let chunk = ChunkStorage::empty();

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
    println!("chunk: {}", humanize_memory(chunk.memory_usage()));
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
