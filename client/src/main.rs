use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::math::Vec3;
use bevy::prelude::{AmbientLight, Assets, Camera, Camera3d, Camera3dBundle, Color, Commands, Cuboid, default, DirectionalLight, DirectionalLightBundle, Mesh, PbrBundle, PluginGroup, ResMut, StandardMaterial, Transform};
use bevy::render::primitives::Frustum;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;
use error_stack::{Report, ResultExt};

use crate::startup::{ClientStartupError, init_logger, StartupParams};
use crate::world::ClientWorldPlugin;

mod startup;
pub mod world;

fn main() -> Result<(), Report<ClientStartupError>> {
    let params = StartupParams::parse();
    init_logger(params.log_level).change_context(ClientStartupError::LoggerInit)?;

    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 500.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientWorldPlugin)
        .add_systems(Startup, test)
        .add_systems(Startup, setup_test_world)
        .add_plugins(WorldInspectorPlugin::new())
        .run();

    Ok(())
}

fn test(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
) {
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            ..default()
        },
        camera: Camera {
            hdr: true,
            ..default()
        },
        frustum: Frustum { ..default() },
        transform: Transform::from_xyz(3.0, 10.0, 3.0),
        //.looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_to(Vec3::new(-0.2, -0.4, -0.2), Vec3::Y),
        ..default()
    });

    //ambient_light.color = Color::WHITE;
    //ambient_light.brightness = 0.75;
}

fn setup_test_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        ..default()
    });
    //spawn underground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(100.0, 0.01, 100.0)),
        material: materials.add(Color::rgb(0.3, 0.3, 0.5)),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });
}
