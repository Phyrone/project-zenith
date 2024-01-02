pub mod lod;

use bevy::app::App;
use bevy::prelude::{Plugin, States};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, States)]
enum WorldCameraConfiguration {
    #[default]
    None,
    FirstPersonCamera,
    ThirdPersonCamera,
    //for debugging and testing only (excluded from release builds)
    #[cfg(debug_assertions)]
    FreeCamera,
}

#[derive(Debug, Default)]
pub struct WorldCameraPlugin;

impl Plugin for WorldCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<WorldCameraConfiguration>();
    }
}
