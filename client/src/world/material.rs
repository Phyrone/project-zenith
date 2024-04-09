use std::any::Any;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::prelude::{Plugin, Resource};
use block_mesh::VoxelVisibility;
use futures_lite::StreamExt;
use itertools::Itertools;
use unstructured::{Document, Unstructured};

use game2::registry::Registry;

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
Default,
Clone,
PartialEq,
Eq,
Ord,
PartialOrd,
Hash,
serde::Serialize,
serde::Deserialize,
)]
pub struct MaterialData(Document);

impl Deref for MaterialData {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MaterialData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Document> for MaterialData {
    fn from(doc: Document) -> Self {
        Self(doc)
    }
}

impl From<&Document> for MaterialData {
    fn from(doc: &Document) -> Self {
        Self(doc.clone())
    }
}

impl MaterialData {
    pub const RENDER_KEY: &'static str = "render";
    pub const KEY_GEOMETRY: &'static str = format!("{}/geometry", Self::RENDER_KEY).as_str();
    pub const KEY_PHYSICS: &'static str = "physics";
    
    pub fn geometry(&self) -> Option<&Document> {
        self.select(Self::KEY_GEOMETRY).ok()
    }
    pub fn voxel_visibility(&self) -> Option<VoxelVisibility> {
        self.select("voxel_visibility")
            .map(|doc| Self::parse_voxel_visibility(doc))
            .flatten()
            .ok()
    }
    pub fn parse_voxel_visibility(data_unstructured: &Document) -> Option<VoxelVisibility> {
        match data_unstructured {
            Unstructured::String(data) => {
                match data.to_lowercase().as_str() {
                    "transparent" | "translucent" | "t"
                    => Some(VoxelVisibility::Translucent),
                    "solid" | "opaque" | "o"
                    => Some(VoxelVisibility::Opaque),
                    "empty" | "air" | "e"
                    => Some(VoxelVisibility::Empty),
                    _ => None
                }
            }
            _ => None,
        }
    }

    fn physics(&self) -> Option<&Document> {
        self.select(Self::KEY_PHYSICS).ok()
    }
}

#[cfg(test)]
pub mod test {}
