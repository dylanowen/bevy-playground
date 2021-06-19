mod camera;
mod debug;
mod level;

use crate::camera::{camera_system, focus_camera};
use crate::level::Chunk;

use bevy::prelude::*;

use crate::debug::Debug;
use std::f32::consts::FRAC_PI_2;
use wasm_bindgen::prelude::*;

#[derive(Default)]
struct Game {}

#[wasm_bindgen]
pub fn run() {
    default_plugins(&mut App::build())
        .add_startup_system(setup.system())
        .add_system(camera_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
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
    builder.add_plugins(DefaultPlugins)
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
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
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
