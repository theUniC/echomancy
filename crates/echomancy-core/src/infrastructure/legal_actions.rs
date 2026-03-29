//! Legal-actions computation — single source of truth for what a player can do.
//!
//! This module is the authoritative implementation of the timing rules that
//! determine which game actions a player may legally take at any given moment.
//! It replaces the duplicate logic that previously lived in the Bevy snapshot
//! layer, making `echomancy-core` the single source of truth.
//!
//! # Usage
//!
//! ```ignore
//! let allowed = compute_legal_actions(&game, player_id);
//! // allowed.playable_lands, .castable_spells, etc.
//! ```

use crate::domain::enums::{CardType, StaticAbility, Step};
use crate::domain::game::Game;
use crate::domain::targets::TargetRequirement;
use crate::domain::value_objects::mana::ManaCost;
use crate::infrastructure::allowed_actions::AllowedActionsResult;
use crate::domain::services::mana_payment::can_pay_cost;

/// Compute the full set of actions a player can legally take right now.
///
/// This is a pure read-only function — it never mutates the game. The caller
/// can store the returned value in whatever UI / ECS layer they choose.
///
/// All timing rules (main phase, priority, sorcery speed, mana check, etc.)
/// are enforced here using the domain's own checks, so there is no need to
/// duplicate them in the Bevy layer or any other consumer.
///
/// # Arguments
///
/// * `game`      — the current game state.
/// * `player_id` — the player whose legal actions we are computing.
///
/// # Returns
///
/// A fully populated [`AllowedActionsResult`]. Every field may be empty (e.g.
/// when it is not the player's turn), but the struct is always returned.
pub fn compute_legal_actions(game: &Game, player_id: &str) -> AllowedActionsResult {
    let playable_lands = compute_playable_lands(game, player_id);
    let tappable_lands = compute_tappable_lands(game, player_id);
    let castable_spells = compute_castable_spells(game, player_id);
    let spells_needing_targets =
        compute_spells_needing_targets(game, player_id, &castable_spells);
    let attackable_creatures = compute_attackable_creatures(game, player_id);
    let blockable_creatures = compute_blockable_creatures(game, player_id);

    AllowedActionsResult {
        playable_lands,
        tappable_lands,
        castable_spells,
        spells_needing_targets,
        attackable_creatures,
        blockable_creatures,
    }
}

// ============================================================================
// Individual rule sub-functions (pub(crate) so Bevy can delegate to them if
// ever needed, but the primary API surface is `compute_legal_actions`).
// ============================================================================

/// Returns the instance IDs of lands in the viewer's hand that can be played.
///
/// Rules enforced (CR 305.4):
/// - Active player only.
/// - No land already played this turn.
/// - Must be a main phase (FirstMain or SecondMain).
/// - Stack must be empty (playing a land is a sorcery-speed special action).
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
    if !matches!(game.current_step(), Step::FirstMain | Step::SecondMain) {
        return Vec::new();
    }
    // Stack must be empty (CR 305.4 — playing a land is a sorcery-speed action).
    if game.stack_has_items() {
        return Vec::new();
    }
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
/// Conditions:
/// 1. Player has priority.
/// 2. Player is the active player (their turn).
/// 3. Current step is a main phase.
/// 4. Stack is empty.
/// 5. Player can pay the mana cost from their current pool.
pub(crate) fn compute_castable_spells(game: &Game, player_id: &str) -> Vec<String> {
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
            // Instants can also be cast at other times (future feature), but for now
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::cards::prebuilt_decks;
    use crate::domain::enums::ManaColor;
    use crate::domain::game::Game;
    use crate::domain::types::PlayerId;
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

    /// Build a started game advanced to FirstMain (p1 has priority).
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

    // ---- compute_legal_actions (integration) --------------------------------

    #[test]
    fn compute_legal_actions_returns_empty_in_untap_step() {
        let (game, p1, _) = make_started_game();
        // Game starts in Untap.
        let allowed = compute_legal_actions(&game, &p1);
        assert!(allowed.playable_lands.is_empty(), "no lands playable in Untap");
        assert!(allowed.castable_spells.is_empty(), "no spells castable in Untap");
    }

    #[test]
    fn compute_legal_actions_has_playable_lands_in_first_main() {
        let (mut game, p1, _) = make_started_game();
        for _ in 0..3 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        let allowed = compute_legal_actions(&game, &p1);
        // Green deck has 24 forests; opening hand will contain some.
        assert!(
            !allowed.playable_lands.is_empty(),
            "Should have playable lands in FirstMain with green deck"
        );
    }

    #[test]
    fn compute_legal_actions_spells_needing_targets_subset_of_castable() {
        let (mut game, p1, _) = make_game_in_first_main();
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let allowed = compute_legal_actions(&game, &p1);

        for id in &allowed.spells_needing_targets {
            assert!(
                allowed.castable_spells.contains(id),
                "spells_needing_targets must be a subset of castable_spells"
            );
        }
    }

    // ---- compute_playable_lands ---------------------------------------------

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
        for _ in 0..3 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        let lands = compute_playable_lands(&game, &p1);
        assert!(
            !lands.is_empty(),
            "Should have playable lands in FirstMain with a green deck"
        );
    }

    // ---- compute_tappable_lands ---------------------------------------------

    #[test]
    fn tappable_lands_empty_when_no_priority() {
        let (game, _, p2) = make_started_game();
        // p2 does not have priority at game start.
        let tappable = compute_tappable_lands(&game, &p2);
        assert!(tappable.is_empty(), "Player without priority cannot tap lands");
    }

    // ---- compute_castable_spells --------------------------------------------

    #[test]
    fn castable_spells_empty_when_no_mana() {
        let (game, p1, _) = make_game_in_first_main();
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
        let castable = compute_castable_spells(&game, &p1);
        assert!(
            castable.is_empty(),
            "Spells should not be castable during Untap step"
        );
    }

    #[test]
    fn castable_spells_returns_affordable_bear_after_adding_mana() {
        let (mut game, p1, _) = make_game_in_first_main();

        let bear_in_hand = game
            .hand(&p1)
            .unwrap()
            .iter()
            .any(|c| c.definition().id() == "bear");
        if !bear_in_hand {
            return; // Seed produced a hand without bears — skip.
        }

        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let castable = compute_castable_spells(&game, &p1);
        assert!(
            !castable.is_empty(),
            "At least one Bear should be castable with {{1}}{{G}} in pool"
        );
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

    // ---- compute_spells_needing_targets -------------------------------------

    #[test]
    fn spells_needing_targets_is_subset_of_castable_spells() {
        let (mut game, p1, _) = make_game_in_first_main();
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let castable = compute_castable_spells(&game, &p1);
        let needs_targets = compute_spells_needing_targets(&game, &p1, &castable);

        for id in &needs_targets {
            assert!(
                castable.contains(id),
                "needs_targets must be a subset of castable_spells, but '{id}' is not in castable"
            );
        }
    }

    #[test]
    fn spells_needing_targets_excludes_bear() {
        let (mut game, p1, _) = make_game_in_first_main();
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let castable = compute_castable_spells(&game, &p1);
        let needs_targets = compute_spells_needing_targets(&game, &p1, &castable);

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

    #[test]
    fn spells_needing_targets_empty_when_nothing_castable() {
        let (game, p1, _) = make_game_in_first_main();
        let castable = compute_castable_spells(&game, &p1);
        assert!(castable.is_empty(), "Precondition: no castable spells");
        let needs_targets = compute_spells_needing_targets(&game, &p1, &castable);
        assert!(
            needs_targets.is_empty(),
            "No castable spells → no spells needing targets"
        );
    }

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

    // ---- compute_attackable_creatures ---------------------------------------

    #[test]
    fn attackable_creatures_empty_outside_declare_attackers_step() {
        let (game, p1, _) = make_started_game();
        let attackable = compute_attackable_creatures(&game, &p1);
        assert!(
            attackable.is_empty(),
            "No attackable creatures outside DeclareAttackers step"
        );
    }

    // ---- compute_blockable_creatures ----------------------------------------

    #[test]
    fn blockable_creatures_empty_outside_declare_blockers_step() {
        let (game, p1, _) = make_started_game();
        let blockable = compute_blockable_creatures(&game, &p1);
        assert!(
            blockable.is_empty(),
            "No blockable creatures outside DeclareBlockers step"
        );
    }
}
