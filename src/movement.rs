use bevy::prelude::*;
use crate::player::PlayerControlled;

const MOVE_SENSITIVITY: f32 = 0.2;

pub struct Controllable;

pub fn first_person_move_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerControlled>>,
) {
    for mut transform in query.iter_mut() {
        let mut move_vec = Vec3::splat(0.0);
        if keyboard_input.pressed(KeyCode::W) {
            move_vec.z -= 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_vec.z += 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_vec.x -= 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_vec.x += 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            transform.translation.y += 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            transform.translation.y -= 1.0 * MOVE_SENSITIVITY;
        }

        let rotated_direction = rotate_vec3_by_quat(transform.rotation, move_vec);
        transform.translation.x += rotated_direction.x;
        transform.translation.y += rotated_direction.y;
        transform.translation.z += rotated_direction.z;
    }
}

fn rotate_vec3_by_quat(quat: Quat, vec: Vec3) -> Vec3 {
    let quat_vec = Vec3::new(quat.x, quat.y, quat.z);

    2.0 * quat_vec.dot(vec) * quat_vec
        + (quat.w * quat.w - quat_vec.dot(quat_vec)) * vec
        + 2.0 * quat.w * quat_vec.cross(vec)
}

pub fn cycle_control_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(Entity, Option<&PlayerControlled>), With<Controllable>>
) {
    if keyboard_input.just_pressed(KeyCode::Insert) || keyboard_input.just_pressed(KeyCode::Grave) {
        for (ent, playerControlled) in query.iter_mut() {
            if let Some(_playerControlled) = playerControlled {
                commands.entity(ent).remove::<PlayerControlled>();
            } else {
                commands.entity(ent).insert(PlayerControlled);
            }
        }
    }
}