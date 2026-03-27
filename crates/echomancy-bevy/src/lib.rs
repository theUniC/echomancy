//! Echomancy Bevy crate — top-level plugin composition.
//!
//! `EchomancyPlugin` is the single plugin added to the Bevy `App` in `main.rs`.
//! It composes sub-plugins:
//! - `GamePlugin` — domain bridge: resources, events, snapshot sync.
//! - `UiPlugin` — card rendering, battlefield display, and UI layout.

mod plugins;

use bevy::prelude::*;
use plugins::game::GamePlugin;
use plugins::ui::UiPlugin;

/// Top-level plugin that composes all Echomancy sub-plugins.
pub struct EchomancyPlugin;

impl Plugin for EchomancyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugin);
        app.add_plugins(UiPlugin);
    }
}
