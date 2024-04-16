use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use bevy::prelude::Resource;
use hashbrown::HashMap;
use slab::Slab;

use crate::material::ResourceKey;

/// simply the marker type used when no marker is needed/set in [Registry] for convenience
#[derive(Debug, Default, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct NoMarker;

/// A [Registry] for storing any kind of resources
///  a registry does 2 things:
///   - it centrally stores arbitrary data to a [ResourceKey] so it can be accessed later
///   - it creates an id for each resource key so an often used resource key can be stored more efficiently
///
/// the [Registry] is basically copy on write (more at [Arc::make_mut]).
/// So it is cheap to clone and can be easily shared between threads
/// but editing the registry may be expensive and does will not be shared to clones made before the edit
///
/// The registry also implements the [Resource] trait, so it can be used as a resource in bevy.
/// to allow multiple registries in one bevy world, create some marker type and set it at [M]
/// default is [NoMarker]
///
#[derive(Debug, Clone, Resource)]
pub struct Registry<T, M = NoMarker> {
    inner: Arc<RegistryInner<T>>,
    _marker: std::marker::PhantomData<M>,
}

impl<T, M> Default for Registry<T, M> {
    fn default() -> Self {
        Self {
            inner: Arc::new(RegistryInner::default()),
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
struct RegistryInner<T> {
    id_mapper: Slab<ResourceKey>,
    key_to_data: HashMap<ResourceKey, RegistryEntry<T>>,
}

impl<T> Default for RegistryInner<T> {
    fn default() -> Self {
        Self {
            id_mapper: Slab::new(),
            key_to_data: HashMap::new(),
        }
    }
}

impl<T, M> Registry<T, M>
    where
        T: Clone + Hash,
{
    fn edit(&mut self) -> &mut RegistryInner<T> {
        Arc::make_mut(&mut self.inner)
    }

    pub fn register_material(&mut self, key: ResourceKey, data: T) -> usize {
        let RegistryInner {
            id_mapper,
            key_to_data,
        } = self.edit();
        let entry = key_to_data.get(&key);
        let short_key = if let Some(entry) = entry {
            entry.id
        } else {
            id_mapper.insert(key.clone()) + 1
        };
        key_to_data.insert(
            key,
            RegistryEntry {
                id: short_key,
                data,
            },
        );
        short_key
    }

    pub fn unregister_material(&mut self, key: &ResourceKey) {
        let RegistryInner {
            id_mapper,
            key_to_data,
        } = self.edit();

        if let Some(entry) = key_to_data.remove(key) {
            id_mapper.remove(entry.id - 1);
        }
    }

    pub fn shrink_to_fit(&mut self) {
        let RegistryInner {
            id_mapper,
            key_to_data,
        } = self.edit();
        id_mapper.shrink_to_fit();
        key_to_data.shrink_to_fit();
    }

    pub fn clear(&mut self) {
        let RegistryInner {
            id_mapper,
            key_to_data,
        } = self.edit();
        id_mapper.clear();
        key_to_data.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        let RegistryInner {
            id_mapper,
            key_to_data,
        } = self.edit();
        id_mapper.reserve(additional);
        key_to_data.reserve(additional);
    }

    pub fn get_by_key(&self, key: &ResourceKey) -> Option<&RegistryEntry<T>> {
        self.inner.key_to_data.get(key)
    }
    pub fn get_by_id(&self, id: usize) -> Option<&RegistryEntry<T>> {
        self.inner
            .id_mapper
            .get(id + 1)
            .and_then(|key| self.inner.key_to_data.get(key))
    }
    pub fn key_of_id(&self, id: usize) -> Option<&ResourceKey> {
        self.inner.id_mapper.get(id + 1)
    }
    pub fn id_of_key(&self, key: &ResourceKey) -> Option<usize> {
        self.inner.key_to_data.get(key).map(|entry| entry.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct RegistryEntry<T> {
    pub id: usize,
    pub data: T,
}

impl<T> Deref for RegistryEntry<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for RegistryEntry<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_registry() -> Registry<unstructured::Document> {
        Registry::default()
    }
}
