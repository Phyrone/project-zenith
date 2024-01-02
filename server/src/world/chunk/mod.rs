use bevy::prelude::{App, Component, DetectChangesMut, IntoSystemConfigs, Plugin, Resource};

use crate::world::chunk::grid::ChunkGridPlugin;

pub mod block_data;
mod generator;
pub mod grid;

#[derive(Debug, Default)]
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkGridPlugin);
    }
}

#[derive(Debug, Default, Component, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct Chunk {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

///informs the chunk storage to load a chunk from storage into memory
#[derive(Debug, Default, Component, Hash)]
pub struct ChunkLoadErrand;

///chunk storage places generate errands on chunks that need to be generated as they are not in storage yet
#[derive(Debug, Default, Component, Hash)]
pub struct ChunkGenerateErrand;

///informs the chunk storage to store a chunk from memory into storage
/// all store errads need to be removed before the server/world can shut down
#[derive(Debug, Default, Component, Hash)]
pub struct ChunkStoreErrand;

// does not need to be applied
#[derive(Debug, Default, Component, Hash)]
pub struct ChunkUnloadErrand;
