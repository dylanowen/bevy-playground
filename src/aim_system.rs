use crate::player::Player;
use crate::view_system::ViewKind;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::render_graph::base::camera::CAMERA_3D;

#[derive(Default)]
pub struct MouseLight;

#[derive(Default, Bundle)]
pub struct MouseLightBundle<B: Bundle> {
    pub tag: MouseLight,
    #[bundle]
    pub light: B,
}

#[allow(clippy::type_complexity)]
pub fn aim_system(
    view_kind: Res<ViewKind>,
    windows: Res<Windows>,
    query: QuerySet<(
        Query<(&GlobalTransform, &Camera)>,
        Query<&mut Transform, With<MouseLight>>,
        Query<&mut Transform, With<Player>>,
    )>,
) {
    match *view_kind {
        ViewKind::First => first_person_aim(),
        ViewKind::Third => third_person_aim(windows, query),
    }
}

fn first_person_aim() {}

#[allow(clippy::type_complexity)]
fn third_person_aim(
    windows: Res<Windows>,
    mut query: QuerySet<(
        Query<(&GlobalTransform, &Camera)>,
        Query<&mut Transform, With<MouseLight>>,
        Query<&mut Transform, With<Player>>,
    )>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(cursor_position) = window.cursor_position() {
        // get the inverse of our camera view matrix and our camera position
        let maybe_inv_camera_projection = query.q0().iter().find_map(|(transform, camera)| {
            if camera.name.as_deref() == Some(CAMERA_3D) {
                Some(inv_camera_projection(camera, transform))
            } else {
                None
            }
        });

        if let Some(gpu_to_world) = maybe_inv_camera_projection {
            // transform our cursor position into our normalized device coordinates
            let screen_size = Vec2::new(window.width(), window.height());
            let normalized_cursor = (cursor_position / screen_size) * 2. - Vec2::splat(1.);

            // borrowed from https://github.com/aevyrie/bevy_mod_raycast/blob/master/src/primitives.rs
            // deal with near and far to support ortho cameras
            let cursor_near_gpu = normalized_cursor.extend(-1.);
            let cursor_far_gpu = normalized_cursor.extend(1.);

            let cursor_near_world = gpu_to_world.project_point3(cursor_near_gpu);
            let cursor_far_world = gpu_to_world.project_point3(cursor_far_gpu);

            // in world coordinates, a ray from our near to far plane through our cursor
            let cursor_ray = cursor_far_world - cursor_near_world;

            // get the negative normal of our ground to our near world cursor
            let ground_near_normal = Vec3::new(0., -cursor_near_world.y, 0.);

            // Using the dot product we have
            // ground_near_normal · cursor_ray = |cursor_ray| |ground_near_normal| cos(θ)
            // Using sohCAHtoa we have
            // cos(θ) = |ground_near_normal| / |ray_to_ground|
            // since we want |ray_to_ground| we can do
            // |ray_to_ground| = |ground_near_normal| / ( ground_near_normal · cursor_ray / |cursor_ray| |ground_near_normal|)
            // which reduces to
            // |ray_to_ground| = (|ground_near_normal|² * |cursor_ray|) /  (ground_near_normal · cursor_ray)
            let distance_to_ground = (cursor_near_world.y.powf(2.) * cursor_ray.length())
                / ground_near_normal.dot(cursor_ray);
            let ray_to_ground = cursor_ray.normalize() * distance_to_ground;

            let ground_intersection = ray_to_ground + cursor_near_world;

            // place our light slightly above the ground
            let light_location = Vec3::new(ground_intersection.x, 0.1, ground_intersection.z);

            for mut transform in query.q1_mut().iter_mut() {
                transform.translation = light_location;
            }
            for mut transform in query.q2_mut().iter_mut() {
                *transform = transform.looking_at(ground_intersection, Vec3::Y);
            }
        }
    }
}

/// Returns a matrix that can transform GPU to World coordinates
fn inv_camera_projection(camera: &Camera, transform: &GlobalTransform) -> Mat4 {
    let camera_position = transform.compute_matrix();
    let projection: Mat4 = camera.projection_matrix;

    camera_position * projection.inverse()
}
