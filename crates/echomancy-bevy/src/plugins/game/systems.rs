//! Bevy systems and plugin registration for the game bridge.
//!
//! - `setup_game` / `setup_camera`: startup systems
//! - `send_initial_snapshot_message`: post-startup notification
//! - `handle_game_actions`: update loop — drains actions, mutates domain, refreshes snapshot
//! - `GamePlugin`: wires everything into the Bevy app

use bevy::prelude::*;
use echomancy_core::domain::game::automation::run_auto_pass_loop;
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

    // Auto-advance to FirstMain using the same loop the game uses after every action.
    run_auto_pass_loop(&mut game);

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
    let mut pending_error: Option<String> = None;

    for message in action_messages.read() {
        info!(action = ?message.0, "Received GameActionMessage");
        match game_state.game.apply(message.0.clone()) {
            Ok(events) => {
                info!(
                    event_count = events.len(),
                    step = ?game_state.game.current_step(),
                    player = %game_state.game.current_player_id(),
                    priority = ?game_state.game.priority_player_id(),
                    "Game action applied"
                );
                any_applied = true;
                // Clear any previous error and any error from earlier in this batch.
                pending_error = None;
            }
            Err(err) => {
                warn!(%err, "GameActionMessage rejected by domain");
                // Only record the error if no action has succeeded yet in this frame.
                // If a prior action in the same batch succeeded, the error is spurious
                // (e.g. a stale or duplicate message) and should not be shown.
                if !any_applied {
                    pending_error = Some(humanize_error(&err.to_string(), &player_ids));
                }
            }
        }
    }

    // Apply the error state: show the error only if no action succeeded this frame.
    // If any action succeeded, clear the previous error instead.
    if any_applied {
        error_message.message = None;
    } else if let Some(err_msg) = pending_error {
        error_message.message = Some(err_msg);
    }

    if any_applied {
        // Auto-pass loop
        let auto_count = run_auto_pass_loop(&mut game_state.game);
        info!(
            auto_passes = auto_count,
            step = ?game_state.game.current_step(),
            player = %game_state.game.current_player_id(),
            priority = ?game_state.game.priority_player_id(),
            "After auto-pass loop"
        );

        // Determine which player's perspective the UI should show.
        // Show whoever has priority — they need to see their hand to act.
        let new_ui_player = resolve_ui_player_id(
            game_state.game.priority_player_id(),
            game_state.game.current_player_id(),
            game_state.game.current_step(),
        )
        .to_owned();

        if new_ui_player != active_player.player_id {
            info!(
                old = %active_player.player_id,
                new = %new_ui_player,
                "UI perspective switched"
            );
            active_player.player_id = new_ui_player.clone();
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

// ============================================================================
// Tests — real Bevy App integration tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::plugins::game::{
        CurrentSnapshot, ErrorMessage, GameState, PlayableCards, PlayerIds,
        PlayerInfo, TargetSelectionState, ActivePlayerId, GameActionMessage,
        SnapshotChangedMessage,
    };
    use crate::plugins::game::snapshot::compute_snapshot;

    /// Helper: build a minimal Bevy App with game resources + handle_game_actions system.
    fn make_test_app() -> (App, String, String) {
        let p1 = "p1".to_string();
        let p2 = "p2".to_string();

        let mut game = Game::create("test");
        game.add_player(&p1, "Player 1").unwrap();
        game.add_player(&p2, "Player 2").unwrap();
        game.assign_deck(&p1, prebuilt_decks::green_deck(&p1)).unwrap();
        game.assign_deck(&p2, prebuilt_decks::red_deck(&p2)).unwrap();
        game.start(&p1, Some(42)).unwrap();

        if let Ok(engine) = create_rules_engine(&[
            "lightning-strike", "bear", "goblin", "forest", "mountain", "giant-growth",
        ]) {
            game.set_rules_engine(engine);
        }

        // Auto-advance to FirstMain
        run_auto_pass_loop(&mut game);
        assert_eq!(game.current_step(), Step::FirstMain);

        let (snapshot, playable) = compute_snapshot(&game, &p1).unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<GameActionMessage>();
        app.add_message::<SnapshotChangedMessage>();
        app.insert_resource(GameState { game });
        app.insert_resource(CurrentSnapshot { snapshot });
        app.insert_resource(PlayableCards { result: playable });
        app.insert_resource(ActivePlayerId { player_id: p1.clone() });
        app.insert_resource(ErrorMessage::default());
        app.insert_resource(TargetSelectionState::default());
        app.insert_resource(PlayerIds {
            p1: PlayerInfo { id: p1.clone(), name: "Player 1".to_string() },
            p2: PlayerInfo { id: p2.clone(), name: "Player 2".to_string() },
        });

        app.add_systems(Update, handle_game_actions);

        (app, p1, p2)
    }

    /// Helper: apply action directly to the Game resource, run auto-pass,
    /// update perspective — exactly what handle_game_actions does.
    fn send_action_and_update(app: &mut App, action: Action) {
        // Apply action + run auto-pass (same as handle_game_actions)
        {
            let world = app.world_mut();
            let mut game_state = world.resource_mut::<GameState>();
            game_state.game.apply(action).expect("action should succeed");
            run_auto_pass_loop(&mut game_state.game);
        }

        // Update perspective (same as handle_game_actions)
        {
            let world = app.world_mut();
            let game = &world.resource::<GameState>().game;
            let new_perspective = resolve_ui_player_id(
                game.priority_player_id(),
                game.current_player_id(),
                game.current_step(),
            ).to_owned();
            world.resource_mut::<ActivePlayerId>().player_id = new_perspective;
        }

        // Recompute snapshot
        {
            let world = app.world_mut();
            let game = &world.resource::<GameState>().game;
            let active = world.resource::<ActivePlayerId>().player_id.clone();
            if let Ok((snapshot, playable)) = compute_snapshot(game, &active) {
                world.resource_mut::<CurrentSnapshot>().snapshot = snapshot;
                world.resource_mut::<PlayableCards>().result = playable;
            }
        }
    }

    #[test]
    fn play_land_keeps_p1_perspective_and_first_main() {
        let (mut app, p1, _p2) = make_test_app();

        // Find a Forest in P1's hand
        let forest_id = {
            let game = &app.world().resource::<GameState>().game;
            game.hand(&p1).unwrap().iter()
                .find(|c| c.definition().id() == "forest")
                .map(|c| c.instance_id().to_owned())
                .expect("P1 should have Forest")
        };

        send_action_and_update(&mut app, Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&forest_id),
        });

        let game = &app.world().resource::<GameState>().game;
        let active = &app.world().resource::<ActivePlayerId>().player_id;

        assert_eq!(game.current_step(), Step::FirstMain, "Should still be FirstMain");
        assert_eq!(game.current_player_id(), p1.as_str(), "P1's turn");
        assert_eq!(active, &p1, "Perspective should be P1");
    }

    #[test]
    fn cast_creature_resolves_keeps_first_main() {
        let (mut app, p1, _p2) = make_test_app();

        // Give P1 mana
        {
            let game = &mut app.world_mut().resource_mut::<GameState>().game;
            game.add_mana(&p1, ManaColor::Green, 1).unwrap();
            game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
        }

        let bear_id = {
            let game = &app.world().resource::<GameState>().game;
            game.hand(&p1).unwrap().iter()
                .find(|c| c.definition().id() == "bear")
                .map(|c| c.instance_id().to_owned())
                .expect("P1 should have Bear")
        };

        send_action_and_update(&mut app, Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&bear_id),
            targets: vec![],
        });

        let game = &app.world().resource::<GameState>().game;
        let active = &app.world().resource::<ActivePlayerId>().player_id;

        assert!(!game.stack_has_items(), "Stack should be empty");
        assert_eq!(game.current_step(), Step::FirstMain);
        assert_eq!(active, &p1, "Perspective should still be P1");
        assert!(game.battlefield(&p1).unwrap().iter().any(|c| c.instance_id() == bear_id),
            "Bear should be on battlefield");
    }

    #[test]
    fn end_turn_switches_to_p2() {
        let (mut app, p1, p2) = make_test_app();

        send_action_and_update(&mut app, Action::EndTurn {
            player_id: PlayerId::new(&p1),
        });

        let game = &app.world().resource::<GameState>().game;
        let active = &app.world().resource::<ActivePlayerId>().player_id;

        assert_eq!(game.current_player_id(), p2.as_str(), "P2 should be active");
        assert_eq!(game.current_step(), Step::FirstMain, "P2 at FirstMain");
        assert_eq!(active, &p2, "Perspective should be P2");
    }

    #[test]
    fn end_turn_from_first_main_reaches_p2_first_main() {
        let (mut app, p1, p2) = make_test_app();

        // Trace the state step by step
        {
            let game = &app.world().resource::<GameState>().game;
            eprintln!("BEFORE EndTurn: step={:?} player={} priority={:?}",
                game.current_step(), game.current_player_id(), game.priority_player_id());
        }

        // Apply EndTurn
        {
            let game = &mut app.world_mut().resource_mut::<GameState>().game;
            game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();
            eprintln!("AFTER EndTurn (before auto-pass): step={:?} player={} priority={:?}",
                game.current_step(), game.current_player_id(), game.priority_player_id());
        }

        // Run auto-pass
        {
            let game = &mut app.world_mut().resource_mut::<GameState>().game;
            let count = run_auto_pass_loop(game);
            eprintln!("AFTER auto-pass (count={}): step={:?} player={} turn={} priority={:?}",
                count, game.current_step(), game.current_player_id(), game.turn_number(), game.priority_player_id());
        }

        let game = &app.world().resource::<GameState>().game;
        assert_eq!(game.current_player_id(), p2.as_str(), "P2 should be active");
        assert_eq!(game.current_step(), Step::FirstMain, "Should be FirstMain");
    }

    #[test]
    fn full_turn_cycle() {
        let (mut app, p1, p2) = make_test_app();

        // P1 ends turn
        send_action_and_update(&mut app, Action::EndTurn {
            player_id: PlayerId::new(&p1),
        });

        {
            let game = &app.world().resource::<GameState>().game;
            assert_eq!(game.current_player_id(), p2.as_str());
        }

        // P2 ends turn
        send_action_and_update(&mut app, Action::EndTurn {
            player_id: PlayerId::new(&p2),
        });

        let game = &app.world().resource::<GameState>().game;
        let active = &app.world().resource::<ActivePlayerId>().player_id;

        assert_eq!(game.current_player_id(), p1.as_str(), "Back to P1");
        assert_eq!(game.current_step(), Step::FirstMain);
        assert_eq!(active, &p1, "Perspective back to P1");
        assert_eq!(game.turn_number(), 2);
    }
}

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
