use std::any::Any;
use std::hash::Hash;
use std::sync::Arc;

use bevy::app::App;
use bevy::asset::AssetContainer;
use bevy::prelude::{Plugin, Resource};
use itertools::Itertools;
use slab::Slab;
use unstructured::{Document, Unstructured};

use game2::mono_bundle::MonoBundle;

#[derive(Debug, Default)]
pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(
    Debug,
    Default,
    Clone,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MaterialDescription {
    pub id: usize,
    pub data: Option<Arc<Document>>,
}

impl MaterialDescription {
    pub const TRANSLUCENT_KEY: &'static str = "mesh/translucent";
    const AIR_MATERIAL_ID: usize = 0;
    pub const AIR: MaterialDescription = MaterialDescription {
        id: Self::AIR_MATERIAL_ID,
        data: None,
    };

    pub fn new(id: usize) -> Self {
        Self { id, data: None }
    }

    pub fn is_translucent(&self) -> bool {
        self.data
            .as_ref()
            .map(|data| data.select(Self::TRANSLUCENT_KEY).ok())
            .flatten()
            .map(|transparent| {
                if let &Unstructured::Bool(transparent) = transparent {
                    transparent
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    pub fn edit_document(&mut self) -> &mut Document {
        if let Some(data) = &mut self.data {
            Arc::make_mut(data)
        } else {
            self.data = Some(Arc::new(Document::default()));
            Arc::make_mut(self.data.as_mut().unwrap())
        }
    }

    pub fn set_translucent(&mut self, translucent: bool) {
        let document = self.edit_document();
        let transparent_section = document
            .select_mut(Self::TRANSLUCENT_KEY)
            .expect("Could not select transparent section");
        *transparent_section = Unstructured::Bool(translucent);
    }

    pub fn merged_clone(&self, document: Option<Document>) -> Self {
        if let Some(document) = document {
            let mut data = self.data.clone();
            if let Some(mut data) = &mut data {
                let mut document_edit = Arc::make_mut(&mut data);
                document_edit.merge(document);
            } else {
                data = Some(Arc::new(document));
            }
            Self { id: self.id, data }
        } else {
            Self {
                id: self.id,
                data: self.data.clone(),
            }
        }
    }
}

#[cfg(test)]
pub mod test {}
