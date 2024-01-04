use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};

use client::world::chunk::voxel::{create_voxel_block, VoxelChunk};
use game2::material::{Material, WoodData};

criterion_group!(benches, rendering);

criterion_main!(benches);

fn rendering(criterion: &mut Criterion) {
    criterion.bench_function("create voxel chunk mesh", create_mesh1);
}

fn create_test_voxel_chunk() -> VoxelChunk {
    let mut voxel_chunk = VoxelChunk::default();
    voxel_chunk.set_many(0..32, 0..20, 0..32, create_voxel_block(Material::STONE));
    voxel_chunk.set_block(
        16,
        21,
        16,
        create_voxel_block(Material::WOOD(WoodData::default())),
    );

    voxel_chunk
}

fn create_mesh1(bencher: &mut Bencher) {
    let voxel_chunk = create_test_voxel_chunk();

    bencher.iter(|| {
        black_box(&voxel_chunk).create_mesh();
    });
}
