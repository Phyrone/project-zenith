use bevy::app::App;
use bevy::prelude::Plugin;
use bevy_atmosphere::prelude::*;

#[derive(Debug, Default)]
pub struct WorldEnvironmnentPlugin;

impl Plugin for WorldEnvironmnentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AtmospherePlugin);
    }
}
