mod dev_flat;

use bevy::app::App;
use bevy::prelude::Plugin;

pub struct CoreGeneratorsPlugin;

impl Plugin for CoreGeneratorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(dev_flat::DevFlatChunkGeneratorPlugin);
    }
}
