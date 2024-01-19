use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use block_mesh::ndshape::ConstShape3u32;
use block_mesh::{
    GreedyQuadsBuffer, OrientedBlockFace, QuadCoordinateConfig, UnitQuadBuffer, VoxelVisibility,
    RIGHT_HANDED_Y_UP_CONFIG,
};
use rayon::prelude::*;

use game2::chunk::ChunkStorage;
use game2::material::Block;
use game2::{BlockFace, CHUNK_SIZE, FACE_BOTTOM, FACE_EAST, FACE_TOP};

use crate::world::chunk::chunk_data::ChunkDataEntry;
use crate::world::chunk::chunk_data::ChunkEdge;
use crate::world::material::BlockClientData;
use crate::world::MESH_TEXTURE_ATTRIBUTE;

pub const VOXELS_PER_METER: usize = 2;
pub const VOXEL_BLOCK_SIZE: usize = VOXELS_PER_METER;
pub const VOXEL_BLOCK_VOLUME: usize = VOXEL_BLOCK_SIZE * VOXEL_BLOCK_SIZE * VOXEL_BLOCK_SIZE;

pub const VOXEL_CHUNK_SIZE: usize = VOXELS_PER_METER * CHUNK_SIZE + 2;
pub const VOXEL_CHUNK_VOLUME: usize = VOXEL_CHUNK_SIZE * VOXEL_CHUNK_SIZE * VOXEL_CHUNK_SIZE;

pub const VOXEL_CHUNK_MEMORY_USAGE: usize = std::mem::size_of::<VoxelChunk>();

const VOXEL_CHUNK_SHAPE: ConstShape3u32<
    { VOXEL_CHUNK_SIZE as u32 },
    { VOXEL_CHUNK_SIZE as u32 },
    { VOXEL_CHUNK_SIZE as u32 },
> = ConstShape3u32 {};

const COORDS_CONFIG: &QuadCoordinateConfig = &RIGHT_HANDED_Y_UP_CONFIG;
const FACES: &[OrientedBlockFace; 6] = &RIGHT_HANDED_Y_UP_CONFIG.faces;

#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
struct Voxel {
    block: Block,
}

impl Voxel {
    pub fn new(block: Block) -> Self {
        Self { block }
    }

    pub fn import(entry: &ChunkDataEntry) -> [Self; VOXEL_BLOCK_VOLUME] {
        match entry {
            ChunkDataEntry::Block(block) => [Voxel::new(*block); VOXEL_BLOCK_VOLUME],
        }
    }

    pub fn get_faced(
        block: &[Self; VOXEL_BLOCK_VOLUME],
        face: BlockFace,
    ) -> [Self; VOXEL_BLOCK_SIZE * VOXEL_BLOCK_SIZE] {
        let mut faced = [Voxel::new(Block::AIR); VOXEL_BLOCK_SIZE * VOXEL_BLOCK_SIZE];
        for i in 0..VOXEL_BLOCK_SIZE * VOXEL_BLOCK_SIZE {
            faced[i] = block[face.iter_num_to_faced_index(VOXEL_BLOCK_SIZE, i)];
        }
        faced
    }
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        self.block.get_visibility()
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = Block;

    fn merge_value(&self) -> Self::MergeValue {
        self.block
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VoxelChunk {
    //around 500kb of memory hopefully it does not overflow the stack
    //keeping it on the stack does hopefully improve performance by a lot since this data is short lived
    grid: [Voxel; VOXEL_CHUNK_VOLUME],
}

impl VoxelChunk {
    pub fn new(data: &ChunkStorage<ChunkDataEntry>, edges: &[ChunkEdge; 6]) -> Self {
        let mut storage = [Voxel::new(Block::AIR); VOXEL_CHUNK_VOLUME];
        //TODO collect chunk data entries to voxels first to not rerun the import function 8 times per block
        storage.par_iter_mut().enumerate().for_each(|(i, voxel)| {
            let x = i % VOXEL_CHUNK_SIZE;
            let y = (i / VOXEL_CHUNK_SIZE) % VOXEL_CHUNK_SIZE;
            let z = i / VOXEL_CHUNK_SIZE / VOXEL_CHUNK_SIZE;

            //we dont care about the corners and there are not used anyways and determine them is unnecessary work
            if (x == 0 || x == VOXEL_CHUNK_SIZE - 1) as u8
                + (y == 0 || y == VOXEL_CHUNK_SIZE - 1) as u8
                + (z == 0 || z == VOXEL_CHUNK_SIZE - 1) as u8
                > 1
            {
                return;
            }

            //i rely on the compiler to optimize this (looks like it easy could)
            let face = if x == 0 {
                Some(BlockFace::West)
            } else if x == VOXEL_CHUNK_SIZE - 1 {
                Some(BlockFace::East)
            } else if y == 0 {
                Some(BlockFace::Bottom)
            } else if y == VOXEL_CHUNK_SIZE - 1 {
                Some(BlockFace::Top)
            } else if z == 0 {
                Some(BlockFace::South)
            } else if z == VOXEL_CHUNK_SIZE - 1 {
                Some(BlockFace::North)
            } else {
                None
            };

            *voxel = match face {
                None => import_from_data(x, y, z, data),
                Some(face) => import_from_data_edges(x, y, z, face, edges),
            }
        });

        Self { grid: storage }
    }

    pub fn create_greedy_quads(&self) -> GreedyQuadsBuffer {
        let mut buffer = GreedyQuadsBuffer::new(VOXEL_CHUNK_VOLUME);

        block_mesh::greedy_quads(
            &self.grid,
            &VOXEL_CHUNK_SHAPE,
            [0; 3],
            [VOXEL_CHUNK_SIZE as u32 - 1; 3],
            FACES,
            &mut buffer,
        );

        buffer
    }

    pub fn create_greedy_quads_mesh(&self) -> Mesh {
        let buffer = self.create_greedy_quads();

        //https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs#L5
        let num_indices = buffer.quads.num_quads() * 6;
        let num_vertices = buffer.quads.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut data = Vec::<u32>::with_capacity(num_vertices);

        for (group, face) in buffer.quads.groups.as_ref().iter().zip(FACES.iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(
                    &face.quad_mesh_positions(&quad, 1.0 / VOXELS_PER_METER as f32),
                );

                normals.extend_from_slice(&face.quad_mesh_normals());
                //TODO (insert material data)
                let pos = quad.minimum;
                let pos = pos[0]
                    + pos[1] * VOXEL_CHUNK_SIZE as u32
                    + pos[2] * VOXEL_CHUNK_SIZE as u32 * VOXEL_CHUNK_SIZE as u32;

                //removed first bit as it stores the transparency
                //let texture_id = self.voxels[pos as usize].0 & 0x7FFFFFFF;

                //set first 3 bits to the face index
                let texture_id = 0;
                //TODO get texture id for material

                data.extend_from_slice(&[texture_id; 4]);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        //render_mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float32x3(positions), );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        //render_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );

        mesh.set_indices(Some(Indices::U32(indices.clone())));

        mesh.insert_attribute(MESH_TEXTURE_ATTRIBUTE, VertexAttributeValues::Uint32(data));

        mesh
    }

    pub fn create_visible_blocks(&self) -> UnitQuadBuffer {
        let mut buffer = UnitQuadBuffer::new();
        block_mesh::visible_block_faces(
            &self.grid,
            &VOXEL_CHUNK_SHAPE,
            [0; 3],
            [VOXEL_CHUNK_SIZE as u32 - 1; 3],
            FACES,
            &mut buffer,
        );

        buffer
    }

    pub fn create_visible_blocks_mesh(&self) -> Mesh {
        let buffer = self.create_visible_blocks();

        //https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs#L5
        let num_indices = buffer.num_quads() * 6;
        let num_vertices = buffer.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut data = Vec::<u32>::with_capacity(num_vertices);

        for (group, face) in buffer.groups.as_ref().iter().zip(FACES.iter()) {
            for quad in group.iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                let quad = block_mesh::UnorientedQuad {
                    minimum: quad.minimum,
                    width: 1,
                    height: 1,
                };

                positions.extend_from_slice(
                    &face.quad_mesh_positions(&quad, 1.0 / VOXELS_PER_METER as f32),
                );
                normals.extend_from_slice(&face.quad_mesh_normals());

                //TODO (insert material data)
                let pos = quad.minimum;
                let pos = pos[0]
                    + pos[1] * VOXEL_CHUNK_SIZE as u32
                    + pos[2] * VOXEL_CHUNK_SIZE as u32 * VOXEL_CHUNK_SIZE as u32;

                //removed first bit as it stores the transparency
                //let texture_id = self.voxels[pos as usize].0 & 0x7FFFFFFF;

                //set first 3 bits to the face index
                let texture_id = 0;
                //TODO get texture id for material

                data.extend_from_slice(&[texture_id; 4]);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        //render_mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float32x3(positions), );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        //render_mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float32x3(normals));

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );

        mesh.set_indices(Some(Indices::U32(indices.clone())));

        mesh.insert_attribute(MESH_TEXTURE_ATTRIBUTE, VertexAttributeValues::Uint32(data));

        mesh
    }
}

fn import_from_data(x: usize, y: usize, z: usize, data: &ChunkStorage<ChunkDataEntry>) -> Voxel {
    //translate to "inner cube" coordinates
    let (x, y, z) = (x - 1, y - 1, z - 1);
    let (chunk_x, chunk_y, chunk_z) = (
        x / VOXELS_PER_METER,
        y / VOXELS_PER_METER,
        z / VOXELS_PER_METER,
    );
    let (offset_x, offset_y, offset_z) = (
        x % VOXELS_PER_METER,
        y % VOXELS_PER_METER,
        z % VOXELS_PER_METER,
    );
    let entry = data.get(chunk_x as u32, chunk_y as u32, chunk_z as u32);

    let block = Voxel::import(entry);
    block[VOXEL_BLOCK_SIZE * VOXEL_BLOCK_SIZE * offset_z + VOXEL_BLOCK_SIZE * offset_y + offset_x]
}

fn import_from_data_edges(
    x: usize,
    y: usize,
    z: usize,
    face: BlockFace,
    edges: &[ChunkEdge; 6],
) -> Voxel {
    let (x, y, z) = ((x.max(1) - 1), y.max(1) - 1, z.max(1) - 1);
    let (chunk_x, chunk_y, chunk_z) = (
        x / VOXELS_PER_METER,
        y / VOXELS_PER_METER,
        z / VOXELS_PER_METER,
    );
    let (offset_x, offset_y, offset_z) = (
        x % VOXELS_PER_METER,
        y % VOXELS_PER_METER,
        z % VOXELS_PER_METER,
    );

    //TODO check if this is correct
    let (entry, index) = match face {
        BlockFace::Top => (
            &edges[FACE_TOP as usize][chunk_x + chunk_z * CHUNK_SIZE],
            offset_x + offset_y * VOXELS_PER_METER,
        ),
        BlockFace::Bottom => (
            &edges[FACE_BOTTOM as usize][chunk_x + chunk_z * CHUNK_SIZE],
            offset_x + offset_y * VOXELS_PER_METER,
        ),
        BlockFace::East => (
            &edges[FACE_EAST as usize][chunk_y + chunk_z * CHUNK_SIZE],
            offset_x + offset_y * VOXELS_PER_METER,
        ),
        BlockFace::West => (
            &edges[FACE_EAST as usize][chunk_y + chunk_z * CHUNK_SIZE],
            offset_x + offset_y * VOXELS_PER_METER,
        ),
        BlockFace::North => (
            &edges[FACE_EAST as usize][chunk_x + chunk_y * CHUNK_SIZE],
            offset_x + offset_y * VOXELS_PER_METER,
        ),
        BlockFace::South => (
            &edges[FACE_EAST as usize][chunk_x + chunk_y * CHUNK_SIZE],
            offset_x + offset_y * VOXELS_PER_METER,
        ),
    };
    let voxels = Voxel::import(entry);
    voxels[index]
}

#[cfg(test)]
mod tests {
    use game2::humanize::humanize_memory;

    use super::*;

    #[test]
    fn test_voxel_chunk_memory_usage() {
        println!(
            "VoxelChunk memory usage: {}",
            humanize_memory(VOXEL_CHUNK_MEMORY_USAGE)
        );
    }
}
