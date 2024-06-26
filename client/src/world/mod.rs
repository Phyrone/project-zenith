use crate::world::camera::WorldCameraPlugin;
use crate::world::cubes::CubeRenderWorldPlugin;
use crate::world::environment::WorldEnvironmnentPlugin;
use bevy::app::App;
use bevy::prelude::*;

pub mod assets;
pub mod camera;
pub mod cubes;
pub mod environment;

#[derive(Default, Debug)]
pub struct ClientWorldPlugin;

impl Plugin for ClientWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CubeRenderWorldPlugin,
            WorldCameraPlugin,
            WorldEnvironmnentPlugin,
        ));
    }
}
