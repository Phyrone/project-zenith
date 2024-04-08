use std::any::Any;
use std::hash::Hash;

use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::prelude::{Plugin, Resource};
use itertools::Itertools;
use unstructured::Document;

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

pub type MaterialRegistry = Registry<Document, MaterialRegistryMarker>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub struct MaterialData(Document);

impl MaterialData{
    
    pub fn 
    
}


#[cfg(test)]
pub mod test {}
