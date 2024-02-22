use bevy::app::{App, Startup};
use bevy::prelude::{Commands, Component, Plugin};

#[derive(Debug, Default)]
pub struct CoreMaterialPlugins;

impl Plugin for CoreMaterialPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_default_materials);
    }
}

fn apply_default_materials(mut commands: Commands) {

}

pub struct MaterialRegistry{

}