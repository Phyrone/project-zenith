use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use client::world::chunk::chunk_data::{ChunkDataEntry, ChunkEdge, ClientChunkStorage};
use client::world::chunk::voxel2::VoxelChunk;
use game2::material::Block;
use game2::CHUNK_SIZE;

criterion_group!(benches, rendering);

criterion_main!(benches);

fn rendering(criterion: &mut Criterion) {
    criterion.bench_function("voxel chunk mesh (greedy quads)", create_mesh1);
    criterion.bench_function("voxel chunk mesh (visible blocks)", create_mesh2);
}

fn create_test_voxel_chunk() -> (ClientChunkStorage, [ChunkEdge; 6]) {
    let mut chunk_data = ClientChunkStorage::new(
        &[ChunkDataEntry::Block(Block::AIR); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
    );

    chunk_data.set_many(
        0..CHUNK_SIZE as u32,
        0..20,
        0..CHUNK_SIZE as u32,
        ChunkDataEntry::Block(Block::DIRT),
    );

    (
        chunk_data,
        [[ChunkDataEntry::empty(); CHUNK_SIZE * CHUNK_SIZE]; 6],
    )
}

fn create_mesh1(bencher: &mut Bencher) {
    let (voxel_chunk, edge) = create_test_voxel_chunk();

    bencher.iter(|| {
        let chunk = VoxelChunk::new(black_box(&voxel_chunk), black_box(&edge));
        let mesh = chunk.create_greedy_quads_mesh();
        black_box(mesh);
    });
}

fn create_mesh2(bencher: &mut Bencher) {
    let (voxel_chunk, edge) = create_test_voxel_chunk();

    bencher.iter(|| {
        let chunk = VoxelChunk::new(black_box(&voxel_chunk), black_box(&edge));
        let mesh = chunk.create_visible_blocks_mesh();
        black_box(mesh);
    });
}
