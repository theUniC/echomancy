//! UiPlugin — composes all Echomancy UI sub-plugins.
//!
//! Current sub-plugins:
//! - `BattlefieldPlugin` — renders player and opponent battlefields (Phase 8.2)
//! - `HandPlugin` — hand display + land play (Phase 8.3)
//!
//! Planned:
//! - `HudPlugin` — turn info, life totals, buttons (Phase 8.4)
//! - `ErrorPlugin` — error messages (Phase 8.5)

pub(crate) mod battlefield;
pub(crate) mod card;
pub(crate) mod hand;

use bevy::prelude::*;
use battlefield::BattlefieldPlugin;
use hand::HandPlugin;

/// Top-level UI plugin composed of all Echomancy UI sub-plugins.
pub(crate) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((BattlefieldPlugin, HandPlugin));
    }
}
