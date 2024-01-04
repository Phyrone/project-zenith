use std::ops::Deref;

use bevy::utils::petgraph::visit::Walker;
use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use game2::chunk::ChunkStorage;
use game2::CHUNK_VOLUME;

criterion_group!(benches, chunk);

criterion_main!(benches);

fn chunk(criterion: &mut Criterion) {
    criterion.bench_function("create chunk (new)", bench_empty_chunk_creation);
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct TestBlock(u128);

type TestChunkStorage = ChunkStorage<TestBlock>;

fn bench_empty_chunk_creation(bencher: &mut Bencher) {
    let mut input_data = Box::new([TestBlock::default(); CHUNK_VOLUME]);

    //create some random data with about 200 different block ids
    for i in 0..CHUNK_VOLUME {
        let id = i % 200;
        input_data[i] = TestBlock(id as u128);
    }

    bencher.iter(|| {
        let input = black_box(input_data.deref());
        let chunk_storage = TestChunkStorage::new(input);
        black_box(chunk_storage);
    });
}
