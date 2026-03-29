//! Bevy systems and plugin registration for the game bridge.
//!
//! - `setup_game` / `setup_camera`: startup systems
//! - `send_initial_snapshot_message`: post-startup notification
//! - `handle_game_actions`: update loop — drains actions, mutates domain, refreshes snapshot
//! - `GamePlugin`: wires everything into the Bevy app

use bevy::prelude::*;
use echomancy_core::domain::game::automation::{
    auto_advance_through_non_interactive, auto_advance_to_main_phase, auto_resolve_stack,
};
use echomancy_core::prelude::*;
use uuid::Uuid;

use super::{
    ActivePlayerId, CurrentSnapshot, ErrorMessage, GameActionMessage, GameState, PlayableCards,
    PlayerIds, PlayerInfo, SnapshotChangedMessage, TargetSelectionState,
};
use super::snapshot::{compute_snapshot, humanize_error, resolve_ui_player_id};

// ============================================================================
// Startup systems
// ============================================================================

/// Startup system: create a 2-player game, assign prebuilt decks, start it,
/// and compute the initial snapshot. Inserts all resources.
pub(crate) fn setup_game(mut commands: Commands) {
    let p1_id = Uuid::new_v4().to_string();
    let p2_id = Uuid::new_v4().to_string();

    let p1_name = "Player 1".to_owned();
    let p2_name = "Player 2".to_owned();

    let mut game = Game::create(Uuid::new_v4().to_string());
    game.add_player(&p1_id, &p1_name).expect("add player 1");
    game.add_player(&p2_id, &p2_name).expect("add player 2");

    game.assign_deck(&p1_id, prebuilt_decks::green_deck(&p1_id))
        .expect("assign green deck");
    game.assign_deck(&p2_id, prebuilt_decks::red_deck(&p2_id))
        .expect("assign red deck");

    // Use OS entropy for shuffling (non-deterministic, as expected in production).
    game.start(&p1_id, None).expect("start game");

    // Wire up the CLIPS rules engine so spells have real effects.
    // Load rules for all card types that appear in either deck.
    let card_ids = ["lightning-strike", "giant-growth", "divination",
                    "bear", "goblin", "forest", "mountain"];
    match create_rules_engine(&card_ids) {
        Ok(engine) => {
            game.set_rules_engine(engine);
            info!("CLIPS rules engine loaded");
        }
        Err(err) => {
            error!(%err, "Failed to create CLIPS rules engine — spells will have no effects");
        }
    }

    // Auto-advance through Untap → Upkeep → Draw → FirstMain so the player
    // immediately sees playable lands on startup.
    auto_advance_to_main_phase(&mut game, &p1_id);

    let (snapshot, playable_cards) =
        compute_snapshot(&game, &p1_id).expect("initial snapshot");

    info!(
        player1_id = %p1_id,
        player2_id = %p2_id,
        turn = snapshot.public_game_state.turn_number,
        step = ?snapshot.public_game_state.current_step,
        "Game created and started"
    );

    commands.insert_resource(PlayerIds {
        p1: PlayerInfo { id: p1_id.clone(), name: p1_name },
        p2: PlayerInfo { id: p2_id.clone(), name: p2_name },
    });
    commands.insert_resource(ActivePlayerId {
        player_id: p1_id.clone(),
    });
    commands.insert_resource(GameState { game });
    commands.insert_resource(CurrentSnapshot { snapshot });
    commands.insert_resource(PlayableCards {
        result: playable_cards,
    });
    commands.insert_resource(ErrorMessage::default());
    commands.insert_resource(TargetSelectionState::default());
}

/// One-shot system that fires after startup to notify UI of initial state.
pub(crate) fn send_initial_snapshot_message(
    mut snapshot_changed: MessageWriter<SnapshotChangedMessage>,
) {
    snapshot_changed.write(SnapshotChangedMessage);
}

/// Startup system: spawn a 2D camera so Bevy renders the window.
pub(crate) fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ============================================================================
// Update system
// ============================================================================

/// Update system: drain `GameActionMessage`s, apply each to the domain game,
/// recompute the snapshot, and send `SnapshotChangedMessage`.
///
/// After each action, checks whether the active player has changed (e.g.,
/// after `EndTurn`) and updates `ActivePlayerId` accordingly so the UI always
/// shows the perspective of the player whose turn it currently is.
///
/// On success, clears `ErrorMessage`. On failure, stores the error string in
/// `ErrorMessage` so the HUD can display it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_game_actions(
    mut game_state: ResMut<GameState>,
    mut active_player: ResMut<ActivePlayerId>,
    mut action_messages: MessageReader<GameActionMessage>,
    mut snapshot_res: ResMut<CurrentSnapshot>,
    mut playable_res: ResMut<PlayableCards>,
    mut snapshot_changed: MessageWriter<SnapshotChangedMessage>,
    mut error_message: ResMut<ErrorMessage>,
    player_ids: Res<PlayerIds>,
) {
    let mut any_applied = false;

    for message in action_messages.read() {
        match game_state.game.apply(message.0.clone()) {
            Ok(events) => {
                debug!(event_count = events.len(), "Game action applied");
                any_applied = true;
                // Clear any previous error on success.
                error_message.message = None;
            }
            Err(err) => {
                warn!(%err, "GameActionMessage rejected by domain");
                error_message.message = Some(humanize_error(&err.to_string(), &player_ids));
            }
        }
    }

    if any_applied {
        // Auto-resolve the stack when no player can respond.
        // In the MVP, neither player has counterspells or instant-speed responses,
        // so we auto-pass priority for both players until the stack empties.
        auto_resolve_stack(&mut game_state.game);

        // Auto-advance through non-interactive steps on every action, not just
        // on perspective changes. This handles the case where the active player
        // clicks "Pass Priority" from FirstMain and the engine enters
        // BeginningOfCombat, which must be auto-skipped to reach DeclareAttackers.
        // Combat damage (CombatDamage step) is auto-calculated by the engine's
        // on_enter_step hook; EndOfCombat, EndStep, Cleanup are also non-interactive.
        let current_player_for_advance = game_state.game.current_player_id().to_owned();
        auto_advance_through_non_interactive(&mut game_state.game, &current_player_for_advance);

        // Determine which player's perspective the UI should show.
        // During DeclareBlockers the defending player (priority holder) drives the UI.
        // Otherwise the active (current) player drives it.
        let new_ui_player = resolve_ui_player_id(
            game_state.game.priority_player_id(),
            game_state.game.current_player_id(),
            game_state.game.current_step(),
        )
        .to_owned();

        let perspective_changed = new_ui_player != active_player.player_id;
        if perspective_changed {
            info!(
                old = %active_player.player_id,
                new = %new_ui_player,
                "UI perspective switched to new active player"
            );
            active_player.player_id = new_ui_player.clone();

            // When a new player takes over the UI (e.g. after EndTurn or after
            // P1's cleanup wraps to P2's Untap), auto-advance through any
            // remaining non-interactive steps so P2 starts in an interactive step.
            auto_advance_through_non_interactive(&mut game_state.game, &new_ui_player);
        }

        match compute_snapshot(&game_state.game, &active_player.player_id) {
            Ok((snapshot, playable_cards)) => {
                *snapshot_res = CurrentSnapshot { snapshot };
                *playable_res = PlayableCards {
                    result: playable_cards,
                };
                snapshot_changed.write(SnapshotChangedMessage);
            }
            Err(err) => {
                error!(%err, "Failed to compute snapshot after action");
            }
        }
    }
}

/// Update system: send `SnapshotChangedMessage` when `TargetSelectionState` changes.
///
/// The battlefield and hand systems rebuild on snapshot-changed messages.
/// When the player enters or exits target-selection mode, we broadcast this
/// message so those systems immediately update their highlights and interactivity.
pub(crate) fn notify_on_target_selection_change(
    target_selection: Res<TargetSelectionState>,
    mut snapshot_changed: MessageWriter<SnapshotChangedMessage>,
) {
    if target_selection.is_changed() {
        snapshot_changed.write(SnapshotChangedMessage);
    }
}

// ============================================================================
// Plugin
// ============================================================================

/// Registers all game-related resources, messages, and systems.
pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GameActionMessage>()
            .add_message::<SnapshotChangedMessage>()
            .add_systems(Startup, (setup_camera, setup_game))
            .add_systems(PostStartup, send_initial_snapshot_message)
            .add_systems(Update, (handle_game_actions, notify_on_target_selection_change));
    }
}
