use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;

pub trait EnhancedGltf {
    fn get_mesh<'a>(&self, name: &str, gltf_meshes: &'a Assets<GltfMesh>) -> &'a GltfMesh;
}

impl EnhancedGltf for Gltf {
    fn get_mesh<'a>(&self, name: &str, gltf_meshes: &'a Assets<GltfMesh>) -> &'a GltfMesh {
        if let Some(mesh_handle) = self.named_meshes.get(name) {
            let gltf_mesh = gltf_meshes.get(mesh_handle).unwrap();

            gltf_mesh
        } else {
            panic!(
                "Couldn't find Mesh \"{}\", available meshes are {:?}",
                name,
                self.named_meshes.keys()
            )
        }
    }
}
