use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AddressMode, SamplerDescriptor};
use bevy::render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy_flycam::{FlyCam, PlayerPlugin};

use mesher::b16::{build_mesh16, VoxelCubeOcclusionMatrix16};
use mesher::meshing::quads_to_mesh;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin{
            default_sampler: ImageSamplerDescriptor{
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                address_mode_w: ImageAddressMode::Repeat,
                ..default()
            }
        }))
        .add_plugins(PlayerPlugin)
        //set lum
        .insert_resource(AmbientLight {
            brightness: 500.0,
            ..Default::default()
        })
        .add_systems(Startup, (setup, spawn_test_chunk))
        .add_systems(Update, display_coords)
        .run();
}


#[derive(Debug, Clone, Eq, PartialEq, Hash, Component)]
struct CoordsText;

fn setup(mut commands: Commands) {
    commands.spawn((
        FlyCam
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(2.0, 2.0, 2.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn((CoordsText, TextBundle::from_section("WAIT", TextStyle::default()).with_text_justify(JustifyText::Left)));
}

fn spawn_test_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load("textures/prototype/red/texture_01.png");
    let texture = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        base_color_texture: Some(texture),
        ..Default::default()
    });
    ImageAddressMode::Repeat;
    let mut matrix = VoxelCubeOcclusionMatrix16::new();
    matrix.set_voxel(0, 0, 0, true);
    matrix.set_voxel(0, 2, 0, true);
    matrix.set_voxel(0, 3, 0, true);
    matrix.set_voxel(1, 3, 0, true);
    matrix.set_voxel(0, 3, 1, true);
    let quads = build_mesh16(&matrix, |x, y, z, face| {
        Some(1)
    });
    let mesh = quads_to_mesh(quads.get(&1).unwrap(), 1.0, RenderAssetUsages::RENDER_WORLD);
    let mesh = meshes.add(mesh);
    commands.spawn(PbrBundle {
        mesh: mesh.clone(),
        material: texture.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}


fn display_coords(
    mut camera_query: Query<(&Transform), With<FlyCam>>,
    mut text_query: Query<&mut Text, With<CoordsText>>,
) {
    if let (camera_transform) = camera_query.single_mut() {
        let coords = camera_transform.translation;
        if let mut text = text_query.single_mut() {
            text.sections[0].value = format!("x: {:.2} y: {:.2} z: {:.2}", coords.x, coords.y, coords.z);
        }
    }
}