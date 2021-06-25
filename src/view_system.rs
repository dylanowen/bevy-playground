use bevy::ecs::schedule::ShouldRun;
use bevy::input::mouse::MouseMotion;

use bevy::prelude::*;

use crate::player::Player;

const THIRD_X_DISTANCE: f32 = 7.;
const THIRD_Y_DISTANCE: f32 = THIRD_X_DISTANCE;
const MOUSE_SENSITIVITY: f32 = 0.2;

pub struct FlyCam {
    yaw: f32,
    pitch: f32,
    x_sensitivity: f32,
    y_sensitivity: f32,
}

pub struct ThirdPersonCam {
    offset: Vec3,
}

pub struct UiCam;
pub struct GameCam;

#[derive(Default)]
pub struct ViewPlugin {}

#[derive(Eq, PartialEq)]
pub enum ViewKind {
    First,
    Third,
}

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_view_system.system())
            .add_system(switch_camera_view_system.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_third_person.system())
                    .with_system(third_person_system.system()),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_first_person.system())
                    .with_system(first_person_system.system()),
            );
    }
}

fn setup_view_system(mut commands: Commands) {
    commands.insert_resource(ViewKind::Third);

    // build our main camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::default(),
            ..Default::default()
        })
        .insert(GameCam)
        .insert(FlyCam {
            yaw: 0.0,
            pitch: 0.0,
            x_sensitivity: MOUSE_SENSITIVITY,
            y_sensitivity: MOUSE_SENSITIVITY,
        })
        .insert(ThirdPersonCam {
            offset: Vec3::new(THIRD_X_DISTANCE, THIRD_Y_DISTANCE, 0.),
        });
}

fn switch_camera_view_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut view_kind: ResMut<ViewKind>,
) {
    if keyboard_input.just_pressed(KeyCode::Insert) || keyboard_input.just_pressed(KeyCode::Grave) {
        let window = windows.get_primary_mut().unwrap();

        *view_kind = match *view_kind {
            ViewKind::First => {
                // give up our mouse
                window.set_cursor_lock_mode(false);
                window.set_cursor_visibility(true);
                ViewKind::Third
            }
            ViewKind::Third => {
                // grab our mouse
                window.set_cursor_lock_mode(true);
                window.set_cursor_visibility(false);
                ViewKind::First
            }
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

pub fn run_first_person(view_kind: Res<ViewKind>) -> ShouldRun {
    ViewKind::First.should_run(&*view_kind)
}

pub fn run_third_person(view_kind: Res<ViewKind>) -> ShouldRun {
    ViewKind::Third.should_run(&*view_kind)
}

#[allow(clippy::type_complexity)]
fn first_person_system(
    mut ev_mouse: EventReader<MouseMotion>,
    mut query: QuerySet<(
        Query<&mut FlyCam, With<GameCam>>,
        Query<&mut Transform, With<Player>>,
        Query<&mut Transform, With<GameCam>>,
    )>,
) {
    let mut cam_delta: Vec2 = Vec2::ZERO;
    for event in ev_mouse.iter() {
        cam_delta += event.delta;
    }

    // get our rotation
    let mut flycam = query.q0_mut().single_mut().unwrap();
    flycam.yaw -= cam_delta.x * flycam.x_sensitivity;
    flycam.pitch += cam_delta.y * flycam.y_sensitivity;

    flycam.pitch = flycam.pitch.clamp(-89.9, 89.9);
    // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

    let yaw_radians = flycam.yaw.to_radians();
    let pitch_radians = flycam.pitch.to_radians();

    let x_rotation = Quat::from_axis_angle(-Vec3::X, pitch_radians);
    let y_rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians);
    let rotation = y_rotation * x_rotation;

    // rotate our player
    let mut player = query.q1_mut().single_mut().unwrap();
    player.rotation = y_rotation;
    let player_location = player.translation;

    // rotate our camera
    let mut camera = query.q2_mut().single_mut().unwrap();
    camera.rotation = rotation;
    // keep our camera at our player's head
    camera.translation = Vec3::new(player_location.x, player_location.y + 1.7, player_location.z);
}

#[allow(clippy::type_complexity)]
pub fn third_person_system(
    // mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: QuerySet<(
        Query<&Transform, With<Player>>,
        Query<&mut ThirdPersonCam, With<GameCam>>,
        Query<&mut Transform, With<GameCam>>,
    )>,
) {
    let player_location = query.q0().single().unwrap().translation;

    let mut rotation = 0.;
    if keyboard_input.pressed(KeyCode::Q) {
        rotation -= 0.1;
    }
    if keyboard_input.pressed(KeyCode::E) {
        rotation += 0.1;
    }

    // get and update our camera offset
    let mut camera_offset = query.q1_mut().single_mut().unwrap();
    let new_offset: Vec3 = Mat3::from_rotation_y(rotation) * camera_offset.offset;
    camera_offset.offset = new_offset;

    for mut camera in query.q2_mut().iter_mut() {
        camera.translation = player_location + new_offset;
        camera.look_at(player_location, Vec3::Y);
    }
}
