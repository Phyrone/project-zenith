use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::utils::petgraph::visit::Walker;
use block_mesh::ndshape::ConstShape3u32;
use block_mesh::{GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};

use game2::CHUNK_SIZE;
use crate::world::MESH_TEXTURE_ATTRIBUTE;

#[derive(Default)]
pub struct VoxelChunkPlugin;

impl Plugin for VoxelChunkPlugin {
    fn build(&self, app: &mut App) {}
}



#[derive(Clone, Default, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Voxel(u32);

impl Voxel {
    
    pub fn air() -> Self {
        Self(0)
    }
    pub fn new(id: u32, transparency: bool) -> Self {
        let mut data = id;
        if transparency {
            data |= 1 << 31;
        }
        Self(data)
    }
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> block_mesh::VoxelVisibility {
        if self.0 == 0 {
            block_mesh::VoxelVisibility::Empty
        } else if (self.0 >> 31) == 1 {
            block_mesh::VoxelVisibility::Translucent
        } else {
            block_mesh::VoxelVisibility::Opaque
        }
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

/* Full */

pub const LOD_FULL_VOXELS_PER_METER: usize = 16;
pub const LOD_FULL_VOXEL_CHUNK_SIZE: usize = CHUNK_SIZE * LOD_FULL_VOXELS_PER_METER + 2;
pub const LOD_FULL_VOXEL_CHUNK_VOLUME: usize =
    LOD_FULL_VOXEL_CHUNK_SIZE * LOD_FULL_VOXEL_CHUNK_SIZE * LOD_FULL_VOXEL_CHUNK_SIZE;

/* Far1 */
//pub const LOD_FAR1_VOXELS_PER_METER: usize = 4;
//pub const LOD_FAR1_VOXEL_CHUNK_SIZE: usize = CHUNK_SIZE * LOD_FAR1_VOXELS_PER_METER + 2;
//pub const LOD_FAR1_VOXEL_CHUNK_VOLUME: usize =
//    LOD_FAR1_VOXEL_CHUNK_SIZE * LOD_FAR1_VOXEL_CHUNK_SIZE * LOD_FAR1_VOXEL_CHUNK_SIZE;

/* Far2 */
//pub const LOD_FAR2_VOXELS_PER_METER: usize = 1;
//pub const LOD_FAR2_VOXEL_CHUNK_SIZE: usize = CHUNK_SIZE * LOD_FAR2_VOXELS_PER_METER + 2;
//pub const LOD_FAR2_VOXEL_CHUNK_VOLUME: usize =
//    LOD_FAR2_VOXEL_CHUNK_SIZE * LOD_FAR2_VOXEL_CHUNK_SIZE * LOD_FAR2_VOXEL_CHUNK_SIZE;


//TODO replace with const generics (when rust allows generic parameters in const operations)
const SIZE: usize = LOD_FULL_VOXELS_PER_METER;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, Component)]
pub struct VoxelChunk {
    #[serde(with = "serde_big_array::BigArray")]
    voxels: [Voxel; (SIZE * CHUNK_SIZE + 2) * (SIZE * CHUNK_SIZE + 2) * (SIZE * CHUNK_SIZE + 2)],
}

impl VoxelChunk {
    pub fn new() -> Self {
        const ROW_SIZE: usize = SIZE * CHUNK_SIZE + 2;
        Self {
            voxels: [Voxel::default(); ROW_SIZE * ROW_SIZE * ROW_SIZE],
        }
    }

    fn cords_into_index(x: usize, y: usize, z: usize) -> usize {
        const ROW_SIZE: usize = SIZE * CHUNK_SIZE + 2;

        x + (y * ROW_SIZE) + (z * ROW_SIZE * ROW_SIZE)
    }

    fn index_into_cords(index: usize) -> (usize, usize, usize) {
        const ROW_SIZE: usize = SIZE * CHUNK_SIZE + 2;

        let x = index % ROW_SIZE;
        let y = (index / ROW_SIZE) % ROW_SIZE;
        let z = index / (ROW_SIZE * ROW_SIZE);

        (x, y, z)
    }

    fn block_cords_into_chunk_cords(x: usize, y: usize, z: usize) -> (usize, usize, usize) {
        (x * SIZE + 1, y * SIZE + 1, z * SIZE + 1)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: VoxelBlock) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("VoxelChunk cords out of bounds");
        }

        //TODO read values from voxel black and copy them into the chunk
        for i in 0..(SIZE * SIZE * SIZE) {
            let local_x = i % SIZE;
            let local_y = (i / SIZE) % SIZE;
            let local_z = i / (SIZE * SIZE);

            let into_index = Self::cords_into_index(
                local_x + i * SIZE + 1,
                local_y + y * SIZE + 1,
                local_z + z * SIZE + 1,
            );
            self.voxels[into_index] = block[i];
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> VoxelBlock {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("VoxelChunk cords out of bounds");
        }
        let mut block = [Voxel::default(); SIZE * SIZE * SIZE];
        for i in 0..(SIZE * SIZE * SIZE) {
            let local_x = i % SIZE;
            let local_y = (i / SIZE) % SIZE;
            let local_z = i / (SIZE * SIZE);

            let into_index = Self::cords_into_index(
                local_x + i * SIZE + 1,
                local_y + y * SIZE + 1,
                local_z + z * SIZE + 1,
            );
            block[i] = self.voxels[into_index];
        }

        block
    }

    /// creates a mesh based on the voxels given into the chunk
    pub fn create_mesh(&self) -> Mesh {
        let shape: ConstShape3u32<
            { SIZE as u32 * CHUNK_SIZE as u32 + 2 },
            { SIZE as u32 * CHUNK_SIZE as u32 + 2 },
            { SIZE as u32 * CHUNK_SIZE as u32 + 2 },
        > = ConstShape3u32 {};
        let faces = &RIGHT_HANDED_Y_UP_CONFIG.faces;

        const ROW_SIZE: usize = SIZE * CHUNK_SIZE + 2;
        let mut buffer = GreedyQuadsBuffer::new(ROW_SIZE * ROW_SIZE * ROW_SIZE);

        block_mesh::greedy_quads(
            &self.voxels,
            &shape,
            [0; 3],
            [ROW_SIZE as u32; 3],
            faces,
            &mut buffer,
        );

        //https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs#L5
        let num_indices = buffer.quads.num_quads() * 6;
        let num_vertices = buffer.quads.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut data = Vec::<u32>::with_capacity(num_vertices);

        for (index /* TODO (insert material data) */, (group, face)) in buffer
            .quads
            .groups
            .as_ref()
            .iter()
            .zip(faces.iter())
            .enumerate()
        {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0 / SIZE as f32));
                normals.extend_from_slice(&face.quad_mesh_normals());
                //TODO (insert material data)
                let pos = quad.minimum;
                let pos =
                    pos[0] + pos[1] * ROW_SIZE as u32 + pos[2] * ROW_SIZE as u32 * ROW_SIZE as u32;

                //removed first bit as it stores the transparency
                let texture_id = self.voxels[pos as usize].0 & 0x7FFFFFFF;

                //set first 3 bits to the face index
                let texture_id = texture_id | ((index as u32) << 29);

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

type FullVoxelChunk = VoxelChunk;
//type Far1VoxelChunk = VoxelChunk<LOD_FAR1_VOXEL_CHUNK_SIZE>;
//type Far2VoxelChunk = VoxelChunk<LOD_FAR2_VOXEL_CHUNK_SIZE>;

pub type VoxelBlock = [Voxel; SIZE * SIZE * SIZE];
