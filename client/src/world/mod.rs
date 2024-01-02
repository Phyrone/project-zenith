use bevy::app::App;
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;

use crate::world::camera::WorldCameraPlugin;
use crate::world::chunk::ClientWorldChunksPlugin;

mod block_data;
mod camera;
mod chunk;

#[derive(Default, Debug)]
pub struct ClientWorldPlugin;

impl Plugin for ClientWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ClientWorldChunksPlugin, WorldCameraPlugin));
    }
}

pub const MESH_TEXTURE_ATTRIBUTE: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Texture", 0x696969, VertexFormat::Uint32);
