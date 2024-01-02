use bevy::app::App;
use bevy::prelude::{IntoSystemConfigs, Plugin, SystemSet};

use crate::world::WorldSchedules::WorldTick;

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, SystemSet)]
pub struct WorldAttention {}

#[derive(Debug, Default)]
pub struct WorldAttentionPlugin;

impl Plugin for WorldAttentionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            WorldTick,
            (update_attention_system).in_set(WorldAttention::default()),
        );
    }
}

fn update_attention_system() {}
