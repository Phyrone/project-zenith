use bevy::app::App;
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;

use crate::world::camera::WorldCameraPlugin;
use crate::world::chunk::ClientWorldChunksPlugin;

pub mod block_data;
pub mod camera;
pub mod chunk;
pub mod material;

#[derive(Default, Debug)]
pub struct ClientWorldPlugin;

impl Plugin for ClientWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ClientWorldChunksPlugin, WorldCameraPlugin));
    }
}

pub const MESH_TEXTURE_ATTRIBUTE: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Texture", 0x696969, VertexFormat::Uint32);
