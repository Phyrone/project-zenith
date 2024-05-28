use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

use ahash::AHasher;
use bevy::log::trace;
use bevy::prelude::{Has, Mesh};
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::utils::label::DynHash;
use block_mesh::ndshape::RuntimeShape;
use block_mesh::{
    GreedyQuadsBuffer, OrientedBlockFace, QuadCoordinateConfig, UnitQuadBuffer, UnorientedQuad,
    VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};
use hashbrown::HashMap;
use itertools::Itertools;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use unstructured::{Document, Unstructured};

use game2::bundle::Bundle;
use game2::registry::RegistryEntry;
use game2::{Direction, CHUNK_SIZE};

use crate::world::chunk::chunk_data::{ChunkDataEntry, ChunkDataStorage, ClientChunkData};
use crate::world::chunk::TextureIden;
use crate::world::material::{MaterialRegistry, AIR_MATERIAL_ID};

const COORDS_CONFIG: &QuadCoordinateConfig = &RIGHT_HANDED_Y_UP_CONFIG;

pub type GroupedVoxelMeshes = Vec<(TextureIden, Mesh)>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
enum VoxelMaterialType {
    Physical,
    SingleColor,
    Custom(u16),
    None,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct VoxelMaterialDescription {
    pub id: usize,
    pub metadata: Option<Box<Document>>,
}

impl VoxelMaterialDescription {
    fn voxel_visibility(&self) -> VoxelVisibility {
        self.metadata
            .as_ref()
            .and_then(|metadata| {
                metadata.select("visibility").ok().and_then(|value| {
                    if let Unstructured::String(value) = value {
                        match value.as_str() {
                            "empty" => Some(VoxelVisibility::Empty),
                            "translucent" => Some(VoxelVisibility::Translucent),
                            _ => None,
                        }
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(VoxelVisibility::Opaque)
    }
}

pub struct NoMaterial;

impl Default for VoxelMaterialDescription {
    fn default() -> Self {
        Self {
            category: VoxelMaterialType::None,
            id: AIR_MATERIAL_ID,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voxel {
    material: Option<Arc<VoxelMaterialDescription>>,
    //TODO extra data
}

impl Voxel {
    pub fn empty() -> Self {
        Self { material: None }
    }
    pub fn new(material_data: Arc<VoxelMaterialDescription>) -> Self {
        Self {
            material: Some(material_data),
        }
    }
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if let Some(material) = &self.material {
            material.voxel_visibility()
        } else {
            VoxelVisibility::Empty
        }
    }
}

impl block_mesh::MergeVoxel for Voxel {
    type MergeValue = Option<u64>;

    fn merge_value(&self) -> Self::MergeValue {
        let mut hasher = AHasher::default();
        self.material.as_ref().hash(&mut hasher);
        Some(hasher.finish())
    }
}

impl ChunkDataEntry {
    fn create_voxel(
        &self,
        registry: &MaterialRegistry,
        //TODO resolution and index could be used for more complex entries
        _resolution: usize,
        _index: usize,
    ) -> Voxel {
        match self {
            ChunkDataEntry::Empty => Voxel::empty(),
            ChunkDataEntry::Block(material, _) => {
                let entry = registry
                    .get_by_id(*material)
                    .map(|entry| entry.voxel.clone())
                    .flatten();
                match entry {
                    Some(entry) => Voxel::new(entry),
                    None => Voxel::empty(),
                }
            }
        }
    }
}

pub fn create_voxel_chunk<'render>(
    registry: &'render MaterialRegistry,
    data: &'render ChunkDataStorage,
    neighbors: &'render [Option<&ChunkDataStorage>; 6],
    resolution: usize,
) -> Vec<Voxel> {
    let voxel_chunk_size = resolution * CHUNK_SIZE + 2; //+2 for the faces
    let voxel_chunk_volume = voxel_chunk_size * voxel_chunk_size * voxel_chunk_size;
    let mut voxels = vec![Voxel::empty(); voxel_chunk_volume];

    voxels.par_iter_mut().enumerate().for_each(|(i, voxel)| {
        let (x, y, z) = (
            i % voxel_chunk_size,
            (i / voxel_chunk_size) % voxel_chunk_size,
            i / (voxel_chunk_size * voxel_chunk_size),
        );

        let mode = (x == 0) as i8
            + (y == 0) as i8
            + (z == 0) as i8
            + (x == voxel_chunk_size - 1) as i8
            + (y == voxel_chunk_size - 1) as i8
            + (z == voxel_chunk_size - 1) as i8;

        //skip edges
        match mode {
            //inner
            0 => get_voxel_inner(x, y, z, resolution, voxel_chunk_size, registry, data, voxel),
            //face
            1 => get_voxel_face(
                x,
                y,
                z,
                resolution,
                voxel_chunk_size,
                registry,
                neighbors,
                voxel,
            ),
            //skip corners
            _ => return,
        }
    });

    voxels
}

#[allow(clippy::too_many_arguments)]
fn get_voxel_inner(
    x: usize,
    y: usize,
    z: usize,
    resolution: usize,
    voxel_chunk_size: usize,
    registry: &MaterialRegistry,
    chunk: &ChunkDataStorage,
    into: &mut Voxel,
) {
    let inner_index = (x - 1)
        + (y - 1) * (voxel_chunk_size - 2)
        + (z - 1) * (voxel_chunk_size - 2) * (voxel_chunk_size - 2);
    let inner_entry = chunk.get(inner_index);
    let (inner_x, inner_y, inner_z) = (x - 1, y - 1, z - 1);
    let (sub_x, sub_y, sub_z) = (
        inner_x % resolution,
        inner_y % resolution,
        inner_z % resolution,
    );
    let sub_index = sub_x + sub_y * resolution + sub_z * resolution * resolution;
    *into = inner_entry.create_voxel(registry, resolution, sub_index);
}

#[allow(clippy::too_many_arguments)]
fn get_voxel_face<'render>(
    x: usize,
    y: usize,
    z: usize,
    resolution: usize,
    voxel_chunk_size: usize,
    registry: &'render MaterialRegistry,
    neighbours: &'render [Option<&'render ChunkDataStorage>; 6],
    into: &mut Voxel,
) {
    //0 = east
    //1 = west
    //2 = north
    //3 = south
    //4 = up
    //5 = down

    let direction = if x == 0 {
        Direction::West
    } else if x == voxel_chunk_size - 1 {
        Direction::East
    } else if y == 0 {
        Direction::Down
    } else if y == voxel_chunk_size - 1 {
        Direction::Up
    } else if z == 0 {
        Direction::South
    } else if z == voxel_chunk_size - 1 {
        Direction::North
    } else {
        unreachable!()
    };
    let neighbour = neighbours[direction as usize];
    let neighbour = match neighbour {
        None => return,
        Some(neighbour) => neighbour,
    };
    let face_dimension = voxel_chunk_size - 2;
    //let (voxel_x,voxel_y,voxel_z) = ((x-1)/resolution, (y-1)/resolution, (z-1)/resolution);
    let (voxel_x, voxel_y, voxel_z) = match direction {
        Direction::East => (0, (y - 1) / resolution, (z - 1) / resolution),
        Direction::West => (CHUNK_SIZE - 1, (y - 1) / resolution, (z - 1) / resolution),
        Direction::Up => ((x - 1) / resolution, 0, (z - 1) / resolution),
        Direction::Down => ((x - 1) / resolution, CHUNK_SIZE - 1, (z - 1) / resolution),
        Direction::North => ((x - 1) / resolution, (y - 1) / resolution, 0),
        Direction::South => ((x - 1) / resolution, (y - 1) / resolution, CHUNK_SIZE - 1),
    };

    let (target_x, target_y, target_z) = match direction {
        Direction::West => (CHUNK_SIZE - 1, voxel_y, voxel_z),
        Direction::East => (0, voxel_y, voxel_z),
        Direction::Down => (voxel_x, CHUNK_SIZE - 1, voxel_z),
        Direction::Up => (voxel_x, 0, voxel_z),
        Direction::South => (voxel_x, voxel_y, 0),
        Direction::North => (voxel_x, voxel_y, CHUNK_SIZE - 1),
    };
    let target_i = target_x + target_y * CHUNK_SIZE + target_z * CHUNK_SIZE * CHUNK_SIZE;
    let neighbor_target_entry = neighbour.get(target_i);
    *into = neighbor_target_entry.create_voxel(registry, resolution, 0);
}

pub fn voxels_geedy_quads(voxels: &[Voxel], resolution: usize) -> GreedyQuadsBuffer {
    let size = resolution * CHUNK_SIZE + 2;
    assert_eq!(
        voxels.len(),
        size * size * size,
        "amount of voxels does not match resolution and chunk size"
    );

    let shape = [size as u32, size as u32, size as u32];
    let voxel_chunk_shape = RuntimeShape::<u32, 3>::new(shape);

    let mut buffer = GreedyQuadsBuffer::new(size * size * size);
    block_mesh::greedy_quads(
        voxels,
        &voxel_chunk_shape,
        [0; 3],
        [size as u32 - 1; 3],
        &COORDS_CONFIG.faces,
        &mut buffer,
    );
    buffer
}

pub fn voxels_quads(voxels: &[Voxel], resolution: usize) -> UnitQuadBuffer {
    let size = resolution * CHUNK_SIZE + 2;
    assert_eq!(
        voxels.len(),
        size * size * size,
        "amount of voxels does not match resolution and chunk size"
    );

    let shape = [size as u32, size as u32, size as u32];
    let voxel_chunk_shape = RuntimeShape::<u32, 3>::new(shape);

    let mut buffer = UnitQuadBuffer::new();
    block_mesh::visible_block_faces(
        voxels,
        &voxel_chunk_shape,
        [0; 3],
        [size as u32 - 1; 3],
        &COORDS_CONFIG.faces,
        &mut buffer,
    );
    buffer
}

pub fn voxels_grouped_mesh(
    voxels: &[Voxel],
    resolution: usize,
    usage: RenderAssetUsages,
) -> GroupedVoxelMeshes {
    let buffer = voxels_quads(voxels, resolution);
    let size = resolution * CHUNK_SIZE + 2;
    let buffer = unit_quads_to_greedy_quads(buffer);

    construct_grouped_mesh(&buffer, voxels, usage, resolution, size)
}

pub fn voxels_geedy_mesh(voxels: &[Voxel], resolution: usize, usage: RenderAssetUsages) -> Mesh {
    let buffer = voxels_geedy_quads(voxels, resolution);
    let size = resolution * CHUNK_SIZE + 2;
    quads_buffer_to_mesh(&buffer, usage, resolution, size)
}

pub fn voxels_grouped_greedy_mesh(
    voxels: &[Voxel],
    resolution: usize,
    usage: RenderAssetUsages,
) -> GroupedVoxelMeshes {
    trace!("creating mesh buffer");
    let time = std::time::Instant::now();
    let buffer = voxels_geedy_quads(voxels, resolution);
    let time = time.elapsed();
    trace!("created mesh buffer in {:?}", time);

    let size = resolution * CHUNK_SIZE + 2;
    trace!("constructing grouped mesh");
    let time = std::time::Instant::now();
    let grouped = construct_grouped_mesh(&buffer, voxels, usage, resolution, size);
    let time = time.elapsed();
    trace!("constructed grouped mesh in {:?}", time);

    grouped
}

/// uses a greedy quads buffer to create multiple meshes grouped by resource
pub fn construct_grouped_mesh(
    buffer: &GreedyQuadsBuffer,
    voxel_chunk: &[Voxel],
    usage: RenderAssetUsages,
    resolution: usize,
    size: usize,
) -> GroupedVoxelMeshes {
    let quads = buffer
        .quads
        .groups
        .par_iter()
        .zip(COORDS_CONFIG.faces.par_iter())
        .flat_map(|(quads, face)| {
            let direction = direction_from_oriented_block_face(face);
            quads.into_par_iter().map(move |quad| {
                let pos = quad.minimum[0]
                    + quad.minimum[1] * size as u32
                    + quad.minimum[2] * size as u32 * size as u32;
                let material = voxel_chunk[pos as usize]
                    .material
                    .clone()
                    .expect("voxel resource is not set");
                (
                    TextureIden {
                        material,
                        direction: direction.clone(),
                    },
                    quad,
                    face,
                )
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

    let mut groups = HashMap::<TextureIden, MeshPrep>::new();
    for (iden, quad, face) in quads {
        let prep = groups.entry(iden).or_default();
        prep.indices
            .extend_from_slice(&face.quad_mesh_indices(prep.positions.len() as u32));
        prep.positions
            .extend_from_slice(&face.quad_mesh_positions(quad, 1.0 / resolution as f32));
        prep.normals.extend_from_slice(&face.quad_mesh_normals());
        //prep.uvs.extend_from_slice(&[[0.0; 2]; 4]);
        prep.uvs
            .extend_from_slice(&[[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]]);
    }

    let groups = groups
        .into_iter()
        .map(|(iden, prep)| {
            (
                iden,
                construct_mesh(prep.indices, prep.positions, prep.normals, prep.uvs, usage),
            )
        })
        .collect::<Vec<_>>();

    groups
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

pub fn quads_buffer_to_mesh(
    buffer: &GreedyQuadsBuffer,
    usage: RenderAssetUsages,
    resolution: usize,
    size: usize,
) -> Mesh {
    //https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs#L5
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    //let mut data = Vec::<u32>::with_capacity(num_vertices);

    for (group, face) in buffer
        .quads
        .groups
        .as_ref()
        .iter()
        .zip(COORDS_CONFIG.faces.iter())
    {
        for quad in group.iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0 / resolution as f32));

            normals.extend_from_slice(&face.quad_mesh_normals());
            //TODO (insert resource data)
            let pos = quad.minimum;
            let _pos = pos[0] + pos[1] * size as u32 + pos[2] * size as u32 * size as u32;

            //let texture_id = 0;
            //TODO get texture id for resource

            //data.extend_from_slice(&[texture_id; 4]);
        }
    }

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

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );
    mesh.insert_indices(Indices::U32(indices));
    //mesh.insert_attribute(MESH_TEXTURE_ATTRIBUTE, VertexAttributeValues::Uint32(data));

    mesh
}

fn unit_quads_to_greedy_quads(buffer: UnitQuadBuffer) -> GreedyQuadsBuffer {
    //let size = buffer.num_quads();
    let mut greedy = GreedyQuadsBuffer::new(0);
    greedy
        .quads
        .groups
        .par_iter_mut()
        .zip(buffer.groups.into_par_iter())
        .for_each(|(group, unit_group)| {
            unit_group
                .into_par_iter()
                .map(|unit_quad| UnorientedQuad::from(unit_quad))
                .collect_into_vec(group);
        });

    greedy
}

#[inline]
fn direction_from_oriented_block_face(oriented_block_face: &OrientedBlockFace) -> Direction {
    let normal = oriented_block_face.signed_normal();
    match [normal.x, normal.y, normal.z] {
        [0, 1, 0] => Direction::Up,
        [0, -1, 0] => Direction::Down,
        [1, 0, 0] => Direction::East,
        [-1, 0, 0] => Direction::West,
        [0, 0, 1] => Direction::South,
        [0, 0, -1] => Direction::North,
        _ => unreachable!("normal vector is not a unit vector"),
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use game2::humanize::humanize_memory;

    use crate::world::chunk::voxel::Voxel;

    #[test]
    fn voxel_size() {
        println!("Size of Voxel: {}", humanize_memory(size_of::<Voxel>()));
    }
}
