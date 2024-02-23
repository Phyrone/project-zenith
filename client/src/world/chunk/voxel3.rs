use bevy::prelude::{Mesh, Query};
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use block_mesh::ndshape::RuntimeShape;
use block_mesh::{
    GreedyQuadsBuffer, QuadCoordinateConfig, UnitQuadBuffer, VoxelVisibility,
    RIGHT_HANDED_Y_UP_CONFIG,
};
use rayon::prelude::*;

use game2::CHUNK_SIZE;

use crate::world::chunk::chunk_data::{ChunkDataEntry, ChunkDataStorage};
use crate::world::material::BlockMaterial;
use crate::world::MESH_TEXTURE_ATTRIBUTE;

const COORDS_CONFIG: &QuadCoordinateConfig = &RIGHT_HANDED_Y_UP_CONFIG;

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct Voxel {
    data: Option<VoxelData>,
}

impl Voxel {
    pub fn empty() -> Self {
        Self { data: None }
    }
    pub fn of(id: u32, material: &BlockMaterial) -> Voxel {
        Voxel {
            data: Some(VoxelData {
                id,
                material: material.clone(),
            }),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct VoxelData {
    id: u32,
    material: BlockMaterial,
}

impl block_mesh::Voxel for Voxel {
    fn get_visibility(&self) -> VoxelVisibility {
        match &self.data {
            None => VoxelVisibility::Empty,
            Some(data) => {
                if data.material.transparent {
                    VoxelVisibility::Translucent
                } else {
                    VoxelVisibility::Opaque
                }
            }
        }
    }
}

impl<'render> block_mesh::MergeVoxel for Voxel {
    type MergeValue = Voxel;

    fn merge_value(&self) -> Self::MergeValue {
        self.clone()
    }
}

impl ChunkDataEntry {
    fn create_voxel(
        &self,
        //TODO resolution and index could be used for more complex entries
        _resolution: usize,
        _index: usize,
        materials: &Query<(&BlockMaterial)>,
    ) -> Voxel {
        match self {
            ChunkDataEntry::Empty => Voxel::empty(),
            ChunkDataEntry::Block(entity, _) => {
                let material = materials.get(*entity).ok();
                match material {
                    None => Voxel::empty(),
                    Some(material) => Voxel::of(entity.index(), material),
                }
            }
        }
    }
}

pub fn create_voxel_chunk(
    data: &ChunkDataStorage,
    neighbors: &[Option<&ChunkDataStorage>; 6],
    materials: &Query<(&BlockMaterial)>,
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
            0 => get_voxel_inner(
                x,
                y,
                z,
                resolution,
                voxel_chunk_size,
                data,
                materials,
                voxel,
            ),
            //face
            1 => get_voxel_face(
                x,
                y,
                z,
                resolution,
                voxel_chunk_size,
                neighbors,
                materials,
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
    chunk: &ChunkDataStorage,
    materials: &Query<(&BlockMaterial)>,
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
    *into = inner_entry.create_voxel(resolution, sub_index, materials);
}

#[allow(clippy::too_many_arguments)]
fn get_voxel_face(
    x: usize,
    y: usize,
    z: usize,
    resolution: usize,
    voxel_chunk_size: usize,
    neighbours: &[Option<&ChunkDataStorage>; 6],
    materials: &Query<(&BlockMaterial)>,
    into: &mut Voxel,
) {
    let face = if x == 0 {
        0
    } else if y == 0 {
        1
    } else if z == 0 {
        2
    } else if x == voxel_chunk_size - 1 {
        3
    } else if y == voxel_chunk_size - 1 {
        4
    } else if z == voxel_chunk_size - 1 {
        5
    } else {
        unreachable!()
    };
    let neighbour = neighbours[face];
    let neighbour = match neighbour {
        None => return,
        Some(neighbour) => neighbour,
    };
    let entry_index = match face {
        0 => {
            (voxel_chunk_size - 2) * (voxel_chunk_size - 2) * (z - 1)
                + (y - 1) * (voxel_chunk_size - 2)
        }
        1 => (voxel_chunk_size - 2) * (voxel_chunk_size - 2) * (z - 1) + (x - 1),
        2 => (voxel_chunk_size - 2) * (y - 1) + (x - 1),
        3 => {
            (voxel_chunk_size - 2) * (voxel_chunk_size - 2) * (z - 1)
                + (y - 1) * (voxel_chunk_size - 2)
        }
        4 => (voxel_chunk_size - 2) * (voxel_chunk_size - 2) * (z - 1) + (x - 1),
        5 => (voxel_chunk_size - 2) * (y - 1) + (x - 1),
        _ => unreachable!(),
    };
    let entry = neighbour.get(entry_index);
    let sub_index = match face {
        0 => (y - 1) * (voxel_chunk_size - 2) + (z - 1),
        1 => (x - 1) * (voxel_chunk_size - 2) + (z - 1),
        2 => (x - 1) * (voxel_chunk_size - 2) + (y - 1),
        3 => (y - 1) * (voxel_chunk_size - 2) + (z - 1),
        4 => (x - 1) * (voxel_chunk_size - 2) + (z - 1),
        5 => (x - 1) * (voxel_chunk_size - 2) + (y - 1),
        _ => unreachable!(),
    };
    *into = entry.create_voxel(resolution, sub_index, materials);
}

pub fn voxels_quads(voxels: &[Voxel], resolution: usize) -> UnitQuadBuffer {
    let size = resolution * CHUNK_SIZE;
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
pub fn voxels_mesh(voxels: &[Voxel], resolution: usize) -> Mesh {
    let buffer = voxels_quads(voxels, resolution);
    let size = resolution * CHUNK_SIZE + 2;

    //https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs#L5
    let num_indices = buffer.num_quads() * 6;
    let num_vertices = buffer.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut data = Vec::<u32>::with_capacity(num_vertices);

    for (group, face) in buffer
        .groups
        .as_ref()
        .iter()
        .zip(COORDS_CONFIG.faces.iter())
    {
        for quad in group.iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            let quad = block_mesh::UnorientedQuad {
                minimum: quad.minimum,
                width: 1,
                height: 1,
            };

            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0 / resolution as f32));
            normals.extend_from_slice(&face.quad_mesh_normals());

            //TODO (insert material data)
            let pos = quad.minimum;
            let pos = pos[0] + pos[1] * size as u32 + pos[2] * size as u32 * size as u32;

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

pub fn voxels_geedy_mesh(voxels: &[Voxel], resolution: usize) -> Mesh {
    let buffer = voxels_geedy_quads(voxels, resolution);
    let size = resolution * CHUNK_SIZE + 2;
    //https://github.com/bonsairobo/block-mesh-rs/blob/main/examples-crate/render/main.rs#L5
    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut data = Vec::<u32>::with_capacity(num_vertices);

    for (group, face) in buffer
        .quads
        .groups
        .as_ref()
        .iter()
        .zip(COORDS_CONFIG.faces.iter())
    {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0 / resolution as f32));

            normals.extend_from_slice(&face.quad_mesh_normals());
            //TODO (insert material data)
            let pos = quad.minimum;
            let pos = pos[0] + pos[1] * size as u32 + pos[2] * size as u32 * size as u32;

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

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use game2::humanize::humanize_memory;

    use crate::world::chunk::voxel3::Voxel;

    #[test]
    fn voxel_size() {
        let size = size_of::<Voxel>();
        println!("Size of Voxel: {}", humanize_memory(size));
    }
}
