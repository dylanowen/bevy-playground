use std::f32::consts::FRAC_PI_2;

use bevy::input::system::exit_on_esc_system;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use wasm_bindgen::prelude::*;

use crate::aim_system::{aim_system, MouseLightBundle};
use crate::debug::Debug;
use crate::debug_physics::DebugPhysicsPlugin;
use crate::level::Chunk;
use crate::mesh_loader::{MeshLoaderPlugin, SpawnMeshAsChildCommands};
use crate::movement::MovePlugin;
use crate::player::{Player, PlayerControlled};
use crate::view_system::{UiCam, ViewPlugin};
use bevy_rapier3d::physics::TimestepMode;

mod aim_system;
mod debug;
mod debug_physics;
mod level;
mod mesh_loader;
mod movement;
mod player;
mod view_system;

#[derive(Default)]
struct Game {}

#[wasm_bindgen]
pub fn run() {
    default_plugins(&mut App::build())
        .add_system(exit_on_esc_system.system())
        .add_startup_system(setup.system())
        .add_plugin(ViewPlugin)
        .add_plugin(MovePlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(MeshLoaderPlugin)
        .add_system(aim_system.system())
        // diagnostics
        .add_plugin(Debug)
        .add_plugin(DebugPhysicsPlugin)
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

    let gltf_handle = asset_server.load("models.gltf");

    // add our character

    // let character_handle = asset_server.load("models.gltf#Mesh0");
    commands
        .spawn_bundle((Transform::default(), GlobalTransform::identity()))
        .insert("Character".to_string())
        .with_children(|builder| {
            builder.spawn_mesh(gltf_handle.clone(), "character", true);
        })
        .insert(Player)
        .insert(PlayerControlled)
        .insert_bundle(RigidBodyBundle {
            activation: RigidBodyActivation {
                sleeping: false,
                ..Default::default()
            },
            body_type: RigidBodyType::Dynamic,
            position: Vec3::new(0.0, 5.0, 0.0).into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::ball(0.5), //i dunno what shape lol
            collider_type: ColliderType::Solid,
            position: Transform::default().translation.into(),
            flags: (ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS).into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete);

    // build our map
    const WIDTH: usize = 20;
    const HEIGHT: usize = 20;
    let chunk = Chunk::<WIDTH, HEIGHT>::arena();

    let x_offset = (WIDTH / 2) as f32;
    let z_offset = (HEIGHT / 2) as f32;
    for z in 0..HEIGHT {
        for x in 0..WIDTH {
            let position = Vector::new(x as f32 - x_offset, 0., z as f32 - z_offset);

            for (dx, dz, rotate) in [(1, 0, false), (0, 1, true)].iter() {
                let nx = x + dx;
                let nz = z + dz;
                if nx < WIDTH && nz < HEIGHT && chunk.grid[z][x] != chunk.grid[nz][nx] {
                    // todo this is gross
                    let wall_transform: Isometry<Real> = if *rotate {
                        Isometry::new(position, Vector::y() * FRAC_PI_2).into()
                    } else {
                        position.into()
                    };

                    // todo why don't our walls work?
                    commands
                        .spawn_bundle((
                            Transform {
                                translation: wall_transform.translation.into(),
                                rotation: wall_transform.rotation.into(),
                                ..Default::default()
                            },
                            GlobalTransform::identity(),
                        ))
                        .insert("Wall".to_string())
                        .with_children(|builder| {
                            builder.spawn_mesh(gltf_handle.clone(), "wall", true);
                        })
                        // .insert_bundle(RigidBodyBundle {
                        //     body_type: RigidBodyType::Static,
                        //     // activation: RigidBodyActivation {
                        //     //     sleeping: false,
                        //     //     ..Default::default()
                        //     // },
                        //     position: wall_transform.into(),
                        //     ..Default::default()
                        // })
                        .insert_bundle(ColliderBundle {
                            shape: ColliderShape::ball(0.5), // give ourselves a dummy shape while we derive from our mesh
                            collider_type: ColliderType::Solid,
                            position: wall_transform.into(),
                            ..Default::default()
                        });
                }
            }
            if chunk.grid[z][x] {
                commands
                    .spawn_bundle((
                        Transform::from_translation(position.into()),
                        GlobalTransform::identity(),
                    ))
                    .insert("Floor".to_string())
                    .with_children(|builder| {
                        builder.spawn_mesh(gltf_handle.clone(), "grass", false);
                    });
            }
        }
    }

    commands.spawn_bundle(ColliderBundle {
        shape: ColliderShape::cuboid(WIDTH as f32, 0.1, HEIGHT as f32),
        // todo use a height field ?
        // shape: ColliderShape::heightfield(
        //     DMatrix::from_vec(WIDTH, HEIGHT, vec![0.; WIDTH * HEIGHT]),
        //     Vector::new(1., 1., 1.),
        // ),
        ..Default::default()
    });
}
