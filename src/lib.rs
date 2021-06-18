mod camera;
mod level;

use crate::camera::{camera_system, focus_camera};
use crate::level::Chunk;


use bevy::prelude::*;

use wasm_bindgen::prelude::*;

#[derive(Default)]
struct Game {}

#[wasm_bindgen]
pub fn run() {
    default_plugins(&mut App::build())
        .add_startup_system(setup.system())
        .add_system(camera_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
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

    // build our camera
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

    // add some light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
        ..Default::default()
    });

    // build our map
    let tile_handle = asset_server.load("grass-tile.glb#Scene0");

    const WIDTH: usize = 20;
    const HEIGHT: usize = 20;
    let chunk = Chunk::<WIDTH, HEIGHT>::random(&mut rand::thread_rng());

    for z in 0..HEIGHT {
        for x in 0..WIDTH {
            if chunk.grid[z][x] {
                commands
                    .spawn_bundle((
                        Transform::from_xyz(
                            x as f32 - (WIDTH / 2) as f32,
                            0.,
                            z as f32 - (HEIGHT / 2) as f32,
                        ),
                        GlobalTransform::identity(),
                    ))
                    .with_children(|tile| {
                        tile.spawn_scene(tile_handle.clone());
                    });
            }
        }
    }
}
