use bevy::prelude::Mesh;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;

use crate::GreedyQuad;

fn quad_to_mesh(
    quads: GreedyQuad,
    scale: f32,
    usage: RenderAssetUsages,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList,usage);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    


}
