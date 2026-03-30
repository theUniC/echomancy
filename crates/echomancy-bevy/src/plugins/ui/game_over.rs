//! GameOverPlugin — full-screen overlay displayed when the game ends.
//!
//! When `PublicGameState.lifecycle_state` transitions to `Finished`, this
//! plugin spawns a semi-transparent overlay with centered text:
//!
//! - "YOU WIN!" in green  — viewer is the winner
//! - "YOU LOSE!" in red   — viewer lost
//! - "DRAW!" in gray      — simultaneous loss / draw
//!
//! The overlay is tagged with `GameOverOverlay` so it can be despawned and
//! re-created if a "New Game" feature is added later.
//!
//! All action buttons in the HUD are effectively disabled once the game is over
//! because `has_priority` returns `false` (no priority holder after game ends).
//! The HUD's `update_hud` system already handles that path.

use bevy::prelude::*;
use echomancy_core::prelude::{GameLifecycleState, GameOutcomeExport};

use crate::plugins::game::{HumanPlayerId, CurrentSnapshot, SnapshotChangedMessage};

// ============================================================================
// Constants
// ============================================================================

/// Background color for the semi-transparent overlay.
const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.75);

/// Text color for a win result.
const WIN_COLOR: Color = Color::srgb(0.20, 0.85, 0.30);

/// Text color for a loss result.
const LOSE_COLOR: Color = Color::srgb(0.90, 0.15, 0.15);

/// Text color for a draw result.
const DRAW_COLOR: Color = Color::srgb(0.70, 0.70, 0.70);

// ============================================================================
// Marker components
// ============================================================================

/// Tags the root overlay node so it can be despawned when starting a new game.
#[derive(Component)]
pub(crate) struct GameOverOverlay;

// ============================================================================
// Pure helper: derive result text and color
// ============================================================================

/// The result to display on the game-over overlay.
#[derive(Debug, Clone)]
pub(crate) struct GameOverDisplay {
    pub(crate) label: &'static str,
    pub(crate) color: Color,
}

/// Derive the display text and color from the outcome and viewer perspective.
///
/// - Win: viewer ID matches the winner ID → "YOU WIN!" green
/// - Win: viewer ID does not match         → "YOU LOSE!" red
/// - Draw: any draw outcome                → "DRAW!" gray
///
/// Returns `None` when the game has not yet finished.
pub(crate) fn game_over_display(
    lifecycle: GameLifecycleState,
    outcome: Option<&GameOutcomeExport>,
    viewer_player_id: &str,
) -> Option<GameOverDisplay> {
    if lifecycle != GameLifecycleState::Finished {
        return None;
    }

    let display = match outcome {
        Some(GameOutcomeExport::Win(win)) => {
            if win.winner_id == viewer_player_id {
                GameOverDisplay { label: "YOU WIN!", color: WIN_COLOR }
            } else {
                GameOverDisplay { label: "YOU LOSE!", color: LOSE_COLOR }
            }
        }
        Some(GameOutcomeExport::Draw(_)) | None => {
            GameOverDisplay { label: "DRAW!", color: DRAW_COLOR }
        }
    };

    Some(display)
}

// ============================================================================
// Systems
// ============================================================================

/// Spawn the game-over overlay when the snapshot reports `Finished` lifecycle.
///
/// Only runs when a `SnapshotChangedMessage` is received and the game just
/// finished.  De-duplicated: if an overlay already exists, nothing is added.
pub(crate) fn spawn_game_over_overlay(
    current_snapshot: Res<CurrentSnapshot>,
    active_player: Res<HumanPlayerId>,
    mut snapshot_changed: MessageReader<SnapshotChangedMessage>,
    existing_overlay: Query<(), With<GameOverOverlay>>,
    mut commands: Commands,
) {
    if snapshot_changed.read().count() == 0 {
        return;
    }

    let pub_state = &current_snapshot.snapshot.public_game_state;

    // Only act when the game is finished.
    if pub_state.lifecycle_state != GameLifecycleState::Finished {
        return;
    }

    // Do not spawn a second overlay if one already exists.
    if !existing_overlay.is_empty() {
        return;
    }

    let Some(display) = game_over_display(
        pub_state.lifecycle_state,
        pub_state.game_outcome.as_ref(),
        &active_player.player_id,
    ) else {
        return;
    };

    commands
        .spawn((
            GameOverOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(OVERLAY_BG),
            ZIndex(100),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new(display.label),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(display.color),
            ));
        });
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers game-over overlay systems.
pub(crate) struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_game_over_overlay);
    }
}

// ============================================================================
// Tests (TDD: written before implementation)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- game_over_display: game not finished --------------------------------

    #[test]
    fn no_display_when_game_is_started() {
        let result = game_over_display(GameLifecycleState::Started, None, "player-1");
        assert!(result.is_none(), "Should return None when game is still in progress");
    }

    #[test]
    fn no_display_when_game_is_created() {
        let result = game_over_display(GameLifecycleState::Created, None, "player-1");
        assert!(result.is_none(), "Should return None when game has not started");
    }

    // ---- game_over_display: viewer wins -------------------------------------

    #[test]
    fn win_display_when_viewer_is_winner() {
        use echomancy_core::prelude::WinOutcomeExport;
        let outcome = GameOutcomeExport::Win(WinOutcomeExport {
            winner_id: "player-1".to_owned(),
            reason: "LifeTotal".to_owned(),
        });
        let result =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "player-1").unwrap();
        assert_eq!(result.label, "YOU WIN!");
        assert_eq!(result.color, WIN_COLOR);
    }

    #[test]
    fn win_label_is_you_win_when_viewer_is_the_winner() {
        use echomancy_core::prelude::WinOutcomeExport;
        let outcome = GameOutcomeExport::Win(WinOutcomeExport {
            winner_id: "alice".to_owned(),
            reason: "EmptyLibrary".to_owned(),
        });
        let display =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "alice").unwrap();
        assert_eq!(display.label, "YOU WIN!");
    }

    // ---- game_over_display: viewer loses ------------------------------------

    #[test]
    fn lose_display_when_viewer_is_not_winner() {
        use echomancy_core::prelude::WinOutcomeExport;
        let outcome = GameOutcomeExport::Win(WinOutcomeExport {
            winner_id: "player-2".to_owned(),
            reason: "LifeTotal".to_owned(),
        });
        let result =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "player-1").unwrap();
        assert_eq!(result.label, "YOU LOSE!");
        assert_eq!(result.color, LOSE_COLOR);
    }

    #[test]
    fn lose_label_is_you_lose_when_different_player_won() {
        use echomancy_core::prelude::WinOutcomeExport;
        let outcome = GameOutcomeExport::Win(WinOutcomeExport {
            winner_id: "bob".to_owned(),
            reason: "LifeTotal".to_owned(),
        });
        let display =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "alice").unwrap();
        assert_eq!(display.label, "YOU LOSE!");
    }

    // ---- game_over_display: draw --------------------------------------------

    #[test]
    fn draw_display_when_outcome_is_draw() {
        use echomancy_core::prelude::DrawOutcomeExport;
        let outcome = GameOutcomeExport::Draw(DrawOutcomeExport {
            reason: "SimultaneousLoss".to_owned(),
        });
        let result =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "player-1").unwrap();
        assert_eq!(result.label, "DRAW!");
        assert_eq!(result.color, DRAW_COLOR);
    }

    #[test]
    fn draw_display_when_outcome_is_none_but_finished() {
        // Edge case: game finished but no outcome recorded (shouldn't happen in practice,
        // but handled gracefully as a draw).
        let result = game_over_display(GameLifecycleState::Finished, None, "player-1").unwrap();
        assert_eq!(result.label, "DRAW!");
        assert_eq!(result.color, DRAW_COLOR);
    }

    // ---- game_over_display: label correctness -------------------------------

    #[test]
    fn win_and_lose_labels_are_distinct() {
        use echomancy_core::prelude::WinOutcomeExport;
        let outcome = GameOutcomeExport::Win(WinOutcomeExport {
            winner_id: "winner".to_owned(),
            reason: "LifeTotal".to_owned(),
        });
        let win_display =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "winner").unwrap();
        let lose_display =
            game_over_display(GameLifecycleState::Finished, Some(&outcome), "loser").unwrap();

        assert_ne!(win_display.label, lose_display.label);
        assert_ne!(win_display.color, lose_display.color);
    }

    #[test]
    fn colors_are_different_for_win_lose_draw() {
        assert_ne!(WIN_COLOR, LOSE_COLOR);
        assert_ne!(WIN_COLOR, DRAW_COLOR);
        assert_ne!(LOSE_COLOR, DRAW_COLOR);
    }
}
