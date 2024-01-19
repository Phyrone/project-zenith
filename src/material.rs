use subenum::subenum;

use crate::BlockRotation;

#[subenum(Block)]
#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Material {
    #[default]
    #[subenum(Block)]
    AIR,
    #[subenum(Block)]
    STONE,
    #[subenum(Block)]
    DIRT,
    #[subenum(Block)]
    GRASS,
    #[subenum(Block)]
    WOOD(WoodData),
    #[subenum(Block)]
    WoodPlanks(WoodPlanksData),
    #[subenum(Block)]
    LEAVES,
    #[subenum(Block)]
    A,

    APPLE,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct WoodData {
    rotation: BlockRotation,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct WoodPlanksData {
    rotation: BlockRotation,
}
