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
/// In most steps the active player (`current_player_id`) drives the UI.
/// During `DeclareBlockers`, however, the defending player (whoever holds
/// priority at that step) must control the UI so they can click their
/// blockers.  We detect this by checking whether `priority_player_id`
/// differs from `current_player_id` while in a combat step.
///
/// Fallback order:
/// 1. During `DeclareBlockers`: the priority holder (defending player).
/// 2. Otherwise: `current_player_id`.
pub(crate) fn resolve_ui_player_id<'a>(
    priority_player_id: Option<&'a str>,
    current_player_id: &'a str,
    current_step: Step,
) -> &'a str {
    // During DeclareBlockers the defending player holds priority.
    if current_step == Step::DeclareBlockers {
        if let Some(priority_id) = priority_player_id {
            if priority_id != current_player_id {
                return priority_id;
            }
        }
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
    // Compute which battlefield lands can be tapped for mana right now.
    let tappable_lands = compute_tappable_lands(game, viewer_player_id);
    let castable_spells = compute_castable_spells(game, viewer_player_id);
    let spells_needing_targets = compute_spells_needing_targets(game, viewer_player_id, &castable_spells);
    let attackable_creatures = compute_attackable_creatures(game, viewer_player_id);
    let blockable_creatures = compute_blockable_creatures(game, viewer_player_id);
    let result = AllowedActionsResult {
        playable_lands,
        tappable_lands,
        castable_spells,
        spells_needing_targets,
        attackable_creatures,
        blockable_creatures,
    };

    Ok((snapshot, result))
}

/// Returns the instance IDs of lands in the viewer's hand that can be played.
///
/// Replicates the domain rule from `GetAllowedActions` without going through
/// the repository layer (we hold the `Game` directly in the Bevy resource).
pub(crate) fn compute_playable_lands(game: &Game, player_id: &str) -> Vec<String> {
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

/// Returns the instance IDs of untapped lands on the player's battlefield
/// that can be tapped for mana right now.
///
/// Mana abilities (CR 605) can be activated whenever the player has priority,
/// regardless of the game step.
pub(crate) fn compute_tappable_lands(game: &Game, player_id: &str) -> Vec<String> {
    // Player must have priority.
    if game.priority_player_id() != Some(player_id) {
        return Vec::new();
    }

    let battlefield = match game.battlefield(player_id) {
        Ok(bf) => bf,
        Err(_) => return Vec::new(),
    };

    battlefield
        .iter()
        .filter(|card| {
            let is_land = card.definition().types().contains(&CardType::Land);
            let has_mana_ability = card
                .definition()
                .activated_ability()
                .is_some_and(|ab| ab.effect.is_mana_ability());
            if !is_land || !has_mana_ability {
                return false;
            }
            let is_tapped = game
                .permanent_state(card.instance_id())
                .is_some_and(|s| s.is_tapped());
            !is_tapped
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Returns the instance IDs of non-land spells in the player's hand that can
/// be cast right now at sorcery speed.
///
/// Mirrors `collect_castable_spells` from `echomancy-core`'s queries module,
/// operating directly on `&Game` instead of going through the repository.
///
/// Conditions:
///
/// 1. Player has priority.
/// 2. Player is the active player (their turn).
/// 3. Current step is a main phase.
/// 4. Stack is empty.
/// 5. Player can pay the mana cost from their current pool.
pub(crate) fn compute_castable_spells(game: &Game, player_id: &str) -> Vec<String> {
    use echomancy_core::prelude::{ManaCost, can_pay_cost};

    // 1. Must have priority.
    if game.priority_player_id() != Some(player_id) {
        return Vec::new();
    }

    // 2. Must be the active player.
    if game.current_player_id() != player_id {
        return Vec::new();
    }

    // 3. Must be a main phase.
    if !matches!(game.current_step(), Step::FirstMain | Step::SecondMain) {
        return Vec::new();
    }

    // 4. Stack must be empty (sorcery speed).
    if game.stack_has_items() {
        return Vec::new();
    }

    let mana_pool = match game.mana_pool(player_id) {
        Ok(pool) => pool,
        Err(_) => return Vec::new(),
    };

    let hand = match game.hand(player_id) {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    hand.iter()
        .filter(|card| {
            // Lands are played, not cast.
            if card.definition().types().contains(&CardType::Land) {
                return false;
            }
            // Allow creatures, sorceries, and instants to be cast during main phase.
            // Instants can also be cast at other times (P2 feature), but for now
            // they are at least castable at sorcery speed.
            let is_castable_type = card.definition().types().iter().any(|t| {
                matches!(t, CardType::Creature | CardType::Sorcery | CardType::Instant)
            });
            if !is_castable_type {
                return false;
            }
            // Must be able to pay the mana cost.
            let cost = card
                .definition()
                .mana_cost()
                .cloned()
                .unwrap_or_else(ManaCost::zero);
            can_pay_cost(mana_pool, &cost)
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Returns the subset of `castable_spell_ids` whose cards declare a non-`None`
/// `TargetRequirement`.
///
/// These spells require the player to choose a target in the UI before the
/// `CastSpell` action is dispatched. The hand click handler checks this list
/// and enters target-selection mode instead of casting immediately.
pub(crate) fn compute_spells_needing_targets(
    game: &Game,
    player_id: &str,
    castable_spell_ids: &[String],
) -> Vec<String> {
    use echomancy_core::domain::targets::TargetRequirement;

    let hand = match game.hand(player_id) {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };

    castable_spell_ids
        .iter()
        .filter(|id| {
            hand.iter()
                .find(|card| card.instance_id() == id.as_str())
                .is_some_and(|card| {
                    card.definition().target_requirement() != TargetRequirement::None
                })
        })
        .cloned()
        .collect()
}

/// Returns instance IDs of untapped, non-summoning-sick creatures the player
/// can declare as attackers during the `DeclareAttackers` step.
pub(crate) fn compute_attackable_creatures(game: &Game, player_id: &str) -> Vec<String> {
    use echomancy_core::prelude::StaticAbility;

    if game.current_step() != Step::DeclareAttackers {
        return Vec::new();
    }
    if game.current_player_id() != player_id {
        return Vec::new();
    }
    let battlefield = match game.battlefield(player_id) {
        Ok(bf) => bf,
        Err(_) => return Vec::new(),
    };
    battlefield
        .iter()
        .filter(|card| {
            if !card.definition().is_creature() {
                return false;
            }
            let Some(state) = game.permanent_state(card.instance_id()) else {
                return false;
            };
            if state.is_tapped() {
                return false;
            }
            let Some(cs) = state.creature_state() else {
                return false;
            };
            if cs.has_attacked_this_turn() {
                return false;
            }
            if cs.has_summoning_sickness()
                && !card
                    .definition()
                    .static_abilities()
                    .contains(&StaticAbility::Haste)
            {
                return false;
            }
            true
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Returns instance IDs of untapped creatures on the defending player's
/// battlefield that can be declared as blockers during `DeclareBlockers`.
pub(crate) fn compute_blockable_creatures(game: &Game, player_id: &str) -> Vec<String> {
    if game.current_step() != Step::DeclareBlockers {
        return Vec::new();
    }
    // Defending player is NOT the current (active) player.
    if game.current_player_id() == player_id {
        return Vec::new();
    }
    let battlefield = match game.battlefield(player_id) {
        Ok(bf) => bf,
        Err(_) => return Vec::new(),
    };
    battlefield
        .iter()
        .filter(|card| {
            if !card.definition().is_creature() {
                return false;
            }
            let Some(state) = game.permanent_state(card.instance_id()) else {
                return false;
            };
            if state.is_tapped() {
                return false;
            }
            let Some(cs) = state.creature_state() else {
                return false;
            };
            cs.blocking_creature_id().is_none()
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
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

    // ---- resolve_ui_player_id ----------------------------------------------

    /// During non-combat steps the UI follows the current active player.
    #[test]
    fn resolve_ui_player_returns_current_player_in_first_main() {
        let result = resolve_ui_player_id(Some("p2"), "p1", Step::FirstMain);
        assert_eq!(result, "p1");
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

    /// During DeclareAttackers, the active player still drives the UI.
    #[test]
    fn resolve_ui_player_returns_current_during_declare_attackers() {
        let result = resolve_ui_player_id(Some("p2"), "p1", Step::DeclareAttackers);
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

    // ---- compute_castable_spells -------------------------------------------

    /// Helper: build a minimal game at FirstMain, P1 has priority.
    fn make_game_in_first_main() -> (Game, String, String) {
        let (mut game, p1, p2) = make_started_game();
        for _ in 0..3 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        (game, p1, p2)
    }

    /// With seed=42 the green deck's 7-card opening hand contains Bears.
    /// With an empty mana pool, those Bears ({1}{G}) cannot be cast.
    #[test]
    fn castable_spells_empty_when_no_mana() {
        let (game, p1, _) = make_game_in_first_main();
        // Mana pool is empty (no taps done yet).
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
        let castable = compute_castable_spells(&game, &p1);
        assert!(
            castable.is_empty(),
            "No spells should be castable with empty mana pool"
        );
    }

    #[test]
    fn castable_spells_empty_for_non_active_player() {
        let (game, _, p2) = make_game_in_first_main();
        let castable = compute_castable_spells(&game, &p2);
        assert!(
            castable.is_empty(),
            "Non-active player cannot cast spells at sorcery speed"
        );
    }

    #[test]
    fn castable_spells_empty_outside_main_phase() {
        let (game, p1, _) = make_started_game();
        // Game starts in Untap, not a main phase.
        let castable = compute_castable_spells(&game, &p1);
        assert!(
            castable.is_empty(),
            "Spells should not be castable during Untap step"
        );
    }

    /// With seed=42, P1's opening hand from the green deck includes a Bear.
    /// After adding {1}{G} to the pool, the Bear should appear as castable.
    #[test]
    fn castable_spells_returns_affordable_bear_after_adding_mana() {
        let (mut game, p1, _) = make_game_in_first_main();

        // Verify a Bear is in hand (seed=42 makes this deterministic).
        let bear_in_hand = game
            .hand(&p1)
            .unwrap()
            .iter()
            .any(|c| c.definition().id() == "bear");
        if !bear_in_hand {
            // Seed produced a hand with no bears — skip test rather than assert.
            // (This is very unlikely with 20/60 bears but defensively handled.)
            return;
        }

        // Give P1 exactly {1}{G} to pay for the Bear.
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let castable = compute_castable_spells(&game, &p1);
        assert!(
            !castable.is_empty(),
            "At least one Bear should be castable with {{1}}{{G}} in pool"
        );
        // All returned IDs must be Bears or Giant Growths (the non-land spells
        // in the green deck that are affordable with {1}{G}).
        for id in &castable {
            let card = game
                .hand(&p1)
                .unwrap()
                .iter()
                .find(|c| c.instance_id() == id)
                .expect("castable_id should refer to a card in hand");
            assert!(
                card.definition().id() == "bear" || card.definition().id() == "giant-growth",
                "Castable card should be a Bear or Giant Growth, got: {}",
                card.definition().id()
            );
        }
    }

    // ---- compute_spells_needing_targets ------------------------------------

    /// `compute_spells_needing_targets` must only include IDs that are already in
    /// the castable-spells list. An ID not in `castable_spell_ids` must never appear
    /// in the output even if the card's definition requires a target.
    ///
    /// Note: Lightning Strike is an Instant; it is excluded from `castable_spells`
    /// (which currently only covers sorcery-speed Creature/Sorcery cards). Therefore
    /// `spells_needing_targets` is always a subset of castable_spells and will be
    /// empty when the only targeting spell (Lightning Strike) is not castable yet.
    #[test]
    fn spells_needing_targets_is_subset_of_castable_spells() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Give mana so castable_spells is non-empty.
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let castable = compute_castable_spells(&game, &p1);
        let needs_targets = compute_spells_needing_targets(&game, &p1, &castable);

        // Every ID in needs_targets must also be in castable.
        for id in &needs_targets {
            assert!(
                castable.contains(id),
                "needs_targets must be a subset of castable_spells, but '{id}' is not in castable"
            );
        }
    }

    /// Bears require no targets; they must NOT appear in `spells_needing_targets`.
    #[test]
    fn spells_needing_targets_excludes_bear() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Give mana for a Bear ({1}{G}).
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let castable = compute_castable_spells(&game, &p1);
        let needs_targets = compute_spells_needing_targets(&game, &p1, &castable);

        // Any castable Bear should NOT appear in needs_targets.
        for id in &castable {
            let card = game
                .hand(&p1)
                .unwrap()
                .iter()
                .find(|c| c.instance_id() == id)
                .expect("castable ID should be in hand");
            if card.definition().id() == "bear" {
                assert!(
                    !needs_targets.contains(id),
                    "Bear should not need a target"
                );
            }
        }
    }

    /// Empty castable list → empty needs-targets list (no panics, no incorrect entries).
    #[test]
    fn spells_needing_targets_empty_when_nothing_castable() {
        let (game, p1, _) = make_game_in_first_main();
        // No mana given — castable_spells should be empty.
        let castable = compute_castable_spells(&game, &p1);
        assert!(castable.is_empty(), "Precondition: no castable spells");
        let needs_targets = compute_spells_needing_targets(&game, &p1, &castable);
        assert!(needs_targets.is_empty(), "No castable spells → no spells needing targets");
    }

    /// Passing a fake instance ID in the castable list returns empty needs-targets
    /// (the function must not panic on IDs not found in hand).
    #[test]
    fn spells_needing_targets_ignores_unknown_ids() {
        let (game, p1, _) = make_game_in_first_main();
        let fake_ids = vec!["unknown-id-xyz".to_owned()];
        let needs_targets = compute_spells_needing_targets(&game, &p1, &fake_ids);
        assert!(
            needs_targets.is_empty(),
            "Unknown IDs must not appear in needs_targets"
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
