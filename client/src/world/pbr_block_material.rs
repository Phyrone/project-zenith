use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

#[derive(Debug, Asset, AsBindGroup, TypePath, Clone)]
pub struct BlockMaterial {
    #[uniform(100)]
    tile_size: u32,
}

impl Default for BlockMaterial {
    fn default() -> Self {
        Self { tile_size: 1024 }
    }
}

impl MaterialExtension for BlockMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/block_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/block_material.wgsl".into()
    }
}
