use crate::b16::{build_mesh16, VoxelCubeOcclusionMatrix16};
use crate::b64::build_mesh64;
use bevy::prelude::shape::Quad;
use bevy::prelude::{Mesh, Vec3};
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{ImageAddressMode, ImageSamplerDescriptor};

use crate::{FaceDirection, GreedyQuad};

pub fn quads_to_mesh(quads: &[GreedyQuad], scale: f32, usage: RenderAssetUsages) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage);
    let mut positions = Vec::with_capacity(quads.len() * 4);
    let mut normals = Vec::with_capacity(quads.len() * 4);
    let mut indices = Vec::with_capacity(quads.len() * 6);
    let mut uvs = Vec::with_capacity(quads.len() * 4);
    for quad in quads {
        add_mesh_data(
            quad,
            scale,
            &mut positions,
            &mut normals,
            &mut indices,
            &mut uvs,
        );
    }
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    mesh.insert_indices(Indices::U32(indices));

    mesh
}

fn add_mesh_data(
    quad: &GreedyQuad,
    scale: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    uvs: &mut Vec<[f32; 2]>,
) {

    let i = positions.len() as u32;
    indices.extend(&[i, i + 1, i + 2, i + 2, i + 1, i + 3]);
    let normal = match quad.direction {
        FaceDirection::ZPos => [0.0, 0.0, 1.0],
        FaceDirection::ZNeg => [0.0, 0.0, -1.0],
        FaceDirection::YPos => [0.0, 1.0, 0.0],
        FaceDirection::YNeg => [0.0, -1.0, 0.0],
        FaceDirection::XPos => [1.0, 0.0, 0.0],
        FaceDirection::XNeg => [-1.0, 0.0, 0.0],
    };
    normals.extend(&[normal, normal, normal, normal]);
    //first vertex position
    positions.extend(quad.vertex_positions(scale));
    uvs.extend(&[[quad.w as f32, quad.h as f32], [quad.w as f32,0.0],[0.0,quad.h as f32], [0.0, 0.0]]);
    

}

#[test]
fn test() {
    let mut matrix = VoxelCubeOcclusionMatrix16::new();
    matrix.set_voxel(1, 1, 3, true);
    matrix.set_voxel(1, 2, 3, true);
    let quads = build_mesh16(&matrix, |x, y, z, face| {
        //println!("x:{} y:{} z:{} face:{}", x, y, z, face);
        if x == 1 && (1..=2).contains(&y) && z == 3 {
            Some(1)
        } else {
            None
        }
    });
    println!("{:?}", quads);
}
