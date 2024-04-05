use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::pbr::ExtendedMaterial;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::primitives::{Aabb, Frustum};
use bevy::utils::info;
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;
use error_stack::{Report, ResultExt};

use client::world::pbr_block_material::BlockMaterial;
use game2::CHUNK_SIZE;

use crate::discord::DiscordRPCPlugin;
use crate::startup::{ClientStartupError, init_logger, StartupParams};
use crate::world::chunk::chunk_data::{ChunkDataEntry, ChunkDataStorage, ClientChunkData};
use crate::world::chunk::chunk_render_mesh::ChunkRenderErrand;
use crate::world::chunk::VoxelWorldFixedChunkPosition;
use crate::world::ClientWorldPlugin;
use crate::world::material::{MaterialRegistry};
use bevy::DefaultPlugins;
use bevy::render::render_resource::ShaderRef::Handle;

pub mod discord;
mod startup;
pub mod ui;
pub mod world;

fn main() -> Result<(), Report<ClientStartupError>> {
    let params = StartupParams::parse();
    init_logger(params.log_level).change_context(ClientStartupError::LoggerInit)?;

    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
        })
        .add_plugins(DefaultPlugins)
        //.add_plugins(DiscordRPCPlugin)
        .add_plugins(ClientWorldPlugin)
        .add_systems(Startup, test)
        .add_systems(Startup, (test_textures, spawn_debug_ui))
        .add_systems(
            Update,
            (toggle_wireframe_btn, rerender_chunks, update_debug_cords),
        )
        //.add_systems(Startup, setup_test_world)
        .add_plugins((WorldInspectorPlugin::new(), WireframePlugin))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::RED,
        })
        .run();

    Ok(())
}

fn toggle_wireframe_btn(
    mut wireframe_config: ResMut<WireframeConfig>,
    mut keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        wireframe_config.global = !wireframe_config.global;
    }
}

#[derive(Debug, Default, Component)]
struct FpsComponentMarker;

#[derive(Debug, Default, Component)]
struct CordsComponentMarker;

fn spawn_debug_ui(mut commands: Commands, camera: Query<&GlobalTransform, With<Camera3d>>) {
    let root_node = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section("FPS HERE", TextStyle { ..default() }),

                    ..default()
                },
                FpsComponentMarker,
            ));
            parent.spawn((
                TextBundle {
                    text: Text::from_section("CORDS Here", TextStyle { ..default() }),
                    ..default()
                },
                CordsComponentMarker,
            ));
        });
}

fn update_debug_cords(
    camera: Query<&GlobalTransform, With<Camera3d>>,
    mut text: Query<&mut Text, With<CordsComponentMarker>>,
) {
    let (camera) = camera.get_single().expect("No camera found");
    for mut text in text.iter_mut() {
        *text = Text::from_section(
            format!(
                "Camera cords: x: {:.2} y: {:.2} z: {:.2}",
                camera.translation().x,
                camera.translation().y,
                camera.translation().z
            ),
            TextStyle { ..default() },
        );
    }
}

fn rerender_chunks(
    mut keys: Res<ButtonInput<KeyCode>>,
    commands: ParallelCommands,
    query: Query<
        (Entity),
        (
            With<VoxelWorldFixedChunkPosition>,
            With<ClientChunkData>,
            Without<ChunkRenderErrand>,
        ),
    >,
) {
    if keys.just_pressed(KeyCode::KeyO) {
        //rerender all chunks
        query.par_iter().for_each(|(entity)| {
            commands.command_scope(|mut commands| {
                commands.entity(entity).insert(ChunkRenderErrand);
            })
        });
    }
}

fn test_textures(
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, BlockMaterial>>>,
    assets: Res<AssetServer>,
    mut material_registry: ResMut<MaterialRegistry>,
) {
    let stone_texture = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::hex("947d75").unwrap(),
            base_color_texture: Some(assets.load("textures/prototype/dark/texture_01.png")),
            unlit: true,
            ..default()
        },
        extension: BlockMaterial,
    });
    let dirt_texture = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::hex("553d31").unwrap(),
            base_color_texture: Some(assets.load("textures/prototype/orange/texture_01.png")),
            ..default()
        },
        extension: BlockMaterial,
    });

    let grass_texture = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::hex("7b824e").unwrap(),
            base_color_texture: Some(assets.load("textures/prototype/green/texture_01.png")),
            //base_color_texture: Some(assets.load("materials/grass/Color.png")),
            //occlusion_texture: Some(assets.load("materials/grass/AmbientOcclusion.png")),
            //normal_map_texture: Some(assets.load("materials/grass/NormalDX.png")),
            //metallic_roughness_texture: Some(assets.load("materials/grass/Roughness.png")),
            perceptual_roughness: 0.9,
            ..default()
        },
        extension: BlockMaterial,
    });

    material_registry
        .new_bundle()
        .add_component(BlockMaterialDescription::MONO(stone_texture));
    material_registry
        .new_bundle()
        .add_component(BlockMaterialDescription::MONO(dirt_texture.clone()));
    material_registry
        .new_bundle()
        .add_component(BlockMaterialDescription::TSB([
            grass_texture,
            dirt_texture.clone(),
            dirt_texture.clone(),
        ]))
}

fn test(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d { ..default() },
            camera: Camera {
                hdr: false,
                ..default()
            },
            transform: Transform::from_xyz(3.0, 10.0, 3.0),
            //.looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        AtmosphereCamera::default(),
    ));

    commands.spawn(
        (DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                illuminance: 10_000.0,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .looking_to(Vec3::new(-0.2, -0.4, -0.2), Vec3::Y),
            ..default()
        }),
    );

    let mut test_chunk = ChunkDataStorage::empty();
    test_chunk.set_many(0..32 * 32 * 14, ChunkDataEntry::Block(3, None));
    test_chunk.set_many(32 * 32 * 14..32 * 32 * 18, ChunkDataEntry::Block(1, None));
    test_chunk.set_many(32 * 32 * 18..32 * 32 * 32, ChunkDataEntry::Block(3, None));

    let test_chunk = ClientChunkData::new(test_chunk);
    info!("test chunk data fabricated");
    for y in -6..-1 {
        for x in -8..8 {
            for z in -8..8 {
                let chunk = test_chunk.clone();
                commands.spawn((
                    chunk,
                    VoxelWorldFixedChunkPosition { x, y, z },
                    ChunkRenderErrand,
                    ViewVisibility::default(),
                    Aabb::from_min_max(Vec3::splat(-1.0), Vec3::splat(1.0 + CHUNK_SIZE as f32)),
                    Visibility::Visible,
                    InheritedVisibility::default(),
                    GlobalTransform::default(),
                ));
            }
        }
    }
    info!("test chunks placed")

    //ambient_light.color = Color::WHITE;
    //ambient_light.brightness = 0.75;
}

fn setup_test_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });

    let texture = asset_server.load("textures/prototype/dark/texture_01.png");
    let material = materials.add(StandardMaterial {
        base_color: Color::hex("8B4513").unwrap(),
        base_color_texture: Some(texture),
        metallic: 1.0,
        depth_bias: 0.0,
        ior: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });
    let size = 1;
    let plate_size = 3.0;
    let mut bundles = vec![
        PbrBundle {
            mesh: meshes.add(Cuboid::new(plate_size, 0.0, plate_size)),
            material,
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        };
        size * size
    ];

    for (i, bundle) in bundles.iter_mut().enumerate() {
        let x = (i % size) as i32 - (size / 2) as i32;
        let z = (i / size) as i32 - (size / 2) as i32;
        bundle.transform = Transform::from_xyz(x as f32 * plate_size, -0.5, z as f32 * plate_size);
    }
    commands.spawn_batch(bundles);

    //spawn underground
    /*
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(10.0, 0.01, 10.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("8B4513").unwrap(),
            base_color_texture: Some(asset_server.load("textures/prototype/dark/texture_01.png")),
            metallic: 1.0,
            depth_bias: 0.0,
            ior: 0.3,
            perceptual_roughness: 0.6,

            ..default()
        }),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

     */
}
