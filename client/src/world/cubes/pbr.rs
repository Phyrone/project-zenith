use std::ops::Deref;

use bevy::prelude::*;
use hashbrown::HashMap;
use uuid::Uuid;

use crate::world::cubes::ChunkRenderStage;

pub struct ChunkPbrPlugin;

impl Plugin for ChunkPbrPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialMapper>();
        app.add_systems(Startup, test_textures);
        app.add_systems(
            Update,
            apply_surfaces.in_set(ChunkRenderStage::ApplyMaterial),
        );
    }
}

#[derive(Debug, Clone, Default, Resource)]
pub struct MaterialMapper {
    pub materials: HashMap<Uuid, Handle<StandardMaterial>>,
}

impl MaterialMapper {
    pub fn clear(&mut self) {
        self.materials.clear();
    }
}

const TEST_TEXTURES: [(&str, Uuid); 2] = [
    (
        "textures/grass.webp",
        Uuid::from_u128(0x00000000000000000000000000000001),
    ),
    (
        "textures/dirt.webp",
        Uuid::from_u128(0x00000000000000000000000000000002),
    ),
];

fn test_textures(
    texture_loader: Res<AssetServer>,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut material_mapper: ResMut<MaterialMapper>,
) {
    for (path, uuid) in TEST_TEXTURES.into_iter() {
        let texture = texture_loader.load(path);
        let texture = assets.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(texture),
            metallic: 0.0,
            reflectance: 0.05,
            ..Default::default()
        });
        material_mapper.materials.insert(uuid, texture);
    }
}

fn apply_surfaces(
    commands: ParallelCommands,
    materials: Res<MaterialMapper>,
    surfaces: Query<(Entity, &SurfaceMaterial), Added<SurfaceMaterial>>,
) {
    surfaces.par_iter().for_each(|(entity, surface)| {
        let material = materials.materials.get(surface.deref()).cloned();
        if let Some(material) = material {
            commands.command_scope(|mut commands| {
                commands.entity(entity).insert(material);
            })
        }
    });
}
