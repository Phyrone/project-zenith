use bevy::prelude::*;
use slab::Slab;

use game2::BlockRotation;

#[derive(
    Clone,
    Copy,
    Debug,
    Component,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ClientBlockState {
    pub material: usize,
    pub rotation: BlockRotation,
}

pub const AIR: BlockMaterial = BlockMaterial { transparent: true };

#[derive(Clone, Debug, Resource)]
/// The material registry contains all materials that are used for the current world.
/// its stores all materials and references them by an id
///
pub struct MaterialRegistry {
    data: Slab<BlockMaterial>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Component,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct BlockMaterial {
    pub transparent: bool,
}

#[derive(Clone, Copy, Debug, Component)]
pub struct CustomMaterialVoxelRenderer {}

pub trait BlockToVoxelMapper {}

mod test {
    use slab::Slab;

    #[test]
    fn a_test() {
        let mut slab = Slab::new();
        slab.insert("hello");
        slab.insert("world");
        slab.insert("test");
        let json = serde_json::to_string_pretty(&slab).unwrap();
        println!("{}", json);
    }
}
