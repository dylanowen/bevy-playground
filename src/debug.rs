use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct Debug;

impl Plugin for Debug {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            // .insert_resource(WireframeConfig { global: true })
            // .add_plugin(WireframePlugin)
            .add_startup_system(debug_setup.system())
            .add_startup_system(debug_lines_setup.system())
            .add_system(debug_system.system());
    }
}

fn debug_system(mut query: Query<&mut Text>, diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get_measurement(FrameTimeDiagnosticsPlugin::FPS) {
        let mut text = query.single_mut().unwrap();
        text.sections[1].value = format!("{:.2}", fps.value);
    }
}

fn debug_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // FPS View
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "FPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: String::new(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::GOLD,
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

#[cfg(target_arch = "wasm32")]
fn debug_lines_setup() {}

#[cfg(not(target_arch = "wasm32"))]
fn debug_lines_setup(mut debug_lines: ResMut<bevy_prototype_debug_lines::DebugLines>) {
    // x y z lines
    debug_lines
        .user_lines
        .push(bevy_prototype_debug_lines::Line::new(
            Vec3::splat(0.),
            Vec3::X * 10.,
            0.,
            Color::RED,
            Color::RED,
        ));
    debug_lines
        .user_lines
        .push(bevy_prototype_debug_lines::Line::new(
            Vec3::splat(0.),
            Vec3::Y * 10.,
            0.,
            Color::GREEN,
            Color::GREEN,
        ));
    debug_lines
        .user_lines
        .push(bevy_prototype_debug_lines::Line::new(
            Vec3::splat(0.),
            Vec3::Z * 10.,
            0.,
            Color::BLUE,
            Color::BLUE,
        ));
}
