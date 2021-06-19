use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(Default)]
pub struct Debug;

impl Plugin for Debug {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_startup_system(debug_setup.system())
            .add_system(debug_system.system());
    }
}

fn debug_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // scoreboard
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

fn debug_system(mut query: Query<&mut Text>, diagnostics: Res<Diagnostics>) {
    if let Some(fps) = diagnostics.get_measurement(FrameTimeDiagnosticsPlugin::FPS) {
        let mut text = query.single_mut().unwrap();
        text.sections[1].value = format!("{:.2}", fps.value);
    }
}
