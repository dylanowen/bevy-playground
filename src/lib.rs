mod camera;
mod debug;
mod level;

use crate::camera::{camera_system, focus_camera};
use crate::level::Chunk;

use bevy::prelude::*;

use crate::debug::Debug;

use bevy::input::system::exit_on_esc_system;
use bevy::render::camera::Camera;
use bevy::render::render_graph::base::camera::CAMERA_3D;
use std::f32::consts::FRAC_PI_2;
use wasm_bindgen::prelude::*;

#[derive(Default)]
struct Game {}

#[wasm_bindgen]
pub fn run() {
    default_plugins(&mut App::build())
        .add_startup_system(setup.system())
        .add_system(camera_system.system())
        .add_system(exit_on_esc_system.system())
        .add_system(light.system())
        // diagnostics
        .add_plugin(Debug::default())
        .run();
}

#[cfg(target_arch = "wasm32")]
fn default_plugins(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder.add_plugins(bevy_webgl2::DefaultPlugins)
}

#[cfg(not(target_arch = "wasm32"))]
fn default_plugins(builder: &mut AppBuilder) -> &mut AppBuilder {
    builder
        .add_plugins(DefaultPlugins)
        // diagnostics
        .add_plugin(bevy_prototype_debug_lines::DebugLinesPlugin)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Tell the asset server to watch for asset changes on disk:
    asset_server.watch_for_changes().unwrap();

    // build our main camera
    let mut camera_transform = Transform::default();
    focus_camera(
        Vec2::new(0., camera::DISTANCE),
        Vec3::splat(0.),
        &mut camera_transform,
    );
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: camera_transform,
        ..Default::default()
    });

    // add a ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // add some light
    commands.spawn_bundle(MouseLightBundle {
        light: LightBundle {
            transform: Transform::from_xyz(4.0, 5.0, 4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // build our map
    let grass_handle = asset_server.load("models.glb#Scene0");
    let wall_handle = asset_server.load("models.glb#Scene1");

    const WIDTH: usize = 40;
    const HEIGHT: usize = 40;
    let chunk = Chunk::<WIDTH, HEIGHT>::random(&mut rand::thread_rng());

    let x_offset = (WIDTH / 2) as f32;
    let z_offset = (HEIGHT / 2) as f32;
    for z in 0..HEIGHT {
        for x in 0..WIDTH {
            let transform = Transform::from_xyz(x as f32 - x_offset, 0., z as f32 - z_offset);

            for (dx, dz, rotate) in [(1, 0, false), (0, 1, true)].iter() {
                let nx = x + dx;
                let nz = z + dz;
                if nx < WIDTH && nz < HEIGHT {
                    if chunk.grid[z][x] != chunk.grid[nz][nx] {
                        // todo this is gross
                        let wall_transform = if *rotate {
                            let mut wall_transform = transform.clone();
                            wall_transform.rotate(Quat::from_rotation_y(-FRAC_PI_2));
                            wall_transform
                        } else {
                            Transform::from_xyz(nx as f32 - x_offset, 0., nz as f32 - z_offset)
                        };

                        commands
                            .spawn_bundle((wall_transform, GlobalTransform::identity()))
                            .with_children(|tile| {
                                tile.spawn_scene(wall_handle.clone());
                            });
                    }
                }
            }
            if chunk.grid[z][x] {
                commands
                    .spawn_bundle((transform, GlobalTransform::identity()))
                    .with_children(|tile| {
                        tile.spawn_scene(grass_handle.clone());
                    });
            }
        }
    }
}

#[derive(Default)]
struct MouseLight;

#[derive(Default, Bundle)]
struct MouseLightBundle<B: Bundle> {
    tag: MouseLight,
    #[bundle]
    light: B,
}

fn light(
    windows: Res<Windows>,
    mut query: QuerySet<(
        Query<(&mut Transform, &MouseLight)>,
        Query<(&GlobalTransform, &Camera)>,
    )>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(cursor_position) = window.cursor_position() {
        // get the inverse of our camera view matrix and our camera position
        let maybe_inv_camera_projection = query.q1().iter().find_map(|(transform, camera)| {
            if camera.name.as_deref() == Some(CAMERA_3D) {
                let camera_position = transform.compute_matrix();
                let projection: Mat4 = camera.projection_matrix;

                Some(camera_position * projection.inverse())
            } else {
                None
            }
        });

        if let Some(inv_camera_projection) = maybe_inv_camera_projection {
            // transform our cursor position into our normalized device coordinates
            let screen_size = Vec2::new(window.width(), window.height());
            let normalized_cursor = (cursor_position / screen_size) * 2. - Vec2::splat(1.);

            // borrowed from https://github.com/aevyrie/bevy_mod_raycast/blob/master/src/primitives.rs
            // deal with near and far to support ortho cameras
            let cursor_near_gpu = normalized_cursor.extend(-1.);
            let cursor_far_gpu = normalized_cursor.extend(1.);

            let cursor_near_world = inv_camera_projection.project_point3(cursor_near_gpu);
            let cursor_far_world = inv_camera_projection.project_point3(cursor_far_gpu);

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

            for (mut transform, _) in query.q0_mut().iter_mut() {
                transform.translation = light_location.clone();
            }
        }
    }
}
