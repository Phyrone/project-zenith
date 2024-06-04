use std::any::{Any, TypeId};
use std::ops::Deref;
use std::sync::Arc;

use hashbrown::HashMap;

pub type AnyComponent = dyn Any + Send + Sync;

#[derive(Default, Clone)]
pub struct Bundle {
    components: HashMap<TypeId, Arc<AnyComponent>>,
}

impl Bundle {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn all(&self) -> Vec<&AnyComponent> {
        self.components
            .values()
            .map(|boxed| boxed.deref())
            .collect()
    }

    pub fn insert<C>(&mut self, component: C) -> Option<C>
    where
        C: Any + Send + Sync,
    {
        let type_id = TypeId::of::<C>();
        let boxed = Arc::new(component);

        self.components
            .insert(type_id, boxed)
            .and_then(|boxed| boxed.downcast::<C>().ok())
            .and_then(|boxed| Arc::into_inner(boxed))
    }

    pub fn contains<C>(&self) -> bool
    where
        C: Any + Send + Sync,
    {
        let type_id = TypeId::of::<C>();
        self.components.contains_key(&type_id)
    }

    pub fn get<C>(&self) -> Option<&C>
    where
        C: Any + Send + Sync,
    {
        let type_id = TypeId::of::<C>();
        self.components
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<C>())
    }
    pub fn get_mut<C>(&mut self) -> Option<&mut C>
    where
        C: Any + Send + Sync,
    {
        let type_id = TypeId::of::<C>();

        self.components
            .get_mut(&type_id)
            .and_then(|boxed| Arc::get_mut(boxed))
            .and_then(|boxed| boxed.downcast_mut::<C>())
    }

    pub fn remove<C>(&mut self) -> Option<C>
    where
        C: Any + Send + Sync,
    {
        let type_id = TypeId::of::<C>();
        self.components
            .remove(&type_id)
            .and_then(|boxed| boxed.downcast::<C>().ok())
            .and_then(|boxed| Arc::into_inner(boxed))
    }

    pub fn clear(&mut self) {
        self.components.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestComponentA;

    struct TestComponentB;

    struct TestComponentC;

    #[test]
    pub fn test_components() {
        let mut bundle = Bundle::new();
        bundle.insert(TestComponentA);
        bundle.insert(TestComponentB);
        bundle.insert(TestComponentC);

        assert!(bundle.contains::<TestComponentA>());
        assert!(bundle.contains::<TestComponentB>());
        assert!(bundle.contains::<TestComponentC>());
    }
}
