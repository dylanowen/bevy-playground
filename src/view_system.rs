use crate::aim_system::MouseLight;
use crate::player::Player;
use bevy::ecs::schedule::ShouldRun;
use bevy::input::mouse::MouseMotion;
use bevy::math::Mat2;
use bevy::math::Vec3Swizzles;

use bevy::prelude::*;
use core::mem;

pub const DISTANCE: f32 = 10.;
const PITCH_HEIGHT: f32 = DISTANCE;

pub struct FlyCam {
    yaw: f32,
    pitch: f32,
    x_sensitivity: f32,
    y_sensitivity: f32,
}
pub struct UiCam;
pub struct GameCam;

const MOVE_SENSITIVITY: f32 = 0.2;

#[derive(Default)]
pub struct ViewPlugin {}

#[derive(Eq, PartialEq)]
pub enum ViewKind {
    First,
    Fly,
    Third,
}

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(ViewKind::Third)
            .add_startup_system(setup_view_system.system())
            .add_system(switch_camera_view_system.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_third_person.system())
                    .with_system(third_person_system.system())
                    .with_system(third_person_move_system.system()),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_first_person.system())
                    .with_system(first_person_system.system())
            );
    }
}

fn setup_view_system(mut commands: Commands) {
    // build our main camera
    let mut camera_transform = Transform::default();
    focus_camera(
        Vec2::new(0., DISTANCE),
        Vec3::splat(0.),
        &mut camera_transform,
    );
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: camera_transform,
            ..Default::default()
        })
        .insert(GameCam)
        .insert(FlyCam {
            yaw: 0.0,
            pitch: 0.0,
            x_sensitivity: 0.2,
            y_sensitivity: 0.2,
        });
}

pub fn switch_camera_view_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut view_kind: ResMut<ViewKind>,
) {
    if keyboard_input.just_pressed(KeyCode::Insert) || keyboard_input.just_pressed(KeyCode::Grave) {
        *view_kind = match mem::replace(&mut *view_kind, ViewKind::Fly) {
            ViewKind::First => ViewKind::Third,
            ViewKind::Third => ViewKind::Fly,
            ViewKind::Fly => ViewKind::First,
        }
    }
}

impl ViewKind {
    fn should_run(&self, view_kind: &ViewKind) -> ShouldRun {
        if view_kind == self {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }
}

fn run_third_person(view_kind: Res<ViewKind>) -> ShouldRun {
    ViewKind::Third.should_run(&*view_kind)
}

#[allow(clippy::type_complexity)]
pub fn third_person_system(
    // mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<GameCam>>,
    )>,
) {
    let focus = query.q0().single().unwrap().translation;

    let mut rotation = 0.;
    if keyboard_input.pressed(KeyCode::Q) {
        rotation -= 0.1;
    }
    if keyboard_input.pressed(KeyCode::E) {
        rotation += 0.1;
    }

    // TODO this is wrong, we don't stay a certain distance from the player, we just watch them
    for mut transform in query.q1_mut().iter_mut() {
        let current_offset: Vec2 = transform.translation.xz() - focus.xz();
        let new_offset = Mat2::from_angle(rotation) * current_offset;

        focus_camera(new_offset, focus, &mut transform);
    }
}

pub fn third_person_move_system(
    mouse_input: Res<Input<MouseButton>>,
    mouse_query: Query<&GlobalTransform, With<MouseLight>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    if mouse_input.pressed(MouseButton::Right) {
        let mouse_location = mouse_query.iter().next().unwrap().translation;

        for mut player_transform in player_query.iter_mut() {
            let mut distance_vector = mouse_location - player_transform.translation;
            distance_vector.y = 0.; // clear out vertical movement

            // check to see if we're already at our location
            if distance_vector.length() > 0. {
                // get our move velocity
                let mut distance_translation = distance_vector.normalize() / 5.;

                // check to see if we're really close and should just step the rest of the way
                if distance_vector.length() < distance_translation.length() {
                    distance_translation = distance_vector;
                }

                player_transform.translation += distance_translation;
            }
        }
    }
}

pub fn focus_camera(offset_location: Vec2, focus: Vec3, transform: &mut Transform) {
    transform.translation = focus + Vec3::new(offset_location.x, PITCH_HEIGHT, offset_location.y);

    transform.look_at(focus, Vec3::Y);
}

fn run_first_person(view_kind: Res<ViewKind>) -> ShouldRun {
    ViewKind::First.should_run(&*view_kind)
}

pub fn first_person_system(
    mut ev_mouse: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &GameCam, Option<&mut FlyCam>)>,
) {
    let mut cam_delta: Vec2 = Vec2::ZERO;
    for event in ev_mouse.iter() {
        cam_delta += event.delta;
    }

    let (mut transform, _overheadcam, opt_flycam) = query.single_mut().unwrap();

    if let Some(mut flycam) = opt_flycam {
        flycam.yaw -= cam_delta.x * flycam.x_sensitivity;
        flycam.pitch += cam_delta.y * flycam.y_sensitivity;

        flycam.pitch = flycam.pitch.clamp(-89.9, 89.9);
        // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

        let yaw_radians = flycam.yaw.to_radians();
        let pitch_radians = flycam.pitch.to_radians();

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
            * Quat::from_axis_angle(-Vec3::X, pitch_radians);
    }
}

/*
pub fn first_person_move_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    for mut cam_transform in query.iter_mut() {
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
            cam_transform.translation.y += 1.0 * MOVE_SENSITIVITY;
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            cam_transform.translation.y -= 1.0 * MOVE_SENSITIVITY;
        }

        let rotated_direction = rotate_vec3_by_quat(cam_transform.rotation, move_vec);
        cam_transform.translation.x += rotated_direction.x;
        cam_transform.translation.y += rotated_direction.y;
        cam_transform.translation.z += rotated_direction.z;
    }
}

fn rotate_vec3_by_quat(quat: Quat, vec: Vec3) -> Vec3 {
    let quat_vec = Vec3::new(quat.x, quat.y, quat.z);

    2.0 * quat_vec.dot(vec) * quat_vec
        + (quat.w * quat.w - quat_vec.dot(quat_vec)) * vec
        + 2.0 * quat.w * quat_vec.cross(vec)
}
*/
