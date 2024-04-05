use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

#[derive(Debug, Default, Asset, AsBindGroup,TypePath,Clone)]
pub struct BlockMaterial{};

impl MaterialExtension for BlockMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/block_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/block_material.wgsl".into()
    }

}
