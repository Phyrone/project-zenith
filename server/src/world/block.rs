use bevy::prelude::Entity;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct BlockData {
    //None = air
    //Some = entity id to resource definition
    material: Option<Entity>,
}

impl BlockData {
    pub fn new(material: Option<Entity>) -> Self {
        Self { material }
    }
}
