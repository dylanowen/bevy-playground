mod gltf;

use crate::mesh_loader::gltf::EnhancedGltf;
use bevy::ecs::system::Command;
use bevy::gltf::{Gltf, GltfMesh, GltfPrimitive};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::pipeline::PrimitiveTopology;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::math::Point;
use std::collections::HashMap;

pub struct MeshLoaderPlugin;

impl Plugin for MeshLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MeshSpawner>().add_system_to_stage(
            CoreStage::PreUpdate,
            mesh_spawner_system.exclusive_system().at_end(),
        );
    }
}

#[derive(Default)]
struct MeshSpawner {
    meshes_to_spawn: HashMap<Handle<Gltf>, Vec<SpawnGltfMeshInfo>>,
    physics_meshes: HashMap<Handle<Mesh>, ColliderShape>,
}

impl MeshSpawner {
    fn created_gltf(
        &mut self,
        handle: &Handle<Gltf>,
        gltfs: &Assets<Gltf>,
        gltf_meshes: &Assets<GltfMesh>,
        meshes: &Assets<Mesh>,
        commands: &mut Commands,
    ) {
        for SpawnGltfMeshInfo {
            mesh_name,
            derive_physics_shape,
            entity,
        } in self
            .meshes_to_spawn
            .remove(&handle)
            .unwrap_or_default()
            .into_iter()
        {
            let gltf = gltfs.get(handle).unwrap();
            let gltf_mesh = gltf.get_mesh(&mesh_name, &gltf_meshes);

            // todo can we have multiple meshes in 1 gltf mesh?
            let gltf_primitive = gltf_mesh.primitives.get(0).unwrap();

            // create a Pbr bundle and pull out the pieces we want
            let pbr = if let Some(material) = &gltf_primitive.material {
                PbrBundle {
                    mesh: gltf_primitive.mesh.clone(),
                    material: material.clone(),
                    ..Default::default()
                }
            } else {
                PbrBundle {
                    mesh: gltf_primitive.mesh.clone(),
                    ..Default::default()
                }
            };

            let mut entity_commands = commands.entity(entity);

            // todo maybe we can pull this out automatically using Bundle?
            entity_commands
                .insert(pbr.mesh)
                .insert(pbr.material)
                .insert(pbr.main_pass)
                .insert(pbr.draw)
                .insert(pbr.visible)
                .insert(pbr.render_pipelines);

            if derive_physics_shape {
                entity_commands.insert(self.derive_physics_shape(gltf_primitive, meshes));
            }
        }
    }

    fn derive_physics_shape(
        &mut self,
        gltf_primitive: &GltfPrimitive,
        meshes: &Assets<Mesh>,
    ) -> ColliderShape {
        self.physics_meshes
            .entry(gltf_primitive.mesh.clone_weak())
            .or_insert_with(|| {
                let mesh = meshes.get(&gltf_primitive.mesh).unwrap();
                log::trace!("Deriving Physics Shape");

                match mesh.primitive_topology() {
                    PrimitiveTopology::TriangleList => {
                        let vertex_position_attributes =
                            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
                        let positions = match vertex_position_attributes {
                            VertexAttributeValues::Float3(values) => values
                                .iter()
                                .map(|p| Into::<Point<_>>::into(*p))
                                .collect::<Vec<_>>(),
                            _ => panic!("Right now we only handle the Float3 vertex type"),
                        };
                        let indices = match mesh.indices().unwrap() {
                            Indices::U32(raw_indices) => raw_indices
                                .chunks(3)
                                .map(|c| [c[0], c[1], c[2]])
                                .collect::<Vec<_>>(),
                            Indices::U16(raw_indices) => raw_indices
                                .chunks(3)
                                .map(|c| [c[0] as u32, c[1] as u32, c[2] as u32])
                                .collect::<Vec<_>>(),
                        };

                        ColliderShape::trimesh(positions, indices)
                    }
                    unknown => {
                        panic!(
                            "We can't generate a ColliderShape from this topology: {:?}",
                            unknown
                        )
                    }
                }
            })
            .clone()
    }
}

fn mesh_spawner_system(
    mut spawner: ResMut<MeshSpawner>,
    mut loaded_gltf: EventReader<AssetEvent<Gltf>>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    meshes: Res<Assets<Mesh>>,
    mut commands: Commands,
) {
    for event in loaded_gltf.iter() {
        // ignore our other events, we already correctly update our models while running
        if let AssetEvent::Created { handle } = event {
            spawner.created_gltf(&handle, &gltfs, &gltf_meshes, &meshes, &mut commands);
        }
    }
}

struct SpawnGltfMeshInfo {
    mesh_name: String,
    derive_physics_shape: bool,
    entity: Entity,
}

struct SpawnGltfMesh {
    gltf_handle: Handle<Gltf>,
    info: SpawnGltfMeshInfo,
}

impl Command for SpawnGltfMesh {
    fn write(self: Box<Self>, world: &mut World) {
        let mut spawner = world.get_resource_mut::<MeshSpawner>().unwrap();

        let meshes_info = spawner
            .meshes_to_spawn
            .entry(self.gltf_handle.clone())
            .or_insert_with(Vec::new);

        meshes_info.push(self.info);
    }
}

pub trait SpawnMeshAsChildCommands {
    fn spawn_mesh<S: ToString>(
        &mut self,
        gltf_handle: Handle<Gltf>,
        mesh_name: S,
        derive_physics_mesh: bool,
    ) -> &mut Self;
}

impl<'a, 'b> SpawnMeshAsChildCommands for ChildBuilder<'a, 'b> {
    fn spawn_mesh<S: ToString>(
        &mut self,
        gltf_handle: Handle<Gltf>,
        mesh_name: S,
        derive_physics_mesh: bool,
    ) -> &mut Self {
        self.add_command(SpawnGltfMesh {
            gltf_handle,
            info: SpawnGltfMeshInfo {
                mesh_name: mesh_name.to_string(),
                derive_physics_shape: derive_physics_mesh,
                entity: self.parent_entity(),
            },
        });

        self
    }

    // fn spawn_mesh_derive_physics<S: ToString>(
    //     &mut self,
    //     gltf_handle: Handle<Gltf>,
    //     mesh_name: S,
    //     derive_physics_mesh: bool,
    // ) -> &mut Self {
    //     self.add_command(SpawnGltfMesh {
    //         gltf_handle,
    //         info: SpawnGltfMeshInfo {
    //             mesh_name: mesh_name.to_string(),
    //             derive_physics_mesh,
    //             entity: self.parent_entity(),
    //         },
    //     });
    //
    //     self
    // }
}
