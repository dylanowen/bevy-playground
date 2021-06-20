use bevy::math::Mat2;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::render_graph::base::camera::CAMERA_3D;

pub const DISTANCE: f32 = 10.;
const PITCH_HEIGHT: f32 = DISTANCE;

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
