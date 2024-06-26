#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{ImageAddressMode, ImageSamplerDescriptor};
use bevy::DefaultPlugins;
use bevy_atmosphere::prelude::{AtmosphereCamera, AtmospherePlugin, AtmosphereSettings};
use bevy_flycam::{FlyCam, PlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use client::world::cubes::CubeRenderWorldPlugin;
use mesher::b32::{build_mesh32, VoxelCubeOcclusionMatrix32};
use mesher::meshing::quads_to_mesh;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Component)]
pub struct ClientGameState {}

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    address_mode_w: ImageAddressMode::Repeat,
                    ..default()
                },
            }),
            WireframePlugin,
        ))
        .add_plugins(AtmospherePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(CubeRenderWorldPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Color::WHITE,
        })
        .insert_resource(AtmosphereSettings {
            resolution: 2048,
            ..Default::default()
        })
        //set lum
        .insert_resource(AmbientLight {
            brightness: 500.0,
            ..Default::default()
        })
        .add_systems(Startup, (setup, spawn_test_chunk))
        .add_systems(Update, (display_coords, attach_atmosphere))
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Component)]
struct CoordsText;

fn setup(mut commands: Commands) {
    commands.spawn((FlyCam));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn((
        CoordsText,
        TextBundle::from_section("", TextStyle::default()).with_text_justify(JustifyText::Left),
    ));
}

fn attach_atmosphere(
    cameras: Query<Entity, (Without<AtmosphereCamera>, With<Camera3d>)>,
    mut commands: Commands,
) {
    for camera in cameras.iter() {
        commands.entity(camera).insert(AtmosphereCamera::default());
    }
}

fn spawn_test_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load("textures/grass.webp");
    let texture = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        metallic: 0.0,
        reflectance: 0.05,
        ..Default::default()
    });

    for x in -32..32 {
        for z in -32..32 {
            let mut matrix = VoxelCubeOcclusionMatrix32::new();
            for x in 0..32 {
                for y in 0..10 {
                    for z in 0..32 {
                        matrix.set_voxel(x, y, z, true);
                    }
                }
            }
            let quads = build_mesh32(&matrix, |x, y, z, face| Some(1));
            let mesh = quads_to_mesh(quads.get(&1).unwrap(), 1.0, RenderAssetUsages::RENDER_WORLD);
            let mesh = meshes.add(mesh);
            commands.spawn(PbrBundle {
                mesh,
                material: texture.clone(),
                transform: Transform::from_xyz(32.0 * x as f32, 0.0, 32.0 * z as f32),
                ..Default::default()
            });
        }
    }
}

fn textureize() {}

fn display_coords(
    mut camera_query: Query<(&Transform), With<FlyCam>>,
    mut text_query: Query<&mut Text, With<CoordsText>>,
) {
    if let (camera_transform) = camera_query.single_mut() {
        let coords = camera_transform.translation;
        if let mut text = text_query.single_mut() {
            text.sections[0].value =
                format!("x: {:.2} y: {:.2} z: {:.2}", coords.x, coords.y, coords.z);
        }
    }
}
