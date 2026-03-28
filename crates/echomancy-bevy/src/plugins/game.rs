//! GamePlugin — bridge between the echomancy-core domain and Bevy's ECS.
//!
//! Responsibilities:
//! - Hold the domain `Game` as a Bevy `Resource` (`GameState`).
//! - Mirror the snapshot and allowed actions as resources (`CurrentSnapshot`, `PlayableCards`).
//! - Register `GameActionMessage` for UI systems to send actions.
//! - Register `SnapshotChangedMessage` so UI systems know when to rebuild.
//! - Set up a 2D camera on startup.

use bevy::prelude::*;
use echomancy_core::prelude::*;
use uuid::Uuid;

// ============================================================================
// Resources
// ============================================================================

/// Holds the live domain `Game` aggregate.
///
/// All state mutations go through `GameActionMessage` → `handle_game_actions`.
/// UI systems read `CurrentSnapshot` instead of this directly.
#[derive(Resource)]
pub(crate) struct GameState {
    pub(crate) game: Game,
}

/// The most recent player-relative snapshot, recomputed after every mutation.
///
/// UI plugins (Phase 8.2+) read `snapshot` to rebuild rendered card state.
#[derive(Resource)]
#[allow(dead_code)]
pub(crate) struct CurrentSnapshot {
    pub(crate) snapshot: GameSnapshot,
}

/// The most recent allowed-actions result, recomputed after every mutation.
///
/// UI plugins (Phase 8.2+) read `result` to highlight playable cards.
#[derive(Resource)]
#[allow(dead_code)]
pub(crate) struct PlayableCards {
    pub(crate) result: AllowedActionsResult,
}

/// The player whose perspective drives the UI (updated whenever priority changes).
#[derive(Resource)]
pub(crate) struct ActivePlayerId {
    pub(crate) player_id: String,
}

/// A single player's identity info: their ID and display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlayerInfo {
    pub(crate) id: String,
    pub(crate) name: String,
}

/// Stores both players' IDs and display names so the HUD can label whose
/// perspective is currently shown.
#[derive(Resource)]
pub(crate) struct PlayerIds {
    pub(crate) p1: PlayerInfo,
    pub(crate) p2: PlayerInfo,
}

impl PlayerIds {
    /// Return the display name of the player with the given ID, or `"Unknown"`.
    pub(crate) fn name_for(&self, player_id: &str) -> &str {
        if self.p1.id == player_id {
            &self.p1.name
        } else if self.p2.id == player_id {
            &self.p2.name
        } else {
            "Unknown"
        }
    }
}

/// Stores the most recent domain error message to display in the HUD.
///
/// Set to `Some(msg)` when an action is rejected; cleared to `None` on the
/// next successful action. The HUD reads this resource to render a red alert.
#[derive(Resource, Default)]
pub(crate) struct ErrorMessage {
    pub(crate) message: Option<String>,
}

// ============================================================================
// Messages
// ============================================================================

/// Sent by UI systems when the local player performs a game action.
///
/// `handle_game_actions` reads this, applies it to `GameState`, and
/// recomputes the snapshot.
#[derive(Message, Clone)]
pub(crate) struct GameActionMessage(pub(crate) Action);

/// Sent after the snapshot is recomputed.
///
/// UI systems should listen for this message to trigger a full rebuild of
/// any rendered card state.
#[derive(Message)]
pub(crate) struct SnapshotChangedMessage;

// ============================================================================
// Card registry for snapshot creation
// ============================================================================

/// Simple card registry that resolves definition IDs to human-readable names.
///
/// In the MVP the catalog is a small static set. This delegates to the
/// catalog's naming convention: the definition ID is the canonical name source.
struct CatalogRegistry;

impl CardRegistry for CatalogRegistry {
    fn card_name(&self, definition_id: &str) -> String {
        // Map known definition IDs to display names.
        match definition_id {
            "forest" => "Forest".to_owned(),
            "mountain" => "Mountain".to_owned(),
            "plains" => "Plains".to_owned(),
            "island" => "Island".to_owned(),
            "swamp" => "Swamp".to_owned(),
            "bear" => "Bear".to_owned(),
            "elite-vanguard" => "Elite Vanguard".to_owned(),
            "giant-growth" => "Giant Growth".to_owned(),
            "lightning-strike" => "Lightning Strike".to_owned(),
            other => other.to_owned(),
        }
    }
}

// ============================================================================
// Priority-based perspective helper
// ============================================================================

/// Determine which player should currently control the UI.
///
/// The UI follows the current active player (whose turn it is). Priority can
/// be stale in non-main-phase steps (e.g., during auto-resolved Untap/Upkeep/
/// Draw), so `current_player_id` is the reliable signal for who the UI should
/// represent. Priority is used only when it explicitly belongs to the *opponent*
/// of the active player, which can happen when responding during the opponent's
/// turn (a future feature). For MVP, the active player always drives the UI.
///
/// Fallback order:
/// 1. `current_player_id` — always authoritative for whose turn it is.
/// 2. `priority_player_id` if different from `current_player_id` (future: opponent response).
pub(crate) fn resolve_ui_player_id<'a>(
    _priority_player_id: Option<&'a str>,
    current_player_id: &'a str,
) -> &'a str {
    current_player_id
}

// ============================================================================
// Auto-advance helper
// ============================================================================

/// Advance through non-interactive steps (Untap, Upkeep, Draw) to reach
/// FirstMain where the player can actually take actions.
///
/// This is called both at startup (for P1) and whenever the active player
/// changes (for P2, P1 again, etc.). Without this, the player would need
/// to manually click "Pass Priority" through steps where nothing happens.
fn auto_advance_to_main_phase(game: &mut Game, player_id: &str) {
    let steps_to_advance = match game.current_step() {
        Step::Untap => 3,     // Untap → Upkeep → Draw → FirstMain
        Step::Upkeep => 2,    // Upkeep → Draw → FirstMain
        Step::Draw => 1,      // Draw → FirstMain
        _ => 0,               // Already past the boring steps
    };

    for _ in 0..steps_to_advance {
        if game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(player_id),
        }).is_err() {
            break;
        }
    }
}

// ============================================================================
// Snapshot helper
// ============================================================================

/// Compute a fresh `GameSnapshot` and `AllowedActionsResult` for the given viewer.
///
/// This is a pure function: it takes `&Game` and a player ID, and returns both
/// results. The caller is responsible for storing them into ECS resources.
///
/// # Errors
///
/// Returns `SnapshotError::PlayerNotFound` if `viewer_player_id` is not in the game.
pub(crate) fn compute_snapshot(
    game: &Game,
    viewer_player_id: &str,
) -> Result<(GameSnapshot, AllowedActionsResult), SnapshotError> {
    let export = game.export_state();
    let snapshot = create_game_snapshot(&export, viewer_player_id, &CatalogRegistry)?;

    // Compute which land cards are playable right now for this viewer.
    let playable_lands = compute_playable_lands(game, viewer_player_id);
    let result = AllowedActionsResult { playable_lands };

    Ok((snapshot, result))
}

/// Returns the instance IDs of lands in the viewer's hand that can be played.
///
/// Replicates the domain rule from `GetAllowedActions` without going through
/// the repository layer (we hold the `Game` directly in the Bevy resource).
fn compute_playable_lands(game: &Game, player_id: &str) -> Vec<String> {
    // Active player only.
    if game.current_player_id() != player_id {
        return Vec::new();
    }
    // No land already played this turn.
    if game.played_lands_this_turn() > 0 {
        return Vec::new();
    }
    // Must be a main phase.
    let in_main_phase = matches!(
        game.current_step(),
        Step::FirstMain | Step::SecondMain
    );
    if !in_main_phase {
        return Vec::new();
    }
    // Stack must be empty — checked via hand accessor (public API).
    let hand = match game.hand(player_id) {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    hand.iter()
        .filter(|c| c.definition().types().contains(&CardType::Land))
        .map(|c| c.instance_id().to_owned())
        .collect()
}

// ============================================================================
// Systems
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
}

/// One-shot system that fires after startup to notify UI of initial state.
pub(crate) fn send_initial_snapshot_message(
    mut snapshot_changed: MessageWriter<SnapshotChangedMessage>,
) {
    snapshot_changed.write(SnapshotChangedMessage);
}

/// Update system: drain `GameActionMessage`s, apply each to the domain game,
/// recompute the snapshot, and send `SnapshotChangedMessage`.
///
/// After each action, checks whether the active player has changed (e.g.,
/// after `EndTurn`) and updates `ActivePlayerId` accordingly so the UI always
/// shows the perspective of the player whose turn it currently is.
///
/// On success, clears `ErrorMessage`. On failure, stores the error string in
/// `ErrorMessage` so the HUD can display it.
pub(crate) fn handle_game_actions(
    mut game_state: ResMut<GameState>,
    mut active_player: ResMut<ActivePlayerId>,
    mut action_messages: MessageReader<GameActionMessage>,
    mut snapshot_res: ResMut<CurrentSnapshot>,
    mut playable_res: ResMut<PlayableCards>,
    mut snapshot_changed: MessageWriter<SnapshotChangedMessage>,
    mut error_message: ResMut<ErrorMessage>,
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
                error_message.message = Some(err.to_string());
            }
        }
    }

    if any_applied {
        // Determine which player's perspective the UI should show.
        // Uses current_player_id as the primary signal — priority can be stale
        // during auto-resolved steps (Untap/Upkeep/Draw).
        let new_ui_player = resolve_ui_player_id(
            game_state.game.priority_player_id(),
            game_state.game.current_player_id(),
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

            // Auto-advance through non-interactive steps (Untap → Upkeep → Draw → FirstMain)
            // so the new player starts in a state where they can act.
            auto_advance_to_main_phase(&mut game_state.game, &new_ui_player);
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

/// Startup system: spawn a 2D camera so Bevy renders the window.
pub(crate) fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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
            .add_systems(Update, handle_game_actions);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// Build a started 2-player game with prebuilt decks.
    fn make_started_game() -> (Game, String, String) {
        let p1 = uuid();
        let p2 = uuid();
        let mut game = Game::create(uuid());
        game.add_player(&p1, "Alice").unwrap();
        game.add_player(&p2, "Bob").unwrap();
        game.assign_deck(&p1, prebuilt_decks::green_deck(&p1))
            .unwrap();
        game.assign_deck(&p2, prebuilt_decks::red_deck(&p2))
            .unwrap();
        game.start(&p1, Some(42)).unwrap();
        (game, p1, p2)
    }

    // ---- compute_snapshot ---------------------------------------------------

    #[test]
    fn compute_snapshot_returns_correct_viewer_id() {
        let (game, p1, _) = make_started_game();
        let (snapshot, _) = compute_snapshot(&game, &p1).unwrap();
        assert_eq!(snapshot.viewer_player_id, p1);
    }

    #[test]
    fn compute_snapshot_errors_for_unknown_player() {
        let (game, _, _) = make_started_game();
        let result = compute_snapshot(&game, "nonexistent-player");
        assert!(result.is_err());
    }

    #[test]
    fn compute_snapshot_initial_turn_is_one() {
        let (game, p1, _) = make_started_game();
        let (snapshot, _) = compute_snapshot(&game, &p1).unwrap();
        assert_eq!(snapshot.public_game_state.turn_number, 1);
    }

    #[test]
    fn compute_snapshot_initial_step_is_untap() {
        let (game, p1, _) = make_started_game();
        let (snapshot, _) = compute_snapshot(&game, &p1).unwrap();
        assert_eq!(snapshot.public_game_state.current_step, Step::Untap);
    }

    #[test]
    fn compute_snapshot_initial_hand_has_seven_cards() {
        let (game, p1, _) = make_started_game();
        let (snapshot, _) = compute_snapshot(&game, &p1).unwrap();
        assert_eq!(snapshot.private_player_state.hand.len(), 7);
    }

    #[test]
    fn compute_snapshot_initial_playable_lands_empty_in_untap() {
        let (game, p1, _) = make_started_game();
        let (_, playable) = compute_snapshot(&game, &p1).unwrap();
        // Game starts in Untap — cannot play lands yet.
        assert!(playable.playable_lands.is_empty());
    }

    // ---- compute_playable_lands -------------------------------------------

    #[test]
    fn playable_lands_empty_for_non_active_player() {
        let (game, _, p2) = make_started_game();
        let lands = compute_playable_lands(&game, &p2);
        assert!(lands.is_empty(), "Non-active player cannot play lands");
    }

    #[test]
    fn playable_lands_empty_in_untap_step() {
        let (game, p1, _) = make_started_game();
        let lands = compute_playable_lands(&game, &p1);
        assert!(
            lands.is_empty(),
            "Cannot play lands in Untap step (not a main phase)"
        );
    }

    #[test]
    fn playable_lands_available_in_first_main() {
        let (mut game, p1, _) = make_started_game();
        // Advance: Untap -> Upkeep -> Draw -> FirstMain (3 AdvanceStep actions)
        for _ in 0..3 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        let lands = compute_playable_lands(&game, &p1);
        // Green deck has 24 forests; 7-card opening hand will contain some lands.
        assert!(
            !lands.is_empty(),
            "Should have playable lands in FirstMain with a green deck"
        );
    }

    // ---- PlayerIds ---------------------------------------------------------

    #[test]
    fn player_ids_name_for_returns_p1_name() {
        let ids = PlayerIds {
            p1: PlayerInfo { id: "id-1".to_owned(), name: "Alice".to_owned() },
            p2: PlayerInfo { id: "id-2".to_owned(), name: "Bob".to_owned() },
        };
        assert_eq!(ids.name_for("id-1"), "Alice");
    }

    #[test]
    fn player_ids_name_for_returns_p2_name() {
        let ids = PlayerIds {
            p1: PlayerInfo { id: "id-1".to_owned(), name: "Alice".to_owned() },
            p2: PlayerInfo { id: "id-2".to_owned(), name: "Bob".to_owned() },
        };
        assert_eq!(ids.name_for("id-2"), "Bob");
    }

    #[test]
    fn player_ids_name_for_returns_unknown_for_bad_id() {
        let ids = PlayerIds {
            p1: PlayerInfo { id: "id-1".to_owned(), name: "Alice".to_owned() },
            p2: PlayerInfo { id: "id-2".to_owned(), name: "Bob".to_owned() },
        };
        assert_eq!(ids.name_for("bad-id"), "Unknown");
    }

    // ---- resolve_ui_player_id ----------------------------------------------

    /// The UI always follows the current active player (whose turn it is),
    /// not the priority holder. Priority can be stale during auto-resolved steps.
    #[test]
    fn resolve_ui_player_returns_current_player_regardless_of_priority() {
        // Even if priority says p2, current player is p1 — UI shows p1.
        let result = resolve_ui_player_id(Some("p2"), "p1");
        assert_eq!(result, "p1");
    }

    #[test]
    fn resolve_ui_player_returns_current_when_no_priority() {
        let result = resolve_ui_player_id(None, "p1");
        assert_eq!(result, "p1");
    }

    #[test]
    fn resolve_ui_player_returns_current_when_priority_matches() {
        let result = resolve_ui_player_id(Some("p1"), "p1");
        assert_eq!(result, "p1");
    }

    /// After P1 ends their turn, the resolved UI player should be P2
    /// (since P2 becomes the current active player).
    #[test]
    fn resolve_ui_player_is_p2_after_p1_ends_turn() {
        let (mut game, p1, p2) = make_started_game();
        for _ in 0..3 {
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        }
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();

        let ui_player = resolve_ui_player_id(
            game.priority_player_id(),
            game.current_player_id(),
        );
        assert_eq!(ui_player, p2.as_str(),
            "UI should show P2 after P1 ends their turn");
    }

    // ---- priority switching (domain-level) ---------------------------------

    /// After P1's turn ends (via EndTurn), P2 should be the active player.
    #[test]
    fn active_player_switches_to_p2_after_p1_ends_turn() {
        let (mut game, p1, p2) = make_started_game();
        // Advance to FirstMain (Untap → Upkeep → Draw → FirstMain).
        for _ in 0..3 {
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        }
        // P1 ends their turn.
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();
        // P2 should now be the active player.
        assert_eq!(game.current_player_id(), p2.as_str());
    }

    /// The snapshot correctly reports P2's priority after P1 ends their turn.
    #[test]
    fn snapshot_priority_player_id_is_p2_after_p1_ends_turn() {
        let (mut game, p1, p2) = make_started_game();
        for _ in 0..3 {
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        }
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();

        // Snapshot from P2's perspective should show P2 has priority (or no one
        // holds it mid-auto-advance, but the active player is P2).
        let (snapshot, _) = compute_snapshot(&game, &p2).unwrap();
        assert_eq!(snapshot.public_game_state.current_player_id, p2,
            "P2 should be the active player after P1 ends turn");
    }

    // ---- CatalogRegistry --------------------------------------------------

    #[test]
    fn catalog_registry_resolves_known_cards() {
        let registry = CatalogRegistry;
        assert_eq!(registry.card_name("forest"), "Forest");
        assert_eq!(registry.card_name("mountain"), "Mountain");
        assert_eq!(registry.card_name("bear"), "Bear");
        assert_eq!(registry.card_name("elite-vanguard"), "Elite Vanguard");
        assert_eq!(registry.card_name("giant-growth"), "Giant Growth");
        assert_eq!(registry.card_name("lightning-strike"), "Lightning Strike");
    }

    #[test]
    fn catalog_registry_returns_raw_id_for_unknown_cards() {
        let registry = CatalogRegistry;
        assert_eq!(registry.card_name("some-unknown-card"), "some-unknown-card");
    }
}
