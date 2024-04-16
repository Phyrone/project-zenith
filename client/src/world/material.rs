use std::any::Any;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::prelude::{Plugin, Resource};
use futures_lite::StreamExt;
use itertools::Itertools;
use unstructured::Document;

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

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MaterialData {
    pub metadata: Document,
    pub voxel: Option<std::sync::Arc<VoxelMaterialDescription>>,
}

impl MaterialData {}

#[cfg(test)]
pub mod test {}
