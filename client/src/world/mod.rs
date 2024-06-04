use bevy::app::App;
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::VertexFormat;

use crate::world::camera::WorldCameraPlugin;
use crate::world::chunk::ClientWorldChunksPlugin;
use crate::world::environment::WorldEnvironmnentPlugin;

pub mod assets;
pub mod camera;
pub mod chunk;
pub mod environment;
pub mod pbr_block_material;
//pub mod texture;

#[derive(Default, Debug)]
pub struct ClientWorldPlugin;

impl Plugin for ClientWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientWorldChunksPlugin,
            WorldCameraPlugin,
            WorldEnvironmnentPlugin,
        ));
    }
}

pub const MESH_TEXTURE_ATTRIBUTE: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Texture", 0x696969, VertexFormat::Uint32);
