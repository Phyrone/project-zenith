use bevy::prelude::Entity;
use std::mem::size_of;

use rayon::prelude::*;

use client::world::chunk::chunk_data::{ChunkDataEntry, ChunkDataStorage};
use game2::compressible::Compressible;
use game2::humanize::humanize_memory;
use game2::material::{Block, WoodData};
use game2::CHUNK_VOLUME;

#[inline]
fn create_test_chunk() -> ChunkDataStorage {
    let mut chunk_storage = ChunkDataStorage::empty();

    //fill half with stone
    chunk_storage.set_many(0..32 * 32 * 10, ChunkDataEntry::Block(Entity::from_raw(1)));
    //cover with grass
    chunk_storage.set_many(
        32 * 32 * 10..32 * 32 * 11,
        ChunkDataEntry::Block(Entity::from_raw(2)),
    );
    //set some random blocks
    chunk_storage.set(
        32 * 32 * 12 + 32 * 2 + 14,
        ChunkDataEntry::Block(Entity::from_raw(3)),
    );

    chunk_storage
}

#[inline]
fn create_worst_case_test_chunk() -> ChunkDataStorage {
    let mut chunk = vec![ChunkDataEntry::empty(); CHUNK_VOLUME];
    chunk.par_iter_mut().enumerate().for_each(|(i, entry)| {
        let id = i % 16;
        *entry = match id {
            0 => ChunkDataEntry::Empty,
            1..=12 => ChunkDataEntry::Block(Entity::from_raw(id as u32)),
            _ => ChunkDataEntry::Empty,
        };
    });

    return ChunkDataStorage::new(&chunk);
}

#[test]
fn test_chunk_compression() {
    let chunk = create_worst_case_test_chunk();
    //let chunk = ChunkStorage::empty();

    let entry_size = size_of::<ChunkDataEntry>();
    println!("entry size: {} bytes", humanize_memory(entry_size));

    println!("creating compressed versions...");
    let binary_raw = bincode::serialize(&chunk).unwrap();
    let lz4_compressed = chunk.compress_lz4();
    let snappy_compressed = chunk.compress_snappy();
    let zstd_compressed = chunk.compress_zstd();
    let zstd_best_compressed = chunk.compress_zstd_best();
    let lzma_extreme_compressed = chunk.compress_lzma_extreme();

    println!("results:");
    println!(
        "uncompressed:theoretical: {} bytes",
        humanize_memory(entry_size * 32 * 32 * 32)
    );
    println!("chunk: {}", humanize_memory(chunk.memory_usage()));
    println!("raw: {}", humanize_memory(binary_raw.len()));
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
    println!(
        "lzma - extreme: {}",
        humanize_memory(lzma_extreme_compressed.len_compressed())
    );
}
