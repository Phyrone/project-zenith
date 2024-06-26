use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::render::texture::{ImageAddressMode, ImageSamplerDescriptor};
use bevy::winit::WinitSettings;

pub mod discord;
mod startup;
pub mod ui;
pub mod world;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum ClientMainState {
    //main menu, settings
    Lobby,
    GameConfiguring,
    GamePlaying,
}

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    address_mode_w: ImageAddressMode::Repeat,
                    ..default()
                },
            }),
            WireframePlugin,
        ));
        app.insert_resource(WinitSettings::game());
    }
}
