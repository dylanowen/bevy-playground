use bevy::input::mouse::MouseMotion;
use bevy::math::Mat2;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::render_graph::base::camera::CAMERA_3D;
use crate::player::Player;

pub const DISTANCE: f32 = 10.;
const PITCH_HEIGHT: f32 = DISTANCE;

pub struct FlyCam {
    yaw: f32,
    pitch: f32,
    x_sensitivity: f32,
    y_sensitivity: f32
}
pub struct UiCam;
pub struct OverheadCam;

pub fn camera_system(
    // mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &Camera)>,
) {
    let focus = Vec3::new(0., 0., 0.);

    let mut rotation = 0.;
    if keyboard_input.pressed(KeyCode::Q) {
        rotation -= 0.1;
    }
    if keyboard_input.pressed(KeyCode::E) {
        rotation += 0.1;
    }

    if rotation != 0. {
        for (mut transform, camera) in camera_query.iter_mut() {
            // make sure this is our 3d camera
            if camera.name.as_deref() == Some(CAMERA_3D) {
                let current_offset: Vec2 = transform.translation.xz() - focus.xz();
                let new_offset = Mat2::from_angle(rotation) * current_offset;

                focus_camera(new_offset, focus, &mut transform);
            }
        }
    }
}

pub fn focus_camera(offset_location: Vec2, focus: Vec3, transform: &mut Transform) {
    transform.translation = focus + Vec3::new(offset_location.x, PITCH_HEIGHT, offset_location.y);

    transform.look_at(focus, Vec3::Y);
}

pub fn mouse_look_system(
    mut ev_mouse: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &OverheadCam, Option<&mut FlyCam>)> 
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

pub fn switch_camera_view_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(Entity, &OverheadCam, Option<&FlyCam>)>
) {
    if keyboard_input.just_pressed(KeyCode::Insert) {
        let (ent, cam, flycam) = query.single_mut().unwrap();
        if let Some(flycam) = flycam {
            //flycam is already present
            commands.entity(ent).remove::<FlyCam>();
        } else {
            //flycam isn't present
            commands.entity(ent).insert(FlyCam {
                yaw: 0.0,
                pitch: 0.0,
                x_sensitivity: 0.2,
                y_sensitivity: 0.2
            });
        }   
    }
}
