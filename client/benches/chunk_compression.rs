use client::world::chunk::chunk_data::{ChunkDataEntry, ChunkDataStorage};
use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use game2::compressible::Compressible;
use game2::material::{Block, WoodData, WoodPlanksData};

criterion_group!(benches, chunk_compression);

criterion_main!(benches);

fn chunk_compression(criterion: &mut Criterion) {
    criterion.bench_function("chunk compression (lz4)", bench_chunk_compression_lz4);
    criterion.bench_function("chunk compression (snappy)", bench_chunk_compression_snappy);
    for level in [1, 2, 3, 5, 8, 12, 15, 20, 22] {
        criterion.bench_function(
            format!("chunk compression (zstd - {})", level).as_str(),
            |bencher| bench_chunk_compression_zstd(bencher, level),
        );
    }

    criterion.bench_function("chunk decompression (lz4)", bench_chunk_decompression_lz4);
    criterion.bench_function(
        "chunk decompression (snappy)",
        bench_chunk_decompression_snappy,
    );
    for level in [1, 2, 3, 5, 8, 12, 15, 20, 22] {
        criterion.bench_function(
            format!("chunk decompression (zstd - {})", level).as_str(),
            |bencher| bench_chunk_decompression_zstd(bencher, level),
        );
    }
}

#[inline]
fn create_test_chunk() -> ChunkDataStorage {
    let mut chunk_storage = ChunkDataStorage::empty();

    //fill half with stone
    chunk_storage.set_many(0..32, 0..20, 0..32, ChunkDataEntry::Block(Block::STONE));
    //cover with grass
    chunk_storage.set_many(0..32, 20..20, 0..32, ChunkDataEntry::Block(Block::GRASS));
    //set some random blocks
    chunk_storage.set(1, 14, 1, ChunkDataEntry::Block(Block::DIRT));
    chunk_storage.set(1, 1, 1, ChunkDataEntry::Block(Block::LEAVES));
    chunk_storage.set(
        23,
        22,
        23,
        ChunkDataEntry::Block(Block::WOOD(WoodData::default())),
    );
    chunk_storage.set_many(
        13..20,
        5..7,
        20..22,
        ChunkDataEntry::Block(Block::WoodPlanks(WoodPlanksData::default())),
    );

    chunk_storage
}

fn bench_chunk_compression_lz4(criterion: &mut Bencher) {
    let chunk = create_test_chunk();

    criterion.iter(|| {
        let compressed = black_box(&chunk).compress_lz4();
        black_box(compressed);
    });
}

fn bench_chunk_compression_zstd(criterion: &mut Bencher, level: i32) {
    let chunk = create_test_chunk();
    criterion.iter(|| {
        let compressed = black_box(&chunk).compress_zstd_with_level(level);
        black_box(compressed);
    });
}

fn bench_chunk_compression_snappy(criterion: &mut Bencher) {
    let chunk = create_test_chunk();
    criterion.iter(|| {
        let compressed = black_box(&chunk).compress_snappy();
        black_box(compressed);
    });
}

fn bench_chunk_decompression_lz4(criterion: &mut Bencher) {
    let chunk = ClientChunkStorage::empty();
    let compressed = chunk.compress_lz4();

    criterion.iter(|| {
        let decompressed = black_box(&compressed).decompress();
        black_box(decompressed);
    });
}

fn bench_chunk_decompression_zstd(criterion: &mut Bencher, level: i32) {
    let chunk = create_test_chunk();
    let compressed = chunk.compress_zstd_with_level(level);

    criterion.iter(|| {
        let decompressed = black_box(&compressed).decompress();
        black_box(decompressed);
    });
}

fn bench_chunk_decompression_snappy(criterion: &mut Bencher) {
    let chunk = create_test_chunk();
    let compressed = chunk.compress_snappy();

    criterion.iter(|| {
        let decompressed = black_box(&compressed).decompress();
        black_box(decompressed);
    });
}
