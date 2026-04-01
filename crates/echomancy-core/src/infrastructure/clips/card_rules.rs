//! Card-specific CLIPS rules registry.
//!
//! Maps card definition IDs to their embedded `.clp` rule strings.
//! Card rules are loaded on demand via `load_card_rules()`.
//!
//! # File organization
//!
//! Card rule files live under `rules/cards/<first-letter>/<card-id>.clp`.
//! Each file is embedded at compile time with `include_str!()`.
//! Vanilla creatures (Bear, Goblin, Elite Vanguard) and basic lands
//! have no `.clp` file — their behaviour is handled entirely by Rust.
//!
//! # Usage
//!
//! ```ignore
//! let mut engine = ClipsEngine::new()?;
//! load_core_templates(&mut engine)?;
//! load_card_rules(&mut engine, "lightning-strike")?;
//! load_card_rules(&mut engine, "divination")?;
//! ```

use crate::domain::rules_engine::RulesError;
use crate::infrastructure::clips::ClipsEngine;

// ============================================================================
// Embedded card rule files
// ============================================================================

const LIGHTNING_STRIKE_RULES: &str =
    include_str!("../../../../../rules/cards/l/lightning-strike.clp");

const DIVINATION_RULES: &str =
    include_str!("../../../../../rules/cards/d/divination.clp");

const GIANT_GROWTH_RULES: &str =
    include_str!("../../../../../rules/cards/g/giant-growth.clp");

const WILD_BOUNTY_RULES: &str =
    include_str!("../../../../../rules/cards/w/wild-bounty.clp");

const CANCEL_RULES: &str =
    include_str!("../../../../../rules/cards/c/cancel.clp");

// ============================================================================
// Public API
// ============================================================================

/// Load the core CLIPS deftemplate definitions into the engine.
///
/// Must be called before loading any card rules or asserting game facts.
/// Calling this more than once is safe — CLIPS deduplicates deftemplates
/// identified by name, so a second call on the same environment is a no-op.
#[allow(dead_code)]
pub(crate) fn load_core_templates(engine: &mut ClipsEngine) -> Result<(), RulesError> {
    const TEMPLATES: &str = include_str!("../../../../../rules/core/templates.clp");
    engine.load_rules(TEMPLATES)
}

/// Load card-specific CLIPS rules for the given card definition ID.
///
/// Returns `Ok(())` for cards that have no `.clp` file (vanilla creatures,
/// basic lands). Only returns `Err` when the card has rules but they fail to
/// load (syntax error in the `.clp` file).
///
/// # Supported cards
///
/// | Card ID | Effect |
/// |---------|--------|
/// | `"lightning-strike"` | Deal 3 damage to opponent |
/// | `"divination"` | Controller draws 2 cards |
/// | `"giant-growth"` | Target creature gets +3/+3 until end of turn |
/// | `"wild-bounty"` | Controller draws 1 card on ETB |
/// | anything else | No rules — silently succeeds |
#[allow(dead_code)]
pub(crate) fn load_card_rules(engine: &mut ClipsEngine, card_id: &str) -> Result<(), RulesError> {
    match card_id {
        "lightning-strike" => engine.load_rules(LIGHTNING_STRIKE_RULES),
        "divination" => engine.load_rules(DIVINATION_RULES),
        "giant-growth" => engine.load_rules(GIANT_GROWTH_RULES),
        "wild-bounty" => engine.load_rules(WILD_BOUNTY_RULES),
        "cancel" => engine.load_rules(CANCEL_RULES),
        // Sol Ring uses no CLIPS rule — its mana ability is handled entirely by
        // the Rust domain (CR 605 mana abilities bypass the stack).
        // Vanilla/keyword-only cards have no .clp — this is expected and fine.
        _ => Ok(()),
    }
}

// ============================================================================
// Tests (TDD: written before implementation to drive the API design)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::CardType;
    use crate::domain::game::test_helpers::{add_card_to_hand, make_game_in_first_main, make_land_card};
    use crate::domain::rules_engine::{RulesAction, RulesEngine};
    use crate::domain::types::{CardInstanceId, PlayerId};

    // -------------------------------------------------------------------------
    // Helper: build an engine with core templates + requested card rules
    // -------------------------------------------------------------------------

    fn engine_for(card_ids: &[&str]) -> ClipsEngine {
        let mut engine = ClipsEngine::new().expect("engine creation");
        load_core_templates(&mut engine).expect("core templates should load");
        for &id in card_ids {
            load_card_rules(&mut engine, id)
                .unwrap_or_else(|e| panic!("failed to load rules for {id}: {e}"));
        }
        engine
    }

    // =========================================================================
    // Infrastructure: load_core_templates / load_card_rules
    // =========================================================================

    #[test]
    fn load_core_templates_succeeds() {
        let mut engine = ClipsEngine::new().expect("engine");
        load_core_templates(&mut engine).expect("core templates should load without error");
    }

    #[test]
    fn load_card_rules_for_lightning_strike_succeeds() {
        let mut engine = ClipsEngine::new().expect("engine");
        load_core_templates(&mut engine).expect("core templates");
        load_card_rules(&mut engine, "lightning-strike")
            .expect("lightning-strike rules should load");
    }

    #[test]
    fn load_card_rules_for_divination_succeeds() {
        let mut engine = ClipsEngine::new().expect("engine");
        load_core_templates(&mut engine).expect("core templates");
        load_card_rules(&mut engine, "divination")
            .expect("divination rules should load");
    }

    #[test]
    fn load_card_rules_for_unknown_card_is_no_op() {
        // Vanilla creatures (bear, goblin, elite-vanguard) have no .clp file.
        // load_card_rules should return Ok without loading anything.
        let mut engine = ClipsEngine::new().expect("engine");
        load_core_templates(&mut engine).expect("core templates");
        for &vanilla in &["bear", "goblin", "elite-vanguard", "forest", "mountain"] {
            load_card_rules(&mut engine, vanilla)
                .unwrap_or_else(|e| panic!("vanilla card {vanilla} should not fail: {e}"));
        }
    }

    // =========================================================================
    // CLIPS rule correctness: Lightning Strike
    // =========================================================================

    /// Lightning Strike rule fires and produces an action-damage fact targeting the chosen player.
    ///
    /// The rule reads the target from the `stack-item` fact (not from a player heuristic).
    #[test]
    fn lightning_strike_rule_produces_damage_action_for_opponent() {
        use crate::domain::events::GameEvent;
        use crate::domain::events::CardInstanceSnapshot;
        use crate::domain::targets::Target;
        use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

        let mut engine = engine_for(&["lightning-strike"]);
        let (mut game, p1, p2) = make_game_in_first_main();

        // Put a free Lightning Strike (with AnyTarget requirement) in p1's hand.
        let strike = CardInstance::new(
            "strike-1",
            CardDefinition::new("lightning-strike", "Lightning Strike", vec![CardType::Instant])
                .with_target_requirement(crate::domain::targets::TargetRequirement::AnyTarget),
            &p1,
        );
        add_card_to_hand(&mut game, &p1, strike);

        // Cast targeting p2 — this puts the spell on the stack with targets=[p2].
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("strike-1"),
            targets: vec![Target::player(&p2)],
        })
        .expect("cast should succeed");

        assert_eq!(game.stack().len(), 1, "spell should be on stack");

        // Synthesize a SpellResolved event for lightning-strike cast by p1.
        // Include the target so the CLIPS rule reads target-id from the game-event.
        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("strike-1"),
                definition_id: CardDefinitionId::new("lightning-strike"),
                owner_id: PlayerId::new(&p1),
            },
            controller_id: PlayerId::new(&p1),
            targets: vec![Target::player(&p2)],
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert_eq!(
            result.actions.len(),
            1,
            "should produce exactly one action-damage"
        );
        assert!(
            matches!(
                &result.actions[0],
                RulesAction::DealDamage { source, target, amount: 3 }
                    if source == "strike-1" && target == p2.as_str()
            ),
            "should deal 3 damage to p2 (chosen target), got: {:?}",
            result.actions[0]
        );
    }

    /// Lightning Strike rule does not fire for a different card.
    #[test]
    fn lightning_strike_rule_does_not_fire_for_other_cards() {
        use crate::domain::events::GameEvent;
        use crate::domain::events::CardInstanceSnapshot;
        use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
        use crate::domain::game::test_helpers::make_started_game;

        let mut engine = engine_for(&["lightning-strike"]);
        let (game, p1, p2) = make_started_game();

        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("div-1"),
                definition_id: CardDefinitionId::new("divination"), // different card
                owner_id: PlayerId::new(&p1),
            },
            controller_id: PlayerId::new(&p1),
            targets: vec![crate::domain::targets::Target::player(&p2)],
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert!(
            result.actions.is_empty(),
            "lightning-strike rule should not fire for divination"
        );
    }

    // =========================================================================
    // CLIPS rule correctness: Divination
    // =========================================================================

    /// Divination rule fires and produces an action-draw fact for the controller.
    #[test]
    fn divination_rule_produces_draw_action_for_controller() {
        use crate::domain::events::GameEvent;
        use crate::domain::events::CardInstanceSnapshot;
        use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
        use crate::domain::game::test_helpers::make_started_game;

        let mut engine = engine_for(&["divination"]);
        let (game, p1, _p2) = make_started_game();

        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("div-1"),
                definition_id: CardDefinitionId::new("divination"),
                owner_id: PlayerId::new(&p1),
            },
            controller_id: PlayerId::new(&p1),
            targets: vec![],
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert_eq!(
            result.actions.len(),
            1,
            "should produce exactly one action-draw"
        );
        assert!(
            matches!(
                &result.actions[0],
                RulesAction::DrawCards { player, amount: 2 }
                    if player == "p1"
            ),
            "should draw 2 cards for p1 (controller), got: {:?}",
            result.actions[0]
        );
    }

    /// Divination rule does not fire for a different card.
    #[test]
    fn divination_rule_does_not_fire_for_other_cards() {
        use crate::domain::events::GameEvent;
        use crate::domain::events::CardInstanceSnapshot;
        use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
        use crate::domain::game::test_helpers::make_started_game;

        let mut engine = engine_for(&["divination"]);
        let (game, p1, _p2) = make_started_game();

        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("strike-1"),
                definition_id: CardDefinitionId::new("lightning-strike"), // different card
                owner_id: PlayerId::new(&p1),
            },
            controller_id: PlayerId::new(&p1),
            targets: vec![],
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert!(
            result.actions.is_empty(),
            "divination rule should not fire for lightning-strike"
        );
    }

    // =========================================================================
    // CLIPS rule correctness: Giant Growth
    // =========================================================================

    /// Giant Growth CLIPS rule loads without error.
    #[test]
    fn load_card_rules_for_giant_growth_succeeds() {
        let mut engine = ClipsEngine::new().expect("engine");
        load_core_templates(&mut engine).expect("core templates");
        load_card_rules(&mut engine, "giant-growth").expect("giant-growth rules should load");
    }

    /// Giant Growth rule fires and produces an action-modify-pt fact for the target.
    #[test]
    fn giant_growth_rule_produces_modify_pt_action() {
        use crate::domain::events::GameEvent;
        use crate::domain::events::CardInstanceSnapshot;
        use crate::domain::rules_engine::RulesAction;
        use crate::domain::targets::Target;
        use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

        let mut engine = engine_for(&["giant-growth"]);
        let (game, p1, _p2) = make_game_in_first_main();

        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("gg-1"),
                definition_id: CardDefinitionId::new("giant-growth"),
                owner_id: PlayerId::new(&p1),
            },
            controller_id: PlayerId::new(&p1),
            targets: vec![Target::creature("creature-1")],
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert_eq!(result.actions.len(), 1, "should produce exactly one action-modify-pt");
        assert!(
            matches!(
                &result.actions[0],
                RulesAction::ModifyPowerToughness { source, target, power: 3, toughness: 3, .. }
                    if source == "gg-1" && target == "creature-1"
            ),
            "should produce +3/+3 for the target, got: {:?}",
            result.actions[0]
        );
    }

    /// Giant Growth rule does not fire when no target is provided.
    #[test]
    fn giant_growth_rule_does_not_fire_without_target() {
        use crate::domain::events::GameEvent;
        use crate::domain::events::CardInstanceSnapshot;
        use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
        use crate::domain::game::test_helpers::make_started_game;

        let mut engine = engine_for(&["giant-growth"]);
        let (game, p1, _p2) = make_started_game();

        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("gg-1"),
                definition_id: CardDefinitionId::new("giant-growth"),
                owner_id: PlayerId::new(&p1),
            },
            controller_id: PlayerId::new(&p1),
            targets: vec![], // no target
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert!(
            result.actions.is_empty(),
            "giant-growth rule should not fire without a target"
        );
    }

    /// Full pipeline: Giant Growth cast on p1's creature gives +3/+3 until end of turn.
    #[test]
    fn e2e_giant_growth_boosts_creature_power_and_toughness() {
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;
        use crate::domain::targets::Target;

        let (mut game, p1, p2) = make_game_in_first_main();

        // Wire CLIPS
        let engine = engine_for(&["giant-growth"]);
        game.set_rules_engine(Box::new(engine));

        // Put a 2/2 creature on p1's battlefield
        let creature_def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let creature = CardInstance::new("bear-1", creature_def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        // Verify base P/T
        let ps = game.permanent_state("bear-1").cloned().unwrap();
        assert_eq!(ps.current_power().unwrap(), 2);
        assert_eq!(ps.current_toughness().unwrap(), 2);

        // Put a Giant Growth in p1's hand (free, with target requirement)
        let gg_def = CardDefinition::new("giant-growth", "Giant Growth", vec![CardType::Instant])
            .with_target_requirement(crate::domain::targets::TargetRequirement::Creature);
        let gg = CardInstance::new("gg-1", gg_def, &p1);
        add_card_to_hand(&mut game, &p1, gg);

        // Cast Giant Growth targeting bear-1
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("gg-1"),
            targets: vec![Target::creature("bear-1")],
        })
        .expect("cast should succeed");

        // Both players pass priority — stack resolves
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p2) }).unwrap();

        assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

        // Creature should now be 5/5
        let ps = game.permanent_state("bear-1").cloned().unwrap();
        assert_eq!(
            ps.current_power().unwrap(),
            5,
            "Giant Growth should boost power to 5"
        );
        assert_eq!(
            ps.current_toughness().unwrap(),
            5,
            "Giant Growth should boost toughness to 5"
        );
        assert_eq!(
            ps.continuous_effects().len(),
            1,
            "should have one active continuous effect"
        );
    }

    /// After advancing to Cleanup, Giant Growth's +3/+3 effect expires.
    #[test]
    fn e2e_giant_growth_expires_at_cleanup() {
        use crate::domain::enums::Step;
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;
        use crate::domain::targets::Target;

        let (mut game, p1, p2) = make_game_in_first_main();

        let engine = engine_for(&["giant-growth"]);
        game.set_rules_engine(Box::new(engine));

        // Put a 2/2 creature on p1's battlefield
        let creature_def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let creature = CardInstance::new("bear-1", creature_def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        // Cast Giant Growth targeting bear-1
        let gg_def = CardDefinition::new("giant-growth", "Giant Growth", vec![CardType::Instant])
            .with_target_requirement(crate::domain::targets::TargetRequirement::Creature);
        let gg = CardInstance::new("gg-1", gg_def, &p1);
        add_card_to_hand(&mut game, &p1, gg);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("gg-1"),
            targets: vec![Target::creature("bear-1")],
        })
        .expect("cast should succeed");

        game.apply(Action::PassPriority { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p2) }).unwrap();

        // Verify boost is active
        let ps = game.permanent_state("bear-1").cloned().unwrap();
        assert_eq!(ps.current_power().unwrap(), 5, "should be boosted before cleanup");

        // Advance to Cleanup step
        // We need to pass through: SecondMain → EndStep → Cleanup
        // Advance past FirstMain
        while game.current_step() != Step::Cleanup {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .expect("advance step should succeed");
        }

        // After reaching cleanup, effects expire
        // Note: on_enter_step(Cleanup) calls expire_continuous_effects
        let ps = game.permanent_state("bear-1").cloned().unwrap();
        assert_eq!(
            ps.continuous_effects().len(),
            0,
            "continuous effects should have expired at cleanup"
        );
        assert_eq!(
            ps.current_power().unwrap(),
            2,
            "power should return to base 2 after cleanup"
        );
        assert_eq!(
            ps.current_toughness().unwrap(),
            2,
            "toughness should return to base 2 after cleanup"
        );
    }

    // =========================================================================
    // End-to-end pipeline: Game → CLIPS → Game mutation
    // =========================================================================

    /// Full pipeline: Lightning Strike cast by p1 deals 3 damage to p2.
    ///
    /// This test exercises the complete chain:
    ///   game.apply(CastSpell) → stack push
    ///   game.apply(PassPriority) × 2 → stack resolves
    ///   → resolve_spell() → engine.evaluate() → CLIPS fires
    ///   → apply_rules_action() → p2 loses 3 life
    #[test]
    fn e2e_lightning_strike_deals_3_damage_to_opponent() {
        let (mut game, p1, p2) = make_game_in_first_main();

        // Wire the CLIPS engine into the game
        let engine = engine_for(&["lightning-strike"]);
        game.set_rules_engine(Box::new(engine));

        // Put a free Lightning Strike in p1's hand (no mana cost to avoid payment logic).
        // The definition must declare AnyTarget so the cast-spell handler accepts the target.
        let strike = CardInstance::new(
            "strike-1",
            CardDefinition::new("lightning-strike", "Lightning Strike", vec![CardType::Instant])
                .with_target_requirement(crate::domain::targets::TargetRequirement::AnyTarget),
            &p1,
        );
        add_card_to_hand(&mut game, &p1, strike);

        let p2_life_before = game.player_life_total(&p2).unwrap();
        assert_eq!(p2_life_before, 20);

        // Cast the spell targeting p2 — goes on stack, priority passes to p2.
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("strike-1"),
            targets: vec![crate::domain::targets::Target::player(&p2)],
        })
        .expect("p1 should be able to cast Lightning Strike");

        assert_eq!(game.stack().len(), 1, "spell should be on stack");

        // CR 117.3c: caster (p1) retains priority after casting. p1 passes first.
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p1) })
            .expect("p1 (caster) should be able to pass priority");
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p2) })
            .expect("p2 should be able to pass priority");

        // After resolution: stack empty, p2 has 17 life
        assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

        let p2_life_after = game.player_life_total(&p2).unwrap();
        assert_eq!(
            p2_life_after,
            p2_life_before - 3,
            "p2 should have taken 3 damage from Lightning Strike"
        );

        let p1_life_after = game.player_life_total(&p1).unwrap();
        assert_eq!(p1_life_after, 20, "p1 (caster) should be unaffected");
    }

    /// Full pipeline: Divination cast by p1 draws 2 cards for p1.
    #[test]
    fn e2e_divination_draws_2_cards_for_controller() {
        let (mut game, p1, p2) = make_game_in_first_main();

        let engine = engine_for(&["divination"]);
        game.set_rules_engine(Box::new(engine));

        // Add 2 cards to p1's library so draw doesn't fail silently
        let lib_1 = make_land_card("lib-1", &p1);
        let lib_2 = make_land_card("lib-2", &p1);
        game.player_state_mut(&p1).unwrap().library.push(lib_1);
        game.player_state_mut(&p1).unwrap().library.push(lib_2);

        // Free Divination in p1's hand
        let divination = CardInstance::new(
            "div-1",
            CardDefinition::new("divination", "Divination", vec![CardType::Sorcery]),
            &p1,
        );
        add_card_to_hand(&mut game, &p1, divination);

        // hand starts empty (0) + 1 divination = 1 card
        // after cast: 0 in hand (divination leaves), + 2 draws = 2 in hand
        let p1_hand_before = game.hand(&p1).unwrap().len(); // 1 (just the divination)

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("div-1"),
            targets: vec![],
        })
        .expect("cast should succeed");

        // CR 117.3c: caster (p1) retains priority. p1 passes first, then p2.
        // Both pass → stack resolves → CLIPS fires → 2 cards drawn
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::PassPriority { player_id: PlayerId::new(&p2) }).unwrap();

        assert_eq!(game.stack().len(), 0, "stack should be empty");

        let p1_hand_after = game.hand(&p1).unwrap().len();
        // The spell itself left hand when cast, so net = -1 (cast) + 2 (draw) = +1
        // But we want to verify 2 cards were drawn:
        // hand_before included the spell, hand_after should be hand_before - 1 (spell) + 2 (draws)
        assert_eq!(
            p1_hand_after,
            p1_hand_before - 1 + 2,
            "p1 should have drawn 2 cards: was {p1_hand_before}, expected {}, got {p1_hand_after}",
            p1_hand_before - 1 + 2,
        );

        // p2 should not have drawn any cards
        let p2_hand_after = game.hand(&p2).unwrap().len();
        assert_eq!(p2_hand_after, 0, "opponent (p2) should not draw any cards");
    }
}
