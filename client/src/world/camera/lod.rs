use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct LODPlugin;

impl Plugin for LODPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LodConfig>()
            .add_systems(Update, (lod_update_system).in_set(LodLevelStage));
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, SystemSet)]
pub struct LodLevelStage;

#[derive(Debug, Default, Clone, Eq, PartialEq, Component)]
//TODO allow other systems to influence the LOD level f.e. when the player is looking trough a portal
pub struct LODObject {
    pub lod_level: LODLevel,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
#[derive(Component)]
//TODO rename levels to be a bit more descriptive (low priority, low effort when refactoring tools work)
pub enum LODLevel {
    /// Entity is near/next to camera and should be rendered at full detail
    /// chunks use 16x16x16 voxels per block
    /// animations are enabled
    FULL = 255,
    /// Entity is further away from the camera and can be rendered with less detail
    /// chunks use 4x4x4 voxels per block
    /// animations can be reduced
    Far1 = 254,
    /// Entity is further away from the camera and can be rendered with less detail
    /// chunks use 1x1x1 voxels per block
    /// animations are disabled
    /// moving objects can be rendered as single blobs
    Far2 = 253,

    /// Entity is in far distance and can be rendered with low detail
    /// chunks merge 4 blocks into one (e.g. 32x32x32 -> 8x8x8)
    /// moving objects are not rendered
    Far3 = 251,

    /// Entity is in far distance and can be rendered with low detail
    /// chunks merge 16 blocks into one (e.g. 32x32x32 -> 1x1x1)
    Fartest = 1,
    #[default]
    Hidden = 0,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Resource)]
pub struct LodConfig {
    pub full_lod_distance: f32,
    pub far1_lod_distance: f32,
    pub far2_lod_distance: f32,
    pub far3_lod_distance: f32,
    pub buffer_zone: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        //TODO more reasonable defaults
        LodConfig {
            full_lod_distance: 100.0,
            far1_lod_distance: 200.0,
            far2_lod_distance: 400.0,
            far3_lod_distance: 800.0,
            buffer_zone: 50.0,
        }
    }
}

//TODO do not update if camera and lod not moved
//TODO support for larger objects by using a center point and radius or something similar
//TODO set not visible objects hidden
fn lod_update_system(
    config: Res<LodConfig>,
    cameras: Query<&Transform, With<Camera>>,
    mut lod_objects: Query<(&Transform, &mut LODObject)>,
) {
    if cameras.is_empty() {
        return;
    }

    lod_objects
        .par_iter_mut()
        .for_each(|(lod_object_position, lod_object)| {
            //get closest camera
            let closest_camera = cameras
                .iter()
                .min_by(|a, b| {
                    let a_distance = a.translation.distance(lod_object_position.translation);
                    let b_distance = b.translation.distance(lod_object_position.translation);
                    a_distance.partial_cmp(&b_distance).unwrap()
                })
                .unwrap();

            //get distance to closest camera
            let distance_to_camera = closest_camera
                .translation
                .distance(lod_object_position.translation);
            //set lod level based on distance
            //there is a buffer zone to prevent flickering and unnecessary updates
            //when an lod object is on the edge of a lod level in the buffer zone it will not be updated
            let current_level = &lod_object.lod_level;

            let (new_level, target_distance) = if distance_to_camera < config.full_lod_distance {
                (LODLevel::FULL, config.full_lod_distance)
            } else if distance_to_camera < config.far1_lod_distance {
                (LODLevel::Far1, config.far1_lod_distance)
            } else if distance_to_camera < config.far2_lod_distance {
                (LODLevel::Far2, config.far2_lod_distance)
            } else if distance_to_camera < config.far3_lod_distance {
                (LODLevel::Far3, config.far3_lod_distance)
            } else {
                (LODLevel::Fartest, config.far3_lod_distance)
            };

            let mut border_distance = target_distance - distance_to_camera;

            if border_distance < 0.0 {
                border_distance *= -1.0;
            }

            if border_distance <= config.buffer_zone {
                return;
            }

            if new_level != *current_level {
                lod_object.into_inner().lod_level = new_level;
            }
        })
}
