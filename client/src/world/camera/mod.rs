use std::f32::consts::PI as PI_32;
use std::ops::{Deref, DerefMut};

use bevy::app::App;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub mod lod;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, States)]
enum WorldCameraConfiguration {
    #[default]
    None,

    FirstPersonCamera,
    ThirdPersonCamera,
    //for debugging and testing only (excluded from release builds)
    #[cfg(debug_assertions)]
    FreeCamera,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, States)]
enum PlayerControlMode {
    #[default]
    OutGameMenu,
    InGameMenu,
    InGame,
}

#[derive(Debug, Default)]
pub struct WorldCameraPlugin;

impl Plugin for WorldCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(WorldCameraConfiguration::default())
            .add_systems(
                Update,
                (
                    test_camera_move_mouse_system,
                    test_camera_move_keys_system,
                    jump_rotation_test_cam_system,
                ),
            );
        //.add_plugins(MaterialPlugin);
    }
}

fn jump_rotation_test_cam_system(
    mut query: Query<&mut Transform, With<Camera>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for mut transform in query.iter_mut() {
        if keys.just_pressed(KeyCode::KeyY) {
            transform.rotation = Quat::from_rotation_y(PI_32 / 2f32);
        }
        if keys.just_pressed(KeyCode::KeyX) {
            transform.rotation = Quat::from_rotation_x(PI_32 / 2f32);
        }
        if keys.just_pressed(KeyCode::KeyZ) {
            transform.rotation = Quat::from_rotation_z(PI_32 / 2f32);
        }
    }
}

fn is_overrated(rotation: &Quat) -> bool {
    ((*rotation) * Vec3::Y).y < 0.0
}

fn get_updown_angle(rotation: &Quat) -> f32 {
    let looking_direction = (*rotation) * Vec3::Z;
    let mut flatd = looking_direction.clone();
    flatd.y = 0.0;
    let flatd = flatd.normalize();
    let angle = looking_direction.angle_between(flatd);
    if is_overrated(rotation) {
        PI_32 - angle
    } else {
        angle
    }
}

fn test_camera_move_mouse_system(
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for motion in motion_evr.read() {
        for mut transform in query.iter_mut() {
            let angle = get_updown_angle(&transform.rotation);
            let new_rotation = transform.rotation * Quat::from_rotation_x(-motion.delta.y * 0.01);
            let new_angle = get_updown_angle(&new_rotation);

            if new_angle <= (PI_32 / 2f32) - 0.01 || angle > new_angle {
                transform.rotation = new_rotation;
            }

            //transform.rotate_local_x(-motion.delta.y * 0.01);
            transform.rotate_y(-motion.delta.x * 0.01);
            //let angle = transform.rotation.angle_between(Quat::from_rotation_x(PI_32 / 2f32));
        }
    }
}

struct Speed(f32);
impl Default for Speed {
    fn default() -> Self {
        Speed(30.0)
    }
}
impl Deref for Speed {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Speed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn test_camera_move_keys_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
    delta_time: Res<Time>,
    mut speed: Local<Speed>,
) {
    let mut forward_back = 0.0f32;
    let mut left_right = 0.0f32;
    let mut up_down = 0.0f32;
    let mut speed_multiplier = *speed.deref().deref();
    if keys.pressed(KeyCode::ShiftLeft) {
        up_down -= 1.0;
    }
    if keys.pressed(KeyCode::Space) {
        up_down += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        forward_back -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        forward_back += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        left_right -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        left_right += 1.0;
    }
    if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
        speed_multiplier *= 2.0;
    }

    let speed = speed.deref_mut().deref_mut();
    if keys.just_pressed(KeyCode::ArrowUp) {
        *speed += 1.0;
        info!("speed: {}", *speed);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        *speed -= 1.0;
        info!("speed: {}", *speed);
    }

    if forward_back == 0.0 && left_right == 0.0 && up_down == 0.0 {
        return;
    }

    query.iter_mut().for_each(|mut transform| {
        //camera diection on the xz plane
        let camera_direction = transform.local_z();
        let move_speed = speed_multiplier * delta_time.delta_seconds();
        let foward_vector =
            Vec3::new(camera_direction.x, 0.0, camera_direction.z).normalize() * move_speed;
        let right_vector =
            Vec3::new(camera_direction.z, 0.0, -camera_direction.x).normalize() * move_speed;
        transform.translation += foward_vector * forward_back
            + right_vector * left_right
            + Vec3::new(0.0, move_speed * up_down, 0.0);
    });
}
