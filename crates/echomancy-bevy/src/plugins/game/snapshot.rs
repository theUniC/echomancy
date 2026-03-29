//! Pure snapshot-computation helpers.
//!
//! All functions here are stateless: they take `&Game` plus a player ID and
//! return derived data. They have no Bevy dependency and are easy to unit-test.

use echomancy_core::prelude::*;

use super::{AllowedActionsResult, CatalogRegistry};

// ============================================================================
// Priority-based perspective helper
// ============================================================================

/// Determine which player should currently control the UI.
///
/// Whenever a player holds priority, they need to see their hand to decide
/// whether to cast a spell or pass. So we show whoever currently holds
/// priority — including the non-active player when they have a response window.
///
/// Non-interactive steps (Untap, Cleanup) have no priority window. Any stale
/// priority value from the previous step is ignored and the current active
/// player drives the UI instead.
///
/// Fallback order:
/// 1. If the step is interactive and there is a priority holder: return the priority holder.
/// 2. Otherwise: `current_player_id`.
pub(crate) fn resolve_ui_player_id<'a>(
    priority_player_id: Option<&'a str>,
    current_player_id: &'a str,
    current_step: Step,
) -> &'a str {
    // Non-interactive steps (Untap, Cleanup) have no priority window.
    // Any stale priority value from the previous step is ignored — the current
    // active player drives the UI instead.
    let is_non_interactive = matches!(current_step, Step::Untap | Step::Cleanup);
    if is_non_interactive {
        return current_player_id;
    }

    // For all interactive steps: show whoever holds priority so they can see
    // their hand and decide whether to cast a spell or pass.
    if let Some(priority_id) = priority_player_id {
        return priority_id;
    }
    current_player_id
}

// ============================================================================
// Snapshot computation
// ============================================================================

/// Compute a fresh `GameSnapshot` and `AllowedActionsResult` for the given viewer.
///
/// This is a pure function: it takes `&Game` and a player ID, and returns both
/// results. The caller is responsible for storing them into ECS resources.
///
/// The allowed-actions logic is entirely delegated to `compute_legal_actions`
/// in `echomancy-core`, keeping all rule knowledge in the domain layer.
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

    let result = compute_legal_actions(game, viewer_player_id);

    Ok((snapshot, result))
}

// ============================================================================
// Error message humanization
// ============================================================================

/// Replace raw UUIDs in an error message string with human-readable player names.
///
/// Domain errors often embed player IDs like `"Player '6eee59fc-...' cannot act"`.
/// This converts them to `"Player 1"` or `"Player 2"` so the HUD shows friendly text.
///
/// Only replaces IDs that appear in `player_ids`. Unknown UUIDs are left as-is.
pub(crate) fn humanize_error(message: &str, player_ids: &super::PlayerIds) -> String {
    let mut result = message.to_owned();
    result = result.replace(&player_ids.p1.id, &player_ids.p1.name);
    result = result.replace(&player_ids.p2.id, &player_ids.p2.name);
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use echomancy_core::prelude::*;
    use uuid::Uuid;

    use crate::plugins::game::{PlayerIds, PlayerInfo};

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

    // ---- resolve_ui_player_id ----------------------------------------------

    /// When the priority holder differs from the current player, show the priority holder.
    /// This covers the response window: p2 holds priority after p1 casts a spell in FirstMain.
    #[test]
    fn resolve_ui_player_returns_priority_holder_when_priority_differs_in_first_main() {
        let result = resolve_ui_player_id(Some("p2"), "p1", Step::FirstMain);
        assert_eq!(result, "p2");
    }

    #[test]
    fn resolve_ui_player_returns_current_when_no_priority() {
        let result = resolve_ui_player_id(None, "p1", Step::FirstMain);
        assert_eq!(result, "p1");
    }

    #[test]
    fn resolve_ui_player_returns_current_when_priority_matches() {
        let result = resolve_ui_player_id(Some("p1"), "p1", Step::FirstMain);
        assert_eq!(result, "p1");
    }

    /// During DeclareBlockers the defending player (priority holder) drives the UI.
    #[test]
    fn resolve_ui_player_returns_priority_holder_during_declare_blockers() {
        // p1 is current (active attacker), p2 has priority (defending player).
        let result = resolve_ui_player_id(Some("p2"), "p1", Step::DeclareBlockers);
        assert_eq!(result, "p2");
    }

    /// During DeclareAttackers, if the opponent holds priority (e.g. after P1 passes),
    /// the UI should show the opponent so they can cast instants.
    #[test]
    fn resolve_ui_player_returns_priority_holder_during_declare_attackers_when_opponent_has_priority() {
        let result = resolve_ui_player_id(Some("p2"), "p1", Step::DeclareAttackers);
        assert_eq!(result, "p2");
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
            game.current_step(),
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

    // ---- Tapped land snapshot (regression: UI should show rotated land) -----

    /// After playing a land and activating its mana ability, `compute_snapshot`
    /// must produce a `CardSnapshot` with `tapped == Some(true)` for that land.
    ///
    /// This test pins the full bridge: domain Game → compute_snapshot → CardSnapshot.tapped
    /// so that the Bevy UI has the correct value to pass to `spawn_card_inner`.
    #[test]
    fn compute_snapshot_reflects_tapped_land_after_activate_ability() {
        let (mut game, p1, _) = make_started_game();

        // Advance to FirstMain so we can play a land and activate abilities.
        for _ in 0..3 {
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        }

        // Find the first forest in P1's hand (green deck has 24 Forests).
        let forest_id = {
            let hand = game.hand(&p1).unwrap();
            hand.iter()
                .find(|c| c.definition().types().contains(&CardType::Land))
                .expect("green deck should have lands in hand")
                .instance_id()
                .to_owned()
        };

        // Play the land onto the battlefield.
        game.apply(Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&forest_id),
        })
        .unwrap();

        // Sanity: land is now on the battlefield and should appear as untapped.
        let (snap_before, _) = compute_snapshot(&game, &p1).unwrap();
        let land_before = snap_before
            .private_player_state
            .battlefield
            .iter()
            .find(|c| c.instance_id == forest_id)
            .expect("land should be on battlefield after PlayLand");
        assert_eq!(
            land_before.tapped,
            Some(false),
            "Newly played land should show tapped == Some(false)"
        );

        // Activate the land's mana ability (tap it for green mana).
        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new(&forest_id),
        })
        .unwrap();

        // The snapshot must now reflect the tapped state.
        let (snap_after, _) = compute_snapshot(&game, &p1).unwrap();
        let land_after = snap_after
            .private_player_state
            .battlefield
            .iter()
            .find(|c| c.instance_id == forest_id)
            .expect("land should still be on battlefield after tapping");
        assert_eq!(
            land_after.tapped,
            Some(true),
            "Tapped land must have tapped == Some(true) so the UI can render it with a tilt"
        );
    }

    // ---- humanize_error -----------------------------------------------------

    fn make_player_ids() -> PlayerIds {
        PlayerIds {
            p1: PlayerInfo { id: "uuid-aaa-111".to_owned(), name: "Player 1".to_owned() },
            p2: PlayerInfo { id: "uuid-bbb-222".to_owned(), name: "Player 2".to_owned() },
        }
    }

    #[test]
    fn humanize_error_replaces_p1_uuid_with_display_name() {
        let ids = make_player_ids();
        let raw = "Player 'uuid-aaa-111' cannot perform action 'CAST_SPELL'";
        let human = humanize_error(raw, &ids);
        assert_eq!(human, "Player 'Player 1' cannot perform action 'CAST_SPELL'");
    }

    #[test]
    fn humanize_error_replaces_p2_uuid_with_display_name() {
        let ids = make_player_ids();
        let raw = "Player 'uuid-bbb-222' cannot perform action 'PASS_PRIORITY'";
        let human = humanize_error(raw, &ids);
        assert_eq!(human, "Player 'Player 2' cannot perform action 'PASS_PRIORITY'");
    }

    #[test]
    fn humanize_error_leaves_unknown_ids_intact() {
        let ids = make_player_ids();
        let raw = "Some error with unknown-uuid-xyz inside";
        let human = humanize_error(raw, &ids);
        assert_eq!(human, raw, "Unknown UUIDs should not be changed");
    }

    #[test]
    fn humanize_error_replaces_all_occurrences_of_uuid() {
        let ids = make_player_ids();
        let raw = "uuid-aaa-111 vs uuid-aaa-111 conflict";
        let human = humanize_error(raw, &ids);
        assert_eq!(human, "Player 1 vs Player 1 conflict");
    }

    #[test]
    fn humanize_error_handles_both_players_in_same_message() {
        let ids = make_player_ids();
        let raw = "uuid-aaa-111 attacked uuid-bbb-222";
        let human = humanize_error(raw, &ids);
        assert_eq!(human, "Player 1 attacked Player 2");
    }

    #[test]
    fn humanize_error_message_with_no_uuid_unchanged() {
        let ids = make_player_ids();
        let raw = "Illegal action: stack is not empty";
        let human = humanize_error(raw, &ids);
        assert_eq!(human, raw);
    }
}
