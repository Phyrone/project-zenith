use std::any::Any;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::prelude::{Plugin, Resource};
use block_mesh::VoxelVisibility;
use futures_lite::StreamExt;
use itertools::Itertools;
use rclite::Arc;
use unstructured::{Document, Unstructured};

use game2::registry::Registry;
use crate::world::chunk::voxel::VoxelMaterialDescription;

#[derive(Debug, Default)]
pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialRegistry>();
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MaterialRegistryMarker;

pub const AIR_MATERIAL_ID: usize = 0;

pub type MaterialRegistry = Registry<MaterialData, MaterialRegistryMarker>;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash
)]
pub struct MaterialData{
    metadata: Document,
    voxel: Option<std::sync::Arc<dyn VoxelMaterialDescription>>
}

impl MaterialData {
    pub const RENDER_KEY: &'static str = "render";
    pub const KEY_GEOMETRY: &'static str = "render/geometry";
    pub const KEY_PHYSICS: &'static str = "physics";

    pub fn geometry(&self) -> Option<&Document> {
        self.select(Self::KEY_GEOMETRY).ok()
    }
    pub fn voxel_visibility(&self) -> Option<VoxelVisibility> {
        self.select(format!("{}/{}/voxel_visibility",Self::KEY_GEOMETRY, Self::RENDER_KEY).as_str())
            .ok()
            .map(|doc| Self::parse_voxel_visibility(doc))
            .flatten()
    }


}

#[cfg(test)]
pub mod test {}
