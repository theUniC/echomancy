//! UiPlugin — composes all Echomancy UI sub-plugins.
//!
//! Current sub-plugins:
//! - `MulliganPlugin` — mulligan screen (runs in AppState::Mulligan)
//! - `BattlefieldPlugin` — renders player and opponent battlefields (Phase 8.2)
//! - `HandPlugin` — hand display + land play (Phase 8.3)
//! - `HudPlugin` — turn info, life totals, priority, buttons (Phase 8.4)
//! - `GameOverPlugin` — full-screen overlay when game ends (Phase F)
//! - `StackPlugin` — center overlay showing the game stack (U1)

pub(crate) mod battlefield;
pub(crate) mod card;
pub(crate) mod game_over;
pub(crate) mod hand;
pub(crate) mod hud;
pub(crate) mod mulligan;
pub(crate) mod stack;

use bevy::prelude::*;
use battlefield::BattlefieldPlugin;
use game_over::GameOverPlugin;
use hand::HandPlugin;
use hud::HudPlugin;
use mulligan::MulliganPlugin;
use stack::StackPlugin;

/// Top-level UI plugin composed of all Echomancy UI sub-plugins.
pub(crate) struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MulliganPlugin, BattlefieldPlugin, HandPlugin, HudPlugin, GameOverPlugin, StackPlugin));
    }
}
