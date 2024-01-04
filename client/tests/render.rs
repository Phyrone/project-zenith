use client::world::chunk::voxel::{create_voxel_block, Voxel, VoxelChunk};
use criterion::{Bencher, black_box, Criterion, criterion_group, criterion_main};
use game2::material::Material;

fn create_test_voxel_chunk() -> Box<VoxelChunk> {
    let mut voxel_chunk = Box::<VoxelChunk>::default();

    voxel_chunk.set
    //voxel_chunk.set_block(0, 0, 0, create_voxel_block(Material::DIRT));

    voxel_chunk
}


#[test]
fn create_mesh1() {
    Voxel::new(Material::AIR);
    dbg!("creating chunk");
    let voxel_chunk = create_test_voxel_chunk();
    dbg!("chunk created");
    voxel_chunk.create_mesh();
}