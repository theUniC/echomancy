use bevy::prelude::*;
use echomancy_bevy::EchomancyPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Echomancy".into(),
                resolution: (1280_u32, 720_u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EchomancyPlugin)
        .run();
}
