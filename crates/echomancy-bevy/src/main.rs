use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Echomancy".to_owned(),
                ..default()
            }),
            ..default()
        }))
        .run();
}
