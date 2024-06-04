use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct LODPlugin;

impl Plugin for LODPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, SystemSet)]
pub struct LodLevelStage;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
#[derive(Component)]
//TODO rename levels to be a bit more descriptive (low priority, low effort when refactoring tools work)
pub enum LODLevel {
    #[default]
    FULL = 255,
    Far1 = 254,
    Far2 = 253,

    Fartest = 1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Resource)]
pub struct LodConfig {
    pub full_lod_distance: f32,
    pub far1_lod_distance: f32,
    pub far2_lod_distance: f32,
    pub buffer_zone: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        //TODO more reasonable defaults
        LodConfig {
            full_lod_distance: 100.0,
            far1_lod_distance: 200.0,
            far2_lod_distance: 400.0,
            buffer_zone: 50.0,
        }
    }
}
