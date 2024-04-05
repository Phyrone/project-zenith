use std::any::Any;
use std::hash::Hash;
use std::sync::Arc;

use bevy::app::App;
use bevy::prelude::{Plugin, Resource};
use slab::Slab;

use game2::mono_bundle::MonoBundle;

#[derive(Debug, Default)]
pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MaterialRegistry>();
    }
}

#[derive(Resource, Clone)]
pub struct MaterialRegistry {
    //bah!... but i didnt found a better solution yet
    materials: rclite::Arc<Slab<Arc<MonoBundle>>>,
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        Self {
            materials: rclite::Arc::new(Slab::new()),
        }
    }
}

impl MaterialRegistry {
    const AIR_DATA: usize = 0;

    fn edit_materials(&mut self) -> &mut Slab<Arc<MonoBundle>> {
        rclite::Arc::make_mut(&mut self.materials)
    }
    pub fn insert(&mut self, material: MonoBundle) -> usize {
        let materials = self.edit_materials();
        materials.insert(Arc::new(material))
    }
    pub fn create(&mut self) -> (usize, Arc<MonoBundle>) {
        let bundle = Arc::new(MonoBundle::new());
        let material_id = self.edit_materials().insert(bundle.clone());
        (material_id, bundle)
    }

    pub fn get_bundle(&self, id: usize) -> Option<Arc<MonoBundle>> {
        self.materials.get(id).map(|bundle| bundle.clone())
    }

    pub fn get<T>(&self, id: usize) -> Option<&T>
    where
        T: Any + Send + Sync,
    {
        self.materials.get(id).and_then(|bundle| bundle.get::<T>())
    }

    pub fn remove(&mut self, id: usize) -> Option<Arc<MonoBundle>> {
        let materials = self.edit_materials();
        if materials.contains(id) {
            Some(materials.remove(id))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.materials.clear();
    }
}



