use bevy::app::{App, Startup};
use bevy::prelude::{Commands, Component, Plugin};

#[derive(Debug, Component, Hash, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct MaterialData {
    pub namespace: String,
    pub name: String,
}

impl MaterialData {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CoreMaterialPlugins;

impl Plugin for CoreMaterialPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_default_materials);
    }
}

fn apply_default_materials(mut commands: Commands) {
    commands.spawn(MaterialData::new("core", "grass"));
    commands.spawn(MaterialData::new("core", "dirt"));
    commands.spawn(MaterialData::new("core", "stone"));
}
