use crate::aim_system::MouseLight;
use crate::player::PlayerControlled;
use crate::view_system::{run_first_person, run_third_person};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const MOVE_SENSITIVITY: f32 = 0.2;

pub struct MovePlugin;

impl Plugin for MovePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(run_first_person.system())
                .with_system(first_person_move_system.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_third_person.system())
                .with_system(third_person_move_system.system()),
        );
    }
}

fn first_person_move_system(
    keyboard_input: Res<Input<KeyCode>>,
    //modified to use rigid body position instead of transform because update direction
    //is one way rigidbody -> transform
    mut query: Query<&mut RigidBodyPosition, With<PlayerControlled>>,
) {
    for mut rigid_body in query.iter_mut() {
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
            rigid_body.position.translation.y += 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            rigid_body.position.translation.y -= 1.0 * MOVE_SENSITIVITY;
        }

        let rotated_direction = rotate_vec3_by_quat(rigid_body.position.rotation.into(), move_vec);
        rigid_body.position.translation.x += rotated_direction.x;
        rigid_body.position.translation.y += rotated_direction.y;
        rigid_body.position.translation.z += rotated_direction.z;
    }
}

fn rotate_vec3_by_quat(quat: Quat, vec: Vec3) -> Vec3 {
    let quat_vec = Vec3::new(quat.x, quat.y, quat.z);

    2.0 * quat_vec.dot(vec) * quat_vec
        + (quat.w * quat.w - quat_vec.dot(quat_vec)) * vec
        + 2.0 * quat.w * quat_vec.cross(vec)
}

fn third_person_move_system(
    mouse_input: Res<Input<MouseButton>>,
    mouse_query: Query<&GlobalTransform, With<MouseLight>>,
    mut player_query: Query<(&mut RigidBodyVelocity, &Transform), With<PlayerControlled>>,
) {
    if mouse_input.pressed(MouseButton::Right) {
        let mouse_location = mouse_query.iter().next().unwrap().translation;

        for (mut player_velocity, player_transform) in player_query.iter_mut() {
            let mut distance_vector = mouse_location - player_transform.translation.into();
            distance_vector.y = 0.; // clear out vertical movement

            // check to see if we're already at our location
            if distance_vector.length() > 0. {
                // get our move velocity
                let mut distance_translation = distance_vector.normalize() * 4.;

                // check to see if we're really close and should just step the rest of the way
                if distance_vector.length() < distance_translation.length() {
                    distance_translation = distance_vector;
                }

                player_velocity.linvel = distance_translation.into();

                // player_transform.next_position.translation.x += distance_translation.x;
                // player_transform.next_position.translation.z += distance_translation.z;
            }
        }
    }
}
