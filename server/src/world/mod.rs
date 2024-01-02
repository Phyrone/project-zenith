
use std::hash::Hash;
use std::time::Duration;

use bevy::app::{App, PluginGroupBuilder, ScheduleRunnerPlugin};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::{
    Component, FrameCountPlugin, Plugin, PluginGroup, TaskPoolPlugin,
    TypeRegistrationPlugin,
};
use bevy::time::TimePlugin;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use game2::DELAY_BETWEEN_TICKS;

use crate::world::attention::WorldAttentionPlugin;
use crate::world::material::CoreMaterialPlugins;

mod attention;
mod block;
mod chunk;
mod material;

pub struct WorldPlugin {
    server: bool,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        if self.server {
            app.add_plugins((
                WorldServerPlugins::default(),
                WorldAttentionPlugin,
                CoreMaterialPlugins,
            ));
        }
    }
}

#[derive(Default, Debug)]
struct WorldServerPlugins {}

impl PluginGroup for WorldServerPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(TaskPoolPlugin::default())
            .add(TypeRegistrationPlugin)
            .add(FrameCountPlugin)
            .add(TimePlugin)
            .add(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                DELAY_BETWEEN_TICKS,
            )))
    }
    fn name() -> String {
        "world_base".to_string()
    }
}

#[derive(
    Debug, Component, Default, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
struct AbsoluteWorldPosition {
    x: i128,
    y: i128,
    z: i128,
}

//world block offset is an addon to world position that allows for sub block precision
//it is not required for blocks or chunks etc.
//allowed values are 0.0 to 1.0 inclusive
#[derive(Debug, Component, Default, Clone, Serialize, Deserialize)]
struct WorldBlockOffset {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(
    Debug, Default, Component, Clone, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct Dimension {
    pub identifier: Uuid,
}

#[derive(Debug, Default, Component)]
struct ChunkStorageContainer {
    //storage: Box<dyn ChunkStorage + Send>
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Ord, PartialOrd, PartialEq, Eq)]
struct PersistentChunkData {
    block_data: chunk::block_data::ChunkBlockData,
}

#[async_trait::async_trait]
pub trait ChunkRepository {
    //TODO error handling
    async fn store_chunk(&self, dimension: &Uuid, chunk: &chunk::Chunk, data: &PersistentChunkData);

    //TODO error handling
    async fn load_chunk(
        &self,
        dimension: &Uuid,
        chunk: &chunk::Chunk,
    ) -> Option<PersistentChunkData>;

    //TODO error handling
    async fn remove_chunk(&self, dimension: &Uuid, chunk: &chunk::Chunk) -> bool;
}

#[derive(Debug, Default)]
struct MemoryChunkStorage {
    map: RwLock<std::collections::HashMap<(Uuid, chunk::Chunk), (Vec<u8>, usize)>>,
}

#[async_trait::async_trait]
impl ChunkRepository for MemoryChunkStorage {
    async fn store_chunk(
        &self,
        dimension: &Uuid,
        chunk: &chunk::Chunk,
        data: &PersistentChunkData,
    ) {
        let binary = bincode::serialize(data).expect("failed to serialize chunk data");
        let capacity = binary.len();
        let mut compressed = zstd::bulk::compress(&binary, 22).unwrap();
        compressed.shrink_to_fit();
        self.map
            .write()
            .await
            .insert((*dimension, *chunk), (compressed, capacity));
    }

    async fn load_chunk(
        &self,
        dimension: &Uuid,
        chunk: &chunk::Chunk,
    ) -> Option<PersistentChunkData> {
        let read_lock_map = self.map.read().await;
        let (binary, capacity) = read_lock_map.get(&(*dimension, *chunk))?;
        let decompressed =
            zstd::bulk::decompress(binary, capacity + 1).expect("failed to decompress chunk data");
        let data = bincode::deserialize::<PersistentChunkData>(&decompressed)
            .expect("failed to deserialize chunk data");
        Some(data)
    }

    async fn remove_chunk(&self, dimension: &Uuid, chunk: &chunk::Chunk) -> bool {
        self.map
            .write()
            .await
            .remove(&(*dimension, *chunk))
            .is_some()
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, ScheduleLabel)]
pub enum WorldSchedules {
    #[default]
    WorldTick,
}
