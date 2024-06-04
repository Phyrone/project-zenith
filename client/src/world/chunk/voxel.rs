use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bitflags::bitflags;
use block_mesh::ndshape::RuntimeShape;
use block_mesh::{
    GreedyQuadsBuffer, QuadCoordinateConfig, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};
use hashbrown::HashMap;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

use common::CHUNK_SIZE;

pub type EntityGroupedMesh = Vec<(Entity, Mesh)>;

const COORDS_CONFIG: &QuadCoordinateConfig = &RIGHT_HANDED_Y_UP_CONFIG;

bitflags! {
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
    pub struct VoxelFlags: u8{
        const TRANSLUCENT = 0x_0000_0001;

    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Voxel {
    pub flags: VoxelFlags,
    pub surfaces: [u64; 6],
}

impl Voxel {
    const EMPTY: Voxel = Voxel {
        flags: VoxelFlags::empty(),
        surfaces: EMPTY_SURFACES,
    };
}

const EMPTY_SURFACES: [u64; 6] = [0; 6];

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.surfaces == EMPTY_SURFACES {
            VoxelVisibility::Empty
        } else if self.flags.contains(VoxelFlags::TRANSLUCENT) {
            VoxelVisibility::Translucent
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = [u64; 6];

    fn merge_value(&self) -> Self::MergeValue {
        self.surfaces
    }
}

/* VoxelRenderData */

#[derive(Debug, Clone, Component)]
#[component(storage = "SparseSet")]
pub struct VoxelRenderData {
    //Copy on write array of voxels
    voxels: Vec<Voxel>,
    size: usize,
}

impl VoxelRenderData {
    pub fn new(size: usize) -> Self {
        let voxels = vec![Voxel::EMPTY; size * size * size].into();
        VoxelRenderData { voxels, size }
    }

    pub fn voxels(&self) -> &[Voxel] {
        &self.voxels
    }

    //Edit voxels (this will create a copy if there is another reference to the voxels)
    pub fn voxel_mut(&mut self) -> &mut [Voxel] {
        self.voxels.as_mut_slice()
    }

    pub fn volume(&self) -> usize {
        self.voxels.len()
    }

    pub fn generate_mesh(&self, usage: RenderAssetUsages) -> EntityGroupedMesh {
        if self.voxels.is_empty() {
            return Vec::new();
        }
        let size = self.size as u32;
        let shape = [size, size, size];
        let shape = RuntimeShape::<u32, 3>::new(shape);
        let mut buffer = GreedyQuadsBuffer::new(size as usize);
        block_mesh::greedy_quads(
            &self.voxels,
            &shape,
            [0; 3],
            [size - 1; 3],
            &COORDS_CONFIG.faces,
            &mut buffer,
        );
        construct_grouped_mesh(&buffer, &self.voxels, usage, size)
    }
}

/* Functions */

pub fn construct_grouped_mesh(
    buffer: &GreedyQuadsBuffer,
    data: &[Voxel],
    usage: RenderAssetUsages,
    size: u32,
) -> EntityGroupedMesh {
    let quads = buffer
        .quads
        .groups
        .par_iter()
        .zip(COORDS_CONFIG.faces.par_iter().enumerate())
        .flat_map(|(quads, (face_index, face))| {
            quads.into_par_iter().filter_map(move |quad| {
                let pos = quad.minimum[0]
                    + quad.minimum[1] * size as u32
                    + quad.minimum[2] * size as u32 * size as u32;
                let voxel = &data[pos as usize];
                let iden = voxel.surfaces[face_index];

                if iden == 0 {
                    return None;
                }
                Some((iden, quad, face))
            })
        })
        .collect::<Vec<_>>();

    //create meshes grouped by resource
    #[derive(Default)]
    struct MeshPrep {
        indices: Vec<u32>,
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        uvs: Vec<[f32; 2]>,
    }

    let mut groups = HashMap::<u64, MeshPrep>::new();
    for (iden, quad, face) in quads {
        let prep = groups.entry(iden).or_default();
        prep.indices
            .extend_from_slice(&face.quad_mesh_indices(prep.positions.len() as u32));
        prep.positions
            .extend_from_slice(&face.quad_mesh_positions(quad, CHUNK_SIZE as f32 / size as f32));
        prep.normals.extend_from_slice(&face.quad_mesh_normals());
        //prep.uvs.extend_from_slice(&[[0.0; 2]; 4]);
        prep.uvs
            .extend_from_slice(&[[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]]);
    }

    groups
        .into_iter()
        .map(|(iden, prep)| {
            (
                Entity::from_bits(iden),
                construct_mesh(prep.indices, prep.positions, prep.normals, prep.uvs, usage),
            )
        })
        .collect()
}

#[inline]
fn construct_mesh(
    mut indices: Vec<u32>,
    mut positions: Vec<[f32; 3]>,
    mut normals: Vec<[f32; 3]>,
    mut uvs: Vec<[f32; 2]>,
    usage: RenderAssetUsages,
) -> Mesh {
    //shrink the vectors so they don't take up unnecessary space when passed to the mesh
    indices.shrink_to_fit();
    positions.shrink_to_fit();
    normals.shrink_to_fit();
    uvs.shrink_to_fit();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage);

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
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
