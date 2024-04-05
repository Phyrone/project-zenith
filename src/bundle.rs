use bevy::math::u32;
use std::any::{Any, TypeId};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Debug, Formatter};

use hashbrown::HashMap;
use rayon::prelude::*;


pub type AnyComponent = dyn Any + Send + Sync;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Bundle {
    id: u32,
}
impl Bundle {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn of(id: u32) -> Self {
        Self { id }
    }
}

pub struct BundleDescriptor<Registry> {
    bundle: Bundle,
    registry: Registry,
}

impl<Registry> BundleDescriptor<Registry> {
    fn new(bundle: Bundle, registry: Registry) -> Self {
        Self { bundle, registry }
    }

    pub fn bundle(&self) -> Bundle {
        self.bundle
    }
}

impl<Registry> BundleDescriptor<Registry>
where
    Registry: Borrow<BundleRegistry>,
{
    pub fn contains<C>(&self) -> bool
    where
        C: Any,
    {
        self.registry.borrow().has_component::<C>(&self.bundle)
    }

    pub fn get<C>(&self) -> Option<&C>
    where
        C: Any,
    {
        self.registry.borrow().get_component::<C>(&self.bundle)
    }
}

impl<Registry> BundleDescriptor<Registry>
where
    Registry: BorrowMut<BundleRegistry>,
{
    pub fn get_mut<C>(&mut self) -> Option<&mut C>
    where
        C: Any,
    {
        self.registry
            .borrow_mut()
            .get_component_mut::<C>(&self.bundle)
    }
    pub fn add_component<C>(&mut self, component: C)
    where
        C: Any + Send + Sync,
    {
        self.registry
            .borrow_mut()
            .add_component(self.bundle, component);
    }
    pub fn remove_component<C>(&mut self)
    where
        C: Any,
    {
        self.registry
            .borrow_mut()
            .remove_component::<C>(&self.bundle);
    }
    pub fn remove(&mut self) {
        self.registry.borrow_mut().remove_bundle(&self.bundle);
    }
}

/// A that allows to tangle multiple components together into a single bundle
/// it is quite close to an ECS except it does not contain systems
///   Bundle = Entity
///   Component = Component
#[derive(Default)]
pub struct BundleRegistry {
    next_id: u32,
    holders: HashMap<TypeId, TypedComponentHolder>,
}

impl Debug for BundleRegistry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BundleRegistry")
            .field("next_id", &self.next_id)
            .field("holders", &self.holders)
            .finish()
    }
}

impl BundleRegistry {
    pub fn bundle_mut(&mut self, bundle: Bundle) -> BundleDescriptor<&mut BundleRegistry> {
        BundleDescriptor::new(bundle, self)
    }
    pub fn bundle(&self, bundle: Bundle) -> BundleDescriptor<&BundleRegistry> {
        BundleDescriptor::new(bundle, self)
    }
    pub fn new_bundle(&mut self) -> BundleDescriptor<&mut BundleRegistry> {
        let bundle = Bundle {
            id: self.determine_next_bundle_id(),
        };
        BundleDescriptor::new(bundle, self)
    }

    fn determine_next_bundle_id(&mut self) -> u32 {
        if self.next_id == u32::MAX {
            panic!("out of available bundle ids")
        }
        self.next_id += 1;
        self.next_id
    }

    fn has_component<C>(&self, bundle: &Bundle) -> bool
    where
        C: Any,
    {
        self.component_holder::<C>()
            .map_or(false, |holder| holder.components.contains_key(bundle))
    }

    fn get_component<C>(&self, bundle: &Bundle) -> Option<&C>
    where
        C: Any,
    {
        self.component_holder::<C>()
            .and_then(|holder| holder.components.get(bundle))
            .and_then(|component| component.downcast_ref())
    }

    fn get_component_mut<C>(&mut self, bundle: &Bundle) -> Option<&mut C>
    where
        C: Any,
    {
        self.component_holder_mut::<C>()
            .components
            .get_mut(bundle)
            .and_then(|component| component.downcast_mut())
    }
    pub fn get_bundles_with_component<C>(&self) -> Vec<Bundle>
    where
        C: Any,
    {
        self.component_holder::<C>()
            .map_or_else(Vec::new, |holder| {
                holder.components.keys().cloned().collect()
            })
    }
    pub fn get_all_bundles(&self) -> Vec<Bundle> {
        self.holders
            .values()
            .flat_map(|holder| holder.components.keys())
            .cloned()
            .collect()
    }

    fn add_component<C>(&mut self, bundle: Bundle, component: C)
    where
        C: Any + Send + Sync,
    {
        let holder = self.component_holder_mut::<C>();
        holder.components.insert(bundle, Box::new(component));
    }

    fn remove_component<C>(&mut self, bundle: &Bundle) -> Option<C>
    where
        C: Any,
    {
        let holder = self.component_holder_mut::<C>();
        let res = holder.components.remove(bundle);
        if holder.is_empty() {
            self.holders.remove(&TypeId::of::<C>());
        }
        res.and_then(|component| component.downcast().ok().map(|component| *component))
    }
    fn remove_bundle(&mut self, bundle: &Bundle) -> Vec<Box<AnyComponent>> {
        let mut components = Vec::new();
        for holder in self.holders.values_mut() {
            let removed = holder.components.remove(bundle);
            if let Some(removed) = removed {
                components.push(removed);
            }
        }
        self.holders.retain(|_, holder| holder.is_not_empty());
        components
    }

    fn component_holder<C>(&self) -> Option<&TypedComponentHolder>
    where
        C: Any,
    {
        let type_id = TypeId::of::<C>();
        self.holders.get(&type_id)
    }

    fn component_holder_mut<C>(&mut self) -> &mut TypedComponentHolder
    where
        C: Any,
    {
        let type_id = TypeId::of::<C>();
        self.holders
            .entry(type_id)
            .or_insert_with(|| TypedComponentHolder::new(type_id))
    }

    pub fn clear(&mut self) {
        self.next_id = 0;
        self.holders.clear();
    }
}

impl AsRef<BundleRegistry> for BundleRegistry {
    fn as_ref(&self) -> &BundleRegistry {
        self
    }
}

impl AsMut<BundleRegistry> for BundleRegistry {
    fn as_mut(&mut self) -> &mut BundleRegistry {
        self
    }
}

#[derive()]
struct TypedComponentHolder {
    type_id: TypeId,
    components: HashMap<Bundle, Box<AnyComponent>>,
}

impl Debug for TypedComponentHolder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedComponentHolder")
            .field("type_id", &self.type_id)
            .finish()
    }
}

impl TypedComponentHolder {
    pub fn new(type_id: TypeId) -> Self {
        Self {
            type_id,
            components: HashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    #[inline]
    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_components() {
        let mut registry = BundleRegistry::default();

        let mut bundle = registry.new_bundle();
        bundle.add_component(42_i32);
        assert_eq!(bundle.get::<i32>(), Some(&42));
    }
}
