use std::f32::consts::FRAC_PI_2;

use bevy::input::system::exit_on_esc_system;
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

use crate::aim_system::{aim_system, MouseLightBundle};
use crate::debug::Debug;
use crate::level::Chunk;
use crate::player::{Player};
use crate::movement::{first_person_move_system, cycle_control_system, Controllable};
use crate::view_system::{UiCam, ViewPlugin};

mod aim_system;
mod debug;
mod level;
mod player;
mod view_system;
mod movement;

#[derive(Default)]
struct Game {}

#[wasm_bindgen]
pub fn run() {
    default_plugins(&mut App::build())
        .add_system(exit_on_esc_system.system())
        .add_startup_system(setup.system())
        .add_plugin(ViewPlugin::default())
        .add_system(aim_system.system())
        .add_system(first_person_move_system.system())
        .add_system(cycle_control_system.system())
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
    // Tell the asset server to watch for asset changes on disk (I'm not sure this actually works)
    asset_server.watch_for_changes().unwrap();

    // add a ui camera
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(UiCam);

    // add some light
    commands.spawn_bundle(MouseLightBundle {
        light: LightBundle {
            transform: Transform::from_xyz(4.0, 5.0, 4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // add our character
    let character_handle = asset_server.load("models.glb#Scene0");
    commands
        .spawn_bundle((Transform::default(), GlobalTransform::identity()))
        .with_children(|tile| {
            tile.spawn_scene(character_handle.clone());
        })
        .insert(Player)
        .insert(Controllable);

    // build our map
    let grass_handle = asset_server.load("models.glb#Scene1");
    let wall_handle = asset_server.load("models.glb#Scene2");

    const WIDTH: usize = 20;
    const HEIGHT: usize = 20;
    let chunk = Chunk::<WIDTH, HEIGHT>::arena();

    let x_offset = (WIDTH / 2) as f32;
    let z_offset = (HEIGHT / 2) as f32;
    for z in 0..HEIGHT {
        for x in 0..WIDTH {
            let transform = Transform::from_xyz(x as f32 - x_offset, 0., z as f32 - z_offset);

            for (dx, dz, rotate) in [(1, 0, false), (0, 1, true)].iter() {
                let nx = x + dx;
                let nz = z + dz;
                if nx < WIDTH && nz < HEIGHT && chunk.grid[z][x] != chunk.grid[nz][nx] {
                    // todo this is gross
                    let wall_transform = if *rotate {
                        let mut wall_transform = transform;
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
