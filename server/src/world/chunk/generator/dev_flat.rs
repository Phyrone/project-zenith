use bevy::app::{App, Plugin};
use bevy::prelude::{Commands, Entity, Query, With};

use game2::CHUNK_SIZE;

use crate::world::block::BlockData;
use crate::world::chunk::block_data::ChunkBlockData;
use crate::world::chunk::{Chunk, ChunkGenerateErrand};
use crate::world::WorldSchedules::WorldTick;

#[derive(Default)]
pub struct DevFlatChunkGeneratorPlugin;

impl Plugin for DevFlatChunkGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(WorldTick, dev_flat_chunk_generator_system);
    }
}

fn dev_flat_chunk_generator_system(
    _commands: Commands,
    chunks_to_generate: Query<(Entity, &Chunk), With<ChunkGenerateErrand>>,
) {
    if chunks_to_generate.is_empty() {
        return;
    }

    for (_entity, chunk) in chunks_to_generate.iter() {
        let _block_data = if chunk.y <= 0 {
            let _block_data = BlockData::new(Some(Entity::from_raw(1)));

            ChunkBlockData {
                blocks: Box::new([[[BlockData::default(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
            }
        } else {
            //TODO generate flat chunk
            ChunkBlockData {
                blocks: Box::new([[[BlockData::default(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
            }
        };
    }
}
