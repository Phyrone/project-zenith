use std::ops::Range;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use block_mesh::ndshape::ConstShape3u32;
use block_mesh::{GreedyQuadsBuffer, RIGHT_HANDED_Y_UP_CONFIG};
use rayon::prelude::*;

use game2::material::Material;
use game2::{Face, CHUNK_SIZE};

use crate::world::chunk::chunk_data::ChunkDataEntry;
use crate::world::chunk::chunk_data::{ClientChunkEdgeData, ClientChunkStorage};
use crate::world::material::MaterialClientData;
use crate::world::MESH_TEXTURE_ATTRIBUTE;

#[derive(Default)]
pub struct VoxelChunkPlugin;

impl Plugin for VoxelChunkPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Voxel(Material);

impl Voxel {
    pub fn air() -> Self {
        Self(Material::AIR)
    }
    pub fn new(material: Material) -> Self {
        Self(material)
    }
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> block_mesh::VoxelVisibility {
        self.0.get_visibility()
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = Material;

    fn merge_value(&self) -> Self::MergeValue {
        self.0.clone()
    }
}

/* Full */

pub const VOXELS_PER_METER: usize = 2;

pub const VOXEL_BLOCK_ROW_SIZE: usize = VOXELS_PER_METER;
pub const VOXEL_BLOCK_VOLUME: usize =
    VOXEL_BLOCK_ROW_SIZE * VOXEL_BLOCK_ROW_SIZE * VOXEL_BLOCK_ROW_SIZE;

pub const VOXEL_CHUNK_ROW_SIZE: usize = CHUNK_SIZE * VOXELS_PER_METER + 2;
pub const VOXEL_CHUNK_VOLUME: usize =
    VOXEL_CHUNK_ROW_SIZE * VOXEL_CHUNK_ROW_SIZE * VOXEL_CHUNK_ROW_SIZE;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoxelChunk {
    voxels: Box<[Voxel; VOXEL_CHUNK_VOLUME]>,
}

impl Default for VoxelChunk {
    fn default() -> Self {
        Self {
            voxels: Box::new([Voxel::default(); VOXEL_CHUNK_VOLUME]),
        }
    }
}

impl VoxelChunk {
    #[inline]
    fn cords_into_index(x: usize, y: usize, z: usize) -> usize {
        x + (y * VOXEL_CHUNK_ROW_SIZE) + (z * VOXEL_CHUNK_ROW_SIZE * VOXEL_CHUNK_ROW_SIZE)
    }

    #[inline]
    fn index_into_cords(index: usize) -> (usize, usize, usize) {
        let x = index % VOXEL_CHUNK_ROW_SIZE;
        let y = (index / VOXEL_CHUNK_ROW_SIZE) % VOXEL_CHUNK_ROW_SIZE;
        let z = index / (VOXEL_CHUNK_ROW_SIZE * VOXEL_CHUNK_ROW_SIZE);

        (x, y, z)
    }

    #[inline]
    fn block_cords_into_chunk_cords(x: usize, y: usize, z: usize) -> (usize, usize, usize) {
        (
            x * VOXELS_PER_METER + 1,
            y * VOXELS_PER_METER + 1,
            z * VOXELS_PER_METER + 1,
        )
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: VoxelBlock) {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("VoxelChunk cords out of bounds");
        }

        //TODO read values from voxel black and copy them into the chunk
        for i in 0..VOXEL_BLOCK_VOLUME {
            let local_x = i % VOXELS_PER_METER;
            let local_y = (i / VOXELS_PER_METER) % VOXELS_PER_METER;
            let local_z = i / (VOXELS_PER_METER * VOXELS_PER_METER);

            let into_index = Self::cords_into_index(
                local_x + i * VOXELS_PER_METER + 1,
                local_y + y * VOXELS_PER_METER + 1,
                local_z + z * VOXELS_PER_METER + 1,
            );
            self.voxels[into_index] = block[i];
        }
    }

    pub fn set_many(
        &mut self,
        x: Range<usize>,
        y: Range<usize>,
        z: Range<usize>,
        block: VoxelBlock,
    ) {
        let (x_len, y_len, z_len) = (x.end - x.start, y.end - y.start, z.end - z.start);
        let batch_volume = x_len * y_len * z_len;
        let (x, y, z) = (x.start, y.start, z.start);

        let (x_len_voxel, y_len_voxel) = (x_len * VOXELS_PER_METER, y_len * VOXELS_PER_METER);

        for i in 0..batch_volume {
            let local_x = i % x_len_voxel;
            let local_y = (i / x_len_voxel) % y_len_voxel;
            let local_z = i / (x_len_voxel * y_len_voxel);

            let voxel_block_x = local_x % VOXELS_PER_METER;
            let voxel_block_y = local_y % VOXELS_PER_METER;
            let voxel_block_z = local_z % VOXELS_PER_METER;

            let into_index = Self::cords_into_index(
                local_x + x * VOXELS_PER_METER + 1,
                local_y + y * VOXELS_PER_METER + 1,
                local_z + z * VOXELS_PER_METER + 1,
            );
            let from_index = voxel_block_x
                + voxel_block_y * VOXELS_PER_METER
                + voxel_block_z * VOXELS_PER_METER * VOXELS_PER_METER;

            self.voxels[into_index] = block[from_index];
        }
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> VoxelBlock {
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("VoxelChunk cords out of bounds");
        }
        let mut block = [Voxel::default(); VOXEL_BLOCK_VOLUME];

        block.par_iter_mut().enumerate().for_each(|(i, voxel)| {
            let local_x = i % VOXELS_PER_METER;
            let local_y = (i / VOXELS_PER_METER) % VOXELS_PER_METER;
            let local_z = i / (VOXELS_PER_METER * VOXELS_PER_METER);

            let into_index = Self::cords_into_index(
                local_x + i * VOXELS_PER_METER + 1,
                local_y + y * VOXELS_PER_METER + 1,
                local_z + z * VOXELS_PER_METER + 1,
            );
            *voxel = self.voxels[into_index];
        });

        block
    }

    pub fn create_greedy_quads_buffer(&self) -> GreedyQuadsBuffer {
        let shape: ConstShape3u32<
            { VOXEL_CHUNK_ROW_SIZE as u32 },
            { VOXEL_CHUNK_ROW_SIZE as u32 },
            { VOXEL_CHUNK_ROW_SIZE as u32 },
        > = ConstShape3u32 {};
        let faces = &RIGHT_HANDED_Y_UP_CONFIG.faces;

        let mut buffer = GreedyQuadsBuffer::new(VOXEL_CHUNK_VOLUME);

        block_mesh::greedy_quads(
            self.voxels.as_ref(),
            &shape,
            [0; 3],
            [VOXEL_CHUNK_ROW_SIZE as u32 - 1; 3],
            faces,
            &mut buffer,
        );

        buffer
    }

    /// creates a mesh based on the voxels given into the chunk
    pub fn create_mesh(&self) -> Mesh {
        let buffer = self.create_greedy_quads_buffer();

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
            .zip(RIGHT_HANDED_Y_UP_CONFIG.faces.iter())
            .enumerate()
        {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(
                    &face.quad_mesh_positions(&quad, 1.0 / VOXELS_PER_METER as f32),
                );
                normals.extend_from_slice(&face.quad_mesh_normals());
                //TODO (insert material data)
                let pos = quad.minimum;
                let pos = pos[0]
                    + pos[1] * VOXEL_CHUNK_ROW_SIZE as u32
                    + pos[2] * VOXEL_CHUNK_ROW_SIZE as u32 * VOXEL_CHUNK_ROW_SIZE as u32;

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

    fn genrate(data: &ClientChunkStorage, edges: &ClientChunkEdgeData) {
        let mut buffer = Vec::with_capacity(VOXEL_CHUNK_VOLUME);
        (0..VOXEL_CHUNK_VOLUME)
            .into_par_iter()
            .map(|i| {
                let (x, y, z) = Self::index_into_cords(i);
                find_matching_entry(data, edges, x, y, z)
            })
            .map(|entry| entry.create_voxel_block())
            .collect_into_vec(&mut buffer);
    }
}

fn find_matching_entry(
    data: &ClientChunkStorage,
    edges: &ClientChunkEdgeData,
    x: usize,
    y: usize,
    z: usize,
) -> ChunkDataEntry {
    todo!()
}

type FullVoxelChunk = VoxelChunk;
//type Far1VoxelChunk = VoxelChunk<LOD_FAR1_VOXEL_CHUNK_SIZE>;
//type Far2VoxelChunk = VoxelChunk<LOD_FAR2_VOXEL_CHUNK_SIZE>;

pub type VoxelBlock = [Voxel; VOXEL_BLOCK_VOLUME];

pub fn create_voxel_block(material: Material) -> VoxelBlock {
    [Voxel::new(material); VOXEL_BLOCK_VOLUME]
}

fn get_edge(block: &VoxelBlock, face: Face) -> [VoxelBlock; VOXELS_PER_METER * VOXELS_PER_METER] {
    let mut edge = [Voxel::default(); VOXELS_PER_METER * VOXELS_PER_METER];

    todo!()
}
