use std::any::Any;
use std::hash::Hash;
use std::sync::Arc;

use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::prelude::{Plugin, Resource};
use game2::material::ResourceKey;
use game2::registry::Registry;
use hashbrown::HashMap;
use itertools::Itertools;
use slab::Slab;
use unstructured::Document;

#[derive(Debug, Default)]
pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialRegistry>();
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MaterialRegistryMarker;

pub type MaterialRegistry = Registry<Document, MaterialRegistryMarker>;

#[cfg(test)]
pub mod test {}
