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
use crate::domain::services::mana_payment::can_pay_cost;
use crate::domain::targets::TargetRequirement;
use crate::domain::value_objects::mana::ManaCost;
use crate::infrastructure::allowed_actions::AllowedActionsResult;

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
    let is_own_sorcery_inline = game.current_player_id() == player_id
        && matches!(game.current_step(), Step::FirstMain | Step::SecondMain)
        && !game.stack_has_items();
    let has_potential_plays_inline = is_own_sorcery_inline
        && !tappable_lands.is_empty()
        && game.hand(player_id)
            .map(|hand| hand.iter().any(|c| !c.definition().is_land()))
            .unwrap_or(false);
    let auto_pass_eligible = playable_lands.is_empty()
        && castable_spells.is_empty()
        && !has_potential_plays_inline
        && attackable_creatures.is_empty()
        && blockable_creatures.is_empty()
        && game.priority_player_id() == Some(player_id);

    AllowedActionsResult {
        playable_lands,
        tappable_lands,
        castable_spells,
        spells_needing_targets,
        attackable_creatures,
        blockable_creatures,
        auto_pass_eligible,
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
/// be cast right now.
///
/// Two timing paths are checked:
///
/// **Instant-speed** (always available with priority):
/// - The card is an Instant, OR has the Flash static ability.
/// - Player can pay the mana cost.
///
/// **Sorcery-speed** (restricted):
/// 1. Player has priority.
/// 2. Player is the active player (their turn).
/// 3. Current step is a main phase (FirstMain or SecondMain).
/// 4. Stack is empty.
/// 5. Player can pay the mana cost.
///
/// Returns the union of both paths. Lands are never included (CR 305.1).
pub(crate) fn compute_castable_spells(game: &Game, player_id: &str) -> Vec<String> {
    // Both paths require priority.
    if game.priority_player_id() != Some(player_id) {
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

    // Pre-compute sorcery-speed conditions (shared across all cards).
    let is_active = game.current_player_id() == player_id;
    let in_main = matches!(game.current_step(), Step::FirstMain | Step::SecondMain);
    let stack_empty = !game.stack_has_items();
    let sorcery_speed_ok = is_active && in_main && stack_empty;

    hand.iter()
        .filter(|card| {
            // Lands are played, not cast (CR 305.1).
            if card.definition().types().contains(&CardType::Land) {
                return false;
            }

            let cost = card
                .definition()
                .mana_cost()
                .cloned()
                .unwrap_or_else(ManaCost::zero);

            if !can_pay_cost(mana_pool, &cost) {
                return false;
            }

            // Instant-speed: Instant type or Flash keyword — always castable with priority.
            let is_instant_speed = card.definition().types().contains(&CardType::Instant)
                || card
                    .definition()
                    .static_abilities()
                    .contains(&StaticAbility::Flash);
            if is_instant_speed {
                return true;
            }

            // Sorcery-speed: active player, main phase, empty stack.
            sorcery_speed_ok
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

/// Returns `true` if the player has priority but cannot take any meaningful
/// instant-speed action.
///
/// The Bevy layer uses this to auto-pass priority without requiring the player
/// to click "Pass Priority" when they have no legal actions at all.
///
/// Auto-pass is eligible when ALL of the following are true:
/// 1. Player has priority.
/// 2. No spells are castable (`compute_castable_spells` is empty).
/// 3. No lands are playable (`compute_playable_lands` is empty).
/// 4. No creatures are attackable (`compute_attackable_creatures` is empty).
/// 5. No creatures are blockable (`compute_blockable_creatures` is empty).
pub fn compute_auto_pass_eligible(game: &Game, player_id: &str) -> bool {
    if game.priority_player_id() != Some(player_id) {
        return false;
    }
    let actions = compute_legal_actions(game, player_id);
    // Potential plays: tappable lands + spells, but ONLY during own
    // sorcery-speed timing. During opponent's turn, only castable_spells
    // (mana already in pool) stops auto-pass. This matches Arena's default
    // behavior: auto-pass everything unless you actively have mana up.
    // Players who want to "hold up mana" must tap before the opponent acts.
    let is_own_sorcery_timing = game.current_player_id() == player_id
        && matches!(game.current_step(), Step::FirstMain | Step::SecondMain)
        && !game.stack_has_items();

    let has_potential_plays = is_own_sorcery_timing
        && !actions.tappable_lands.is_empty()
        && game.hand(player_id)
            .map(|hand| hand.iter().any(|c| !c.definition().is_land()))
            .unwrap_or(false);

    actions.playable_lands.is_empty()
        && actions.castable_spells.is_empty()
        && !has_potential_plays
        && actions.attackable_creatures.is_empty()
        && actions.blockable_creatures.is_empty()
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

    // ---- instant-speed casting: compute_castable_spells --------------------

    /// Helper: advance a started empty-deck game to FirstMain for p1.
    fn make_empty_game_in_first_main() -> (Game, String, String) {
        use crate::domain::game::test_helpers::make_game_in_first_main;
        make_game_in_first_main()
    }

    /// Helper: make a lightning-strike-like instant card instance.
    fn make_instant_card(instance_id: &str, owner_id: &str) -> crate::domain::cards::card_instance::CardInstance {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;
        let cost = crate::domain::value_objects::mana::ManaCost::parse("1R").unwrap();
        let def = CardDefinition::new("instant-test", "Test Instant", vec![CardType::Instant])
            .with_mana_cost(cost);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Helper: make a sorcery card instance.
    fn make_sorcery_card(instance_id: &str, owner_id: &str) -> crate::domain::cards::card_instance::CardInstance {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;
        let cost = crate::domain::value_objects::mana::ManaCost::parse("1R").unwrap();
        let def = CardDefinition::new("sorcery-test", "Test Sorcery", vec![CardType::Sorcery])
            .with_mana_cost(cost);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Helper: make a creature card with Flash.
    fn make_flash_creature_card(instance_id: &str, owner_id: &str) -> crate::domain::cards::card_instance::CardInstance {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::{CardType, StaticAbility};
        let cost = crate::domain::value_objects::mana::ManaCost::parse("1G").unwrap();
        let def = CardDefinition::new("flash-creature", "Flash Creature", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_mana_cost(cost)
            .with_static_ability(StaticAbility::Flash);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Helper: make a plain creature card (no flash, sorcery-speed).
    fn make_creature_card_with_cost(instance_id: &str, owner_id: &str) -> crate::domain::cards::card_instance::CardInstance {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;
        let cost = crate::domain::value_objects::mana::ManaCost::parse("1G").unwrap();
        let def = CardDefinition::new("creature-test", "Test Creature", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_mana_cost(cost);
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn instant_in_hand_is_castable_during_opponents_turn_with_mana() {
        // p1 is active. After p1 passes, p2 has priority during p1's main phase.
        // p2 holds an instant and has mana — it must be castable.
        let (mut game, p1, p2) = make_empty_game_in_first_main();
        let instant = make_instant_card("instant-1", &p2);
        game.add_card_to_hand(&p2, instant).unwrap();
        game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p2, ManaColor::Red, 1).unwrap();

        // p1 passes priority → p2 gets priority
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.priority_player_id(), Some(p2.as_str()));
        let castable = compute_castable_spells(&game, &p2);
        assert!(
            !castable.is_empty(),
            "p2 should be able to cast instant during p1's main phase"
        );
        assert!(
            castable.contains(&"instant-1".to_owned()),
            "instant-1 should be castable"
        );
    }

    #[test]
    fn instant_in_hand_is_castable_during_combat_step_with_mana() {
        // Advance p1's turn to DeclareAttackers step, then pass priority to p2.
        let (mut game, p1, p2) = make_empty_game_in_first_main();
        let instant = make_instant_card("instant-1", &p2);
        game.add_card_to_hand(&p2, instant).unwrap();
        game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p2, ManaColor::Red, 1).unwrap();

        // Advance from FirstMain through combat steps to DeclareAttackers
        // FirstMain → BeginningOfCombat → DeclareAttackers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → BeginningOfCombat
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → DeclareAttackers

        // p1 passes → p2 gets priority during DeclareAttackers
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.current_step(), Step::DeclareAttackers);
        assert_eq!(game.priority_player_id(), Some(p2.as_str()));

        let castable = compute_castable_spells(&game, &p2);
        assert!(
            castable.contains(&"instant-1".to_owned()),
            "instant should be castable during DeclareAttackers step"
        );
    }

    #[test]
    fn instant_is_castable_with_spell_on_stack() {
        // p1 casts a creature → stack has a spell → p1 retains priority (CR 117.3c) → passes → p2 gets priority
        // p2 holds instant + mana → instant should be castable
        let (mut game, p1, p2) = make_empty_game_in_first_main();
        let creature = make_creature_card_with_cost("bear-1", &p1);
        game.add_card_to_hand(&p1, creature).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();

        let instant = make_instant_card("instant-1", &p2);
        game.add_card_to_hand(&p2, instant).unwrap();
        game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p2, ManaColor::Red, 1).unwrap();

        // p1 casts the creature → on the stack
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: crate::domain::types::CardInstanceId::new("bear-1"),
            targets: vec![],
        })
        .unwrap();

        assert!(game.stack_has_items(), "creature should be on the stack");

        // Per CR 117.3c, p1 retains priority. p1 passes → p2 gets priority.
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.priority_player_id(), Some(p2.as_str()));
        let castable = compute_castable_spells(&game, &p2);
        assert!(
            castable.contains(&"instant-1".to_owned()),
            "p2 should be able to cast instant in response to spell on stack"
        );
    }

    #[test]
    fn sorcery_is_not_castable_during_combat_step() {
        // p1 is active at DeclareAttackers. A sorcery card in p1's hand should NOT be castable.
        let (mut game, p1, _p2) = make_empty_game_in_first_main();
        let sorcery = make_sorcery_card("sorcery-1", &p1);
        game.add_card_to_hand(&p1, sorcery).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();

        // Advance to DeclareAttackers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → BeginningOfCombat
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → DeclareAttackers

        assert_eq!(game.current_step(), Step::DeclareAttackers);
        assert_eq!(game.priority_player_id(), Some(p1.as_str()));

        let castable = compute_castable_spells(&game, &p1);
        assert!(
            !castable.contains(&"sorcery-1".to_owned()),
            "sorcery should NOT be castable during DeclareAttackers"
        );
    }

    #[test]
    fn sorcery_is_not_castable_with_spell_on_stack() {
        // p1 casts a creature → p1 retains priority. Sorcery should not be castable with a spell on stack.
        let (mut game, p1, _p2) = make_empty_game_in_first_main();
        let creature = make_creature_card_with_cost("bear-1", &p1);
        game.add_card_to_hand(&p1, creature).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();

        // Put a second sorcery in p1's hand
        let sorcery = make_sorcery_card("sorcery-1", &p1);
        game.add_card_to_hand(&p1, sorcery).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();

        // Cast the creature → it goes on the stack. p1 retains priority.
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: crate::domain::types::CardInstanceId::new("bear-1"),
            targets: vec![],
        })
        .unwrap();

        assert!(game.stack_has_items());

        let castable = compute_castable_spells(&game, &p1);
        assert!(
            !castable.contains(&"sorcery-1".to_owned()),
            "sorcery should NOT be castable with spell on stack"
        );
    }

    #[test]
    fn flash_creature_is_castable_during_opponents_turn() {
        // p2 holds a Flash creature. After p1 passes priority in main phase, p2 can cast it.
        let (mut game, p1, p2) = make_empty_game_in_first_main();
        let flash_creature = make_flash_creature_card("flash-1", &p2);
        game.add_card_to_hand(&p2, flash_creature).unwrap();
        game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p2, ManaColor::Green, 1).unwrap();

        // p1 passes priority → p2 gets priority
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.priority_player_id(), Some(p2.as_str()));
        let castable = compute_castable_spells(&game, &p2);
        assert!(
            castable.contains(&"flash-1".to_owned()),
            "Flash creature should be castable during opponent's turn"
        );
    }

    // ---- compute_auto_pass_eligible ----------------------------------------

    #[test]
    fn auto_pass_eligible_when_no_actions_available() {
        // p1 in first main, no cards, no mana → auto-pass should be true
        let (game, p1, _p2) = make_empty_game_in_first_main();
        assert!(
            compute_auto_pass_eligible(&game, &p1),
            "Player with no actions should be auto-pass eligible"
        );
    }

    #[test]
    fn auto_pass_not_eligible_when_has_instant_and_mana() {
        // p2 has priority, holds instant + mana → should NOT auto-pass
        let (mut game, p1, p2) = make_empty_game_in_first_main();
        let instant = make_instant_card("instant-1", &p2);
        game.add_card_to_hand(&p2, instant).unwrap();
        game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p2, ManaColor::Red, 1).unwrap();

        // p1 passes → p2 gets priority
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert!(
            !compute_auto_pass_eligible(&game, &p2),
            "Player with castable instant should NOT be auto-pass eligible"
        );
    }

    #[test]
    fn auto_pass_not_eligible_when_has_playable_land() {
        // p1 in first main, has a land in hand → should NOT auto-pass
        let (mut game, p1, _p2) = make_empty_game_in_first_main();
        let land = {
            use crate::domain::cards::card_definition::CardDefinition;
            use crate::domain::cards::card_instance::CardInstance;
            let def = CardDefinition::new("forest", "Forest", vec![crate::domain::enums::CardType::Land]);
            CardInstance::new("land-1", def, &p1)
        };
        game.add_card_to_hand(&p1, land).unwrap();

        assert!(
            !compute_auto_pass_eligible(&game, &p1),
            "Player with playable land should NOT be auto-pass eligible"
        );
    }

    #[test]
    fn auto_pass_not_eligible_when_has_attackable_creatures() {
        // p1 in DeclareAttackers, has an attackable creature → should NOT auto-pass
        let (mut game, p1, _p2) = make_empty_game_in_first_main();

        // Add a ready creature to p1's battlefield
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;
        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let creature = CardInstance::new("bear-1", def, &p1);
        game.add_permanent_to_battlefield(&p1, creature).unwrap();
        // Clear summoning sickness
        crate::domain::game::test_helpers::clear_summoning_sickness(&mut game, "bear-1");

        // Advance to DeclareAttackers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → BeginningOfCombat
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → DeclareAttackers

        assert_eq!(game.current_step(), Step::DeclareAttackers);
        assert!(
            !compute_auto_pass_eligible(&game, &p1),
            "Player with attackable creatures should NOT be auto-pass eligible"
        );
    }

    #[test]
    fn auto_pass_not_eligible_after_tapping_land_with_castable_creature_in_hand() {
        // Reproduces the bug: player at FirstMain, taps land for mana,
        // has a creature in hand that costs exactly that mana.
        // After tapping, auto-pass should NOT fire.
        use crate::domain::abilities::{ActivatedAbility, ActivationCost};
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::effects::Effect;
        use crate::domain::enums::CardType;
        use crate::domain::types::CardInstanceId;

        let (mut game, p1, _p2) = make_empty_game_in_first_main();
        assert_eq!(game.current_step(), Step::FirstMain);
        assert_eq!(game.priority_player_id(), Some(p1.as_str()));

        // Add a Mountain to P1's battlefield
        let mountain = CardInstance::new(
            "mtn-1",
            CardDefinition::new("mountain", "Mountain", vec![CardType::Land])
                .with_activated_ability(ActivatedAbility {
                    cost: ActivationCost::Tap,
                    effect: Effect::AddMana { color: ManaColor::Red, amount: 1 },
                }),
            &p1,
        );
        game.add_permanent_to_battlefield(&p1, mountain).unwrap();

        // Add a Goblin ({R}) to P1's hand
        let goblin = CardInstance::new(
            "goblin-1",
            CardDefinition::new("goblin", "Goblin", vec![CardType::Creature])
                .with_power_toughness(1, 1)
                .with_mana_cost(crate::domain::value_objects::mana::ManaCost::parse("R").unwrap()),
            &p1,
        );
        game.add_card_to_hand(&p1, goblin).unwrap();

        // Before tapping: has tappable land → NOT auto-pass
        assert!(
            !compute_auto_pass_eligible(&game, &p1),
            "Before tapping: has tappable land, should NOT auto-pass"
        );

        // Tap the Mountain → R in pool
        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("mtn-1"),
        }).unwrap();

        // After tapping: R in pool, Goblin ({R}) in hand → castable!
        let castable = compute_castable_spells(&game, &p1);
        assert!(
            castable.iter().any(|id| id == "goblin-1"),
            "Goblin should be castable with R in pool. Castable: {:?}, step: {:?}, active: {}, priority: {:?}",
            castable, game.current_step(), game.current_player_id(), game.priority_player_id()
        );

        assert!(
            !compute_auto_pass_eligible(&game, &p1),
            "After tapping: has castable Goblin, should NOT auto-pass"
        );
    }

    #[test]
    fn auto_pass_eligible_field_set_in_allowed_actions_result() {
        // AllowedActionsResult.auto_pass_eligible should match compute_auto_pass_eligible
        let (game, p1, _p2) = make_empty_game_in_first_main();
        let actions = compute_legal_actions(&game, &p1);
        assert_eq!(
            actions.auto_pass_eligible,
            compute_auto_pass_eligible(&game, &p1),
            "auto_pass_eligible in AllowedActionsResult must match compute_auto_pass_eligible"
        );
    }
}
