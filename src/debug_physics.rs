use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

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
    mut visibility_query: Query<(&mut Visible, &Handle<StandardMaterial>)>,
    mut collider_shapes: Query<Entity, With<ColliderShape>>,
    // mut contact_events: EventReader<ContactEvent>,
    // mut position_query: Query<(
    //     Option<&String>,
    //     &ColliderPosition,
    //     Option<&RigidBodyPosition>,
    //     &GlobalTransform,
    // )>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Semicolon) {
        log::warn!("Enabling physics debug mode");

        // set our transparency so we can see our collider meshes
        for (mut visible, material_handle) in visibility_query.iter_mut() {
            visible.is_transparent = true;
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color.set_a(0.5);
            }
        }

        for (i, entity) in collider_shapes.iter_mut().enumerate() {
            commands
                .entity(entity)
                .remove_bundle::<PbrBundle>()
                .insert(ColliderDebugRender::with_id(i))
                // if we've started debugging we'll have lost our original transform so explicitly sync it
                .insert(ColliderPositionSync::Discrete);
        }
    }
    if keyboard_input.just_pressed(KeyCode::P) {
        rapier_config.physics_pipeline_active = !rapier_config.physics_pipeline_active;
        rapier_config.query_pipeline_active = !rapier_config.query_pipeline_active;
    }

    // log::trace!("Debug Physics");
    // for contact_event in contact_events.iter() {
    //     println!("{:?}", contact_event);
    //     log::trace!("{:?}", contact_event);
    // }

    // for (debug_name, collider_position, rigidbody_position, position) in position_query.iter() {
    //     println!(
    //         "{:?} collider: {:?} rigidbody: {:?} transform: {}",
    //         debug_name,
    //         collider_position.translation.vector,
    //         rigidbody_position.map(|p| p.position.translation.vector),
    //         position.translation
    //     );
    //     log::trace!(
    //         "collider: {:?} rigidbody: {:?} transform: {:?}",
    //         collider_position.translation.vector,
    //         rigidbody_position.map(|p| p.position.translation.vector),
    //         position.translation
    //     );
    // }
    // log::trace!("");
}
