//! Echomancy Bevy crate — top-level plugin composition.
//!
//! `EchomancyPlugin` is the single plugin added to the Bevy `App` in `main.rs`.
//! It composes sub-plugins:
//! - `GamePlugin` — domain bridge: resources, events, snapshot sync.

mod plugins;

use bevy::prelude::*;
use plugins::game::GamePlugin;

/// Top-level plugin that composes all Echomancy sub-plugins.
pub struct EchomancyPlugin;

impl Plugin for EchomancyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugin);
        // Future: app.add_plugins(UiPlugin);
    }
}
