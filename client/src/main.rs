use bevy::app::{App, Startup};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::math::Vec3;
use bevy::prelude::{
    default, shape, AmbientLight, Assets, Camera, Camera3d, Camera3dBundle, Color, Commands,
    DirectionalLight, DirectionalLightBundle, Mesh, PbrBundle, PluginGroup, ResMut,
    StandardMaterial, Transform,
};
use bevy::render::primitives::Frustum;
use bevy::DefaultPlugins;
use bevy_framepace::Limiter;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;
use error_stack::{Report, ResultExt};

use crate::startup::{init_logger, ClientStartupError, StartupParams};
use crate::world::ClientWorldPlugin;

mod startup;
pub mod world;

fn main() -> Result<(), Report<ClientStartupError>> {
    let params = StartupParams::parse();
    init_logger(params.log_level).change_context(ClientStartupError::LoggerInit)?;

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClientWorldPlugin)
        .add_systems(Startup, test)
        .add_systems(Startup, setup_test_world)
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, set_frame_limit_system)
        .run();

    Ok(())
}

fn set_frame_limit_system(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = Limiter::Auto
}

fn test(mut commands: Commands, mut ambient_light: ResMut<AmbientLight>) {
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
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

    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 0.75;
}

fn setup_test_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube::new(0.5).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    //spawn underground
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Cube::new(0.5).into()),
        material: materials.add(Color::rgb(0.3, 0.3, 0.5).into()),
        transform: Transform::from_xyz(0.0, -0.5, 0.0).with_scale(Vec3::new(100.0, 0.01, 100.0)),
        ..default()
    });
}
