use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::mem;

pub struct DebugPhysicsPlugin;

impl Plugin for DebugPhysicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(RapierRenderPlugin)
            .add_system(debug_physics_system.system());
    }
}

fn debug_physics_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Visible, &Handle<StandardMaterial>)>,
    mut collider_shapes: Query<Entity, With<ColliderShape>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Semicolon) {
        log::warn!("Enabling physics debug mode");

        // set our transparency so we can see our collider meshes
        for (mut visible, material_handle) in query.iter_mut() {
            visible.is_transparent = true;
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color.set_a(0.3);
            }
        }

        for (i, entity) in collider_shapes.iter_mut().enumerate() {
            commands
                .entity(entity)
                .remove_bundle::<PbrBundle>()
                .insert(ColliderDebugRender::with_id(i))
                // if we've started debugging we'll have lost our original transform so explicitly sync it
                .insert(RigidBodyPositionSync::Discrete);
        }
    }
    if keyboard_input.just_pressed(KeyCode::P) {
        rapier_config.physics_pipeline_active = !rapier_config.physics_pipeline_active;
        rapier_config.query_pipeline_active = !rapier_config.query_pipeline_active;
    }
}
