//! Stack and spell resolution helpers for the `Game` aggregate.

use crate::domain::entities::the_stack::{AbilityOnStack, SpellOnStack, StackItem};
use crate::domain::enums::{GraveyardReason, ManaColor, ZoneName};
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::rules_engine::RulesAction;
use crate::domain::services::trigger_evaluation::{PermanentOnBattlefield, TriggeredAbilityInfo};
use crate::domain::triggers::TriggerEventType;
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
use crate::domain::value_objects::permanent_state::{ContinuousEffect, EffectDuration};

use super::Game;

impl Game {
    /// Resolve the top item on the stack.
    pub(crate) fn resolve_top_of_stack(&mut self) -> Vec<GameEvent> {
        if self.stack.is_empty() {
            return Vec::new();
        }
        let stack_item = match self.stack.pop() {
            Some(item) => item,
            None => return Vec::new(),
        };

        let mut events = match stack_item {
            StackItem::Spell(spell) => self.resolve_spell(spell),
            StackItem::Ability(ability) => self.resolve_ability(ability),
        };

        self.players_who_passed_priority.clear();
        let current_player = self.turn_state.current_player_id().as_str().to_owned();
        events.extend(self.assign_priority_to(&current_player));
        events
    }

    pub(crate) fn collect_triggered_abilities(&self, event: &GameEvent) -> Vec<TriggeredAbilityInfo> {
        use crate::domain::services::trigger_evaluation::find_matching_triggers;

        let all_permanents: Vec<PermanentOnBattlefield<'_>> = self
            .players
            .iter()
            .flat_map(|p| {
                p.battlefield.iter().map(move |card| PermanentOnBattlefield {
                    permanent: card,
                    controller_id: p.player_id.as_str(),
                })
            })
            .collect();

        find_matching_triggers(&all_permanents, event)
    }

    pub(crate) fn execute_triggered_abilities(&mut self, abilities: Vec<TriggeredAbilityInfo>) {
        if abilities.is_empty() || self.rules_engine.is_none() {
            return;
        }

        for ability in abilities {
            let source_snapshot = CardInstanceSnapshot {
                instance_id: CardInstanceId::new(&ability.source_id),
                definition_id: CardDefinitionId::new(&ability.source_definition_id),
                owner_id: PlayerId::new(&ability.source_owner_id),
            };
            let trigger_type = trigger_type_string(&ability.trigger_event_type);
            let trigger_event = GameEvent::TriggeredAbilityFires {
                source: source_snapshot,
                controller_id: PlayerId::new(&ability.controller_id),
                trigger_type,
            };

            // Take the engine out to avoid a simultaneous &mut self borrow
            // (same pattern as resolve_spell). Put it back unconditionally.
            let mut engine = match self.rules_engine.take() {
                Some(e) => e,
                None => return,
            };
            match engine.evaluate(self, &trigger_event) {
                Ok(result) => {
                    let actions: Vec<crate::domain::rules_engine::RulesAction> =
                        result.actions.clone();
                    for action in &actions {
                        self.apply_rules_action(action);
                    }
                }
                Err(_) => {
                    // Log error but don't crash — CLIPS bugs shouldn't break the
                    // game. Routing/logging infrastructure is added in a later
                    // milestone.
                }
            }
            self.rules_engine = Some(engine);
        }
    }

    pub(crate) fn resolve_spell(&mut self, spell: SpellOnStack) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // CR 608.2b: If all targets of a spell are illegal at resolution,
        // the spell is countered by game rules ("fizzles"). It goes to the
        // graveyard without any effect.
        if !spell.targets.is_empty() && !self.any_target_legal(&spell.targets) {
            let owner_id = spell.card.owner_id().to_owned();
            if let Ok(owner) = self.player_state_mut(&owner_id) {
                owner.graveyard.push(spell.card);
            }
            return events;
        }

        // Move card to appropriate zone
        if self.is_permanent_type(&spell.card) {
            events.extend(self.enter_battlefield(spell.card.clone(), &spell.controller_id, ZoneName::Stack));
        } else {
            // Non-permanent (instant/sorcery) goes to graveyard
            let owner_id = spell.card.owner_id().to_owned();
            if let Ok(owner) = self.player_state_mut(&owner_id) {
                owner.graveyard.push(spell.card.clone());
            }
        }

        // Emit spell resolved event
        let snapshot = CardInstanceSnapshot {
            instance_id: CardInstanceId::new(spell.card.instance_id()),
            definition_id: CardDefinitionId::new(spell.card.definition().id()),
            owner_id: PlayerId::new(spell.card.owner_id()),
        };
        let event = GameEvent::SpellResolved {
            card: snapshot,
            controller_id: PlayerId::new(&spell.controller_id),
            targets: spell.targets.clone(),
        };
        events.push(event.clone());

        // === Call CLIPS to determine spell effects ===
        // We take the engine out temporarily to avoid a borrow conflict:
        // engine.evaluate(&mut self, ...) and self.apply_rules_action() both
        // need &mut self, which the borrow checker won't allow if the engine is
        // borrowed through self.rules_engine. Taking it out and putting it back
        // is the idiomatic solution.
        if self.rules_engine.is_some() {
            let mut engine = self.rules_engine.take().expect("checked is_some above");
            match engine.evaluate(self, &event) {
                Ok(result) => {
                    // Collect actions first to avoid borrow issues
                    let actions: Vec<RulesAction> = result.actions.clone();
                    for action in &actions {
                        self.apply_rules_action(action);
                    }
                }
                Err(_e) => {
                    // Log error but don't crash — CLIPS bugs shouldn't break the game.
                    // Routing/logging infrastructure is added in a later milestone.
                }
            }
            self.rules_engine = Some(engine);
        }

        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);

        events
    }

    /// Apply a single `RulesAction` proposed by the rules engine to the game state.
    ///
    /// Each variant maps to one mutation on `Game`. Actions that target unknown
    /// entities are silently ignored (rules engine may fire for entities that
    /// were removed before the action was applied).
    pub(crate) fn apply_rules_action(&mut self, action: &RulesAction) {
        match action {
            RulesAction::DealDamage { source: _, target, amount } => {
                // Try as player damage first
                if let Ok(player) = self.player_state_mut(target) {
                    player.life_total -= *amount as i32;
                } else if let Some(pstate) = self.permanent_states.get(target).cloned() {
                    // Creature damage: accumulate damage on the permanent
                    let current_damage = pstate
                        .creature_state()
                        .map(|cs| cs.damage_marked_this_turn())
                        .unwrap_or(0);
                    if let Ok(damaged) = pstate.with_damage(current_damage + *amount as i32) {
                        self.permanent_states.insert(target.clone(), damaged);
                    }
                    // SBA checks lethal damage and destroys the creature
                    self.perform_state_based_actions();
                }
            }
            RulesAction::DrawCards { player, amount } => {
                self.draw_cards_internal(player, *amount);
            }
            RulesAction::DestroyPermanent { target } => {
                // Ignore if the permanent no longer exists
                let _ = self.move_permanent_to_graveyard(target, GraveyardReason::Destroy);
            }
            RulesAction::GainLife { player, amount } => {
                if let Ok(p) = self.player_state_mut(player) {
                    p.life_total += *amount as i32;
                }
            }
            RulesAction::LoseLife { player, amount } => {
                if let Ok(p) = self.player_state_mut(player) {
                    p.life_total -= *amount as i32;
                }
            }
            RulesAction::Tap { permanent_id } => {
                // Ignore error if permanent not found
                let _ = self.tap_permanent(permanent_id);
            }
            RulesAction::Untap { permanent_id } => {
                // Ignore error if permanent not found
                let _ = self.untap_permanent(permanent_id);
            }
            RulesAction::AddMana { player, color, amount } => {
                if let Some(mana_color) = parse_mana_color(color) {
                    let _ = self.add_mana_to_pool(player, mana_color, *amount);
                }
            }
            RulesAction::Exile { target } => {
                // Ignore if the permanent no longer exists.
                let _ = self.move_permanent_to_exile(target);
            }
            RulesAction::ModifyPowerToughness { target, power, toughness, duration, source } => {
                if let Some(state) = self.permanent_states.get(target).cloned() {
                    let effect_duration = parse_effect_duration(duration);
                    let effect = ContinuousEffect {
                        power_modifier: *power,
                        toughness_modifier: *toughness,
                        duration: effect_duration,
                        source_id: source.clone(),
                    };
                    let new_state = state.with_continuous_effect(effect);
                    self.permanent_states.insert(target.clone(), new_state);
                }
            }
            // Stubs for M3: log but don't crash
            RulesAction::MoveZone { .. }
            | RulesAction::AddCounter { .. }
            | RulesAction::CreateToken { .. } => {
                // TODO(M4): implement these actions
            }
        }
    }

    pub(crate) fn resolve_ability(&mut self, _ability: AbilityOnStack) -> Vec<GameEvent> {
        // Find the source permanent (for Last Known Information)
        // The effect was stored when activated, so it can resolve even if the source left.
        // In MVP: effects are simple and operate on the game state directly.
        // TODO: Call ability.effect.resolve() when Effect::resolve() signature is implemented
        Vec::new()
    }

    /// Check if at least one target in the list is still legal.
    ///
    /// Per CR 608.2b, a spell fizzles only if ALL targets are illegal.
    /// A player target is legal if the player exists in the game.
    /// A creature target is legal if the permanent is still on the battlefield.
    fn any_target_legal(&self, targets: &[crate::domain::targets::Target]) -> bool {
        use crate::domain::targets::Target;
        targets.iter().any(|t| match t {
            Target::Player { player_id } => {
                self.players.iter().any(|p| p.player_id.as_str() == player_id)
            }
            Target::Creature { permanent_id } | Target::Permanent { permanent_id } => {
                self.players.iter().any(|p| {
                    p.battlefield.iter().any(|c| c.instance_id() == permanent_id.as_str())
                })
            }
            Target::StackSpell { stack_index } => *stack_index < self.stack.len(),
        })
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Convert a `TriggerEventType` into the short string used in the
/// `GameEvent::TriggeredAbilityFires::trigger_type` field.
///
/// These strings appear in CLIPS `game-event` facts under the `data` slot,
/// allowing CLIPS rules to distinguish trigger categories.
fn trigger_type_string(event_type: &TriggerEventType) -> String {
    match event_type {
        TriggerEventType::ZoneChanged => "ZONE_CHANGED".to_owned(),
        TriggerEventType::StepStarted => "STEP_START".to_owned(),
        TriggerEventType::CreatureDeclaredAttacker => "ATTACK".to_owned(),
        TriggerEventType::CombatEnded => "COMBAT_ENDED".to_owned(),
    }
}

/// Parse a duration string (as produced by CLIPS action facts) into `EffectDuration`.
///
/// Defaults to `UntilEndOfTurn` for unknown strings.
fn parse_effect_duration(duration: &str) -> EffectDuration {
    match duration.to_ascii_lowercase().as_str() {
        "until_end_of_turn" | "until-end-of-turn" | "eot" => EffectDuration::UntilEndOfTurn,
        _ => EffectDuration::UntilEndOfTurn,
    }
}

/// Parse a mana color string (as produced by CLIPS action facts) into `ManaColor`.
///
/// CLIPS represents colors as SYMBOL values like `WHITE`, `BLUE`, etc.
/// Returns `None` for unknown color strings.
fn parse_mana_color(color: &str) -> Option<ManaColor> {
    match color.to_ascii_uppercase().as_str() {
        "WHITE" | "W" => Some(ManaColor::White),
        "BLUE" | "U" => Some(ManaColor::Blue),
        "BLACK" | "B" => Some(ManaColor::Black),
        "RED" | "R" => Some(ManaColor::Red),
        "GREEN" | "G" => Some(ManaColor::Green),
        "COLORLESS" | "C" => Some(ManaColor::Colorless),
        _ => None,
    }
}

// ============================================================================
// Tests for rules engine integration
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::GameEvent;
    use crate::domain::rules_engine::{RulesAction, RulesEngine, RulesError, RulesResult};
    use crate::domain::game::test_helpers::{make_game_in_first_main, add_card_to_hand};
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::CardType;
    use crate::domain::actions::Action;
    use crate::domain::types::{CardInstanceId, PlayerId};

    // =========================================================================
    // MockRulesEngine: returns a pre-configured RulesResult
    // =========================================================================

    struct MockRulesEngine {
        result: Result<RulesResult, RulesError>,
    }

    impl MockRulesEngine {
        fn returning(actions: Vec<RulesAction>) -> Self {
            Self {
                result: Ok(RulesResult {
                    actions,
                    awaiting_input: None,
                    rules_fired: 1,
                    warnings: Vec::new(),
                }),
            }
        }

        fn returning_error() -> Self {
            Self {
                result: Err(RulesError::Internal("mock CLIPS error".to_owned())),
            }
        }
    }

    impl RulesEngine for MockRulesEngine {
        fn evaluate(
            &mut self,
            _state: &Game,
            _event: &GameEvent,
        ) -> Result<RulesResult, RulesError> {
            // Return a clone of the preset result
            match &self.result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(RulesError::Internal(e.to_string())),
            }
        }

        fn resume_after_choice(
            &mut self,
            _choice: &crate::domain::rules_engine::PlayerChoice,
        ) -> Result<RulesResult, RulesError> {
            Err(RulesError::NoInputPending)
        }
    }

    fn make_sorcery(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("shock", "Shock", vec![CardType::Sorcery]);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Cast a sorcery and resolve the stack (both players pass priority).
    ///
    /// Per CR 117.3c, after casting the caster retains priority. So the caster
    /// passes first, then the opponent, for the stack to resolve.
    fn cast_and_resolve_sorcery(
        game: &mut Game,
        player_id: &str,
        opponent_id: &str,
        card_id: &str,
    ) {
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(player_id),
            card_id: CardInstanceId::new(card_id),
            targets: vec![],
        })
        .expect("cast should succeed");

        // Caster (CR 117.3c retains priority) passes first
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(player_id),
        })
        .expect("caster pass priority should succeed");

        // Opponent passes priority — both have passed, spell resolves
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(opponent_id),
        })
        .expect("opponent pass priority should succeed");
    }

    // =========================================================================
    // Tests: set_rules_engine
    // =========================================================================

    #[test]
    fn game_can_have_rules_engine_set() {
        let mut game = Game::create("test-game");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(42)).unwrap();

        let engine = MockRulesEngine::returning(vec![]);
        game.set_rules_engine(Box::new(engine));
        // If it compiles and doesn't panic, the field exists and is writable.
    }

    // =========================================================================
    // Tests: apply_rules_action — DealDamage
    // =========================================================================

    #[test]
    fn rules_action_deal_damage_reduces_player_life() {
        let (mut game, p1, p2) = make_game_in_first_main();

        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::DealDamage {
                source: "shock-1".to_owned(),
                target: p2.clone(),
                amount: 3,
            },
        ])));

        let spell = make_sorcery("shock-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let life_before = game.player_life_total(&p2).unwrap();
        cast_and_resolve_sorcery(&mut game, &p1, &p2, "shock-1");

        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            life_before - 3,
            "DealDamage should reduce target player's life by the amount"
        );
    }

    // =========================================================================
    // Tests: apply_rules_action — DrawCards
    // =========================================================================

    #[test]
    fn rules_action_draw_cards_increases_hand_size() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        // Give p1 a 10-card library
        let deck: Vec<_> = (0..10)
            .map(|i| {
                let def = CardDefinition::new(
                    &format!("card-{i}"),
                    &format!("Card {i}"),
                    vec![CardType::Land],
                );
                CardInstance::new(&format!("card-{i}"), def, "p1")
            })
            .collect();
        game.assign_deck("p1", deck).unwrap();
        game.assign_deck("p2", vec![]).unwrap();
        game.start("p1", Some(42)).unwrap();

        // Advance to FirstMain
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();

        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::DrawCards {
                player: "p1".to_owned(),
                amount: 2,
            },
        ])));

        let spell = make_sorcery("s1", "p1");
        add_card_to_hand(&mut game, "p1", spell);

        let hand_before = game.hand("p1").unwrap().len();
        cast_and_resolve_sorcery(&mut game, "p1", "p2", "s1");

        // Hand should grow by 2 (the drawn cards) minus 1 (the spell that resolved)
        // Actually the sorcery was already removed from hand when cast — the hand
        // size at cast time is hand_before - 1, and then draw adds 2.
        // But hand_before was measured BEFORE casting. So net: +2 from draw, -1 from cast = +1.
        assert_eq!(
            game.hand("p1").unwrap().len(),
            hand_before - 1 + 2,
            "DrawCards should add cards to hand"
        );
    }

    // =========================================================================
    // Tests: apply_rules_action — GainLife
    // =========================================================================

    #[test]
    fn rules_action_gain_life_increases_player_life() {
        let (mut game, p1, p2) = make_game_in_first_main();

        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::GainLife {
                player: p1.clone(),
                amount: 5,
            },
        ])));

        let spell = make_sorcery("healing-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let life_before = game.player_life_total(&p1).unwrap();
        cast_and_resolve_sorcery(&mut game, &p1, &p2, "healing-1");

        assert_eq!(
            game.player_life_total(&p1).unwrap(),
            life_before + 5,
            "GainLife should add to player's life total"
        );
    }

    // =========================================================================
    // Tests: apply_rules_action — LoseLife
    // =========================================================================

    #[test]
    fn rules_action_lose_life_reduces_player_life() {
        let (mut game, p1, p2) = make_game_in_first_main();

        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::LoseLife {
                player: p1.clone(),
                amount: 4,
            },
        ])));

        let spell = make_sorcery("pain-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let life_before = game.player_life_total(&p1).unwrap();
        cast_and_resolve_sorcery(&mut game, &p1, &p2, "pain-1");

        assert_eq!(
            game.player_life_total(&p1).unwrap(),
            life_before - 4,
            "LoseLife should reduce player's life total"
        );
    }

    // =========================================================================
    // Tests: backward compatibility — no rules engine
    // =========================================================================

    #[test]
    fn without_rules_engine_spell_resolves_normally() {
        let (mut game, p1, p2) = make_game_in_first_main();

        // No rules engine set — just resolve normally
        let creature = {
            let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
                .with_power_toughness(2, 2);
            CardInstance::new("bear-1", def, &p1)
        };
        add_card_to_hand(&mut game, &p1, creature);

        cast_and_resolve_sorcery(&mut game, &p1, &p2, "bear-1");

        // Creature should land on battlefield, no crash
        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
        assert!(game.stack().is_empty());
    }

    // =========================================================================
    // Tests: error handling — CLIPS error doesn't crash the game
    // =========================================================================

    #[test]
    fn rules_engine_error_does_not_crash_game() {
        let (mut game, p1, p2) = make_game_in_first_main();

        game.set_rules_engine(Box::new(MockRulesEngine::returning_error()));

        let spell = make_sorcery("shock-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        // Cast and resolve — should NOT panic even with CLIPS error
        cast_and_resolve_sorcery(&mut game, &p1, &p2, "shock-1");

        // The game continues normally (spell went to graveyard)
        assert!(game.stack().is_empty());
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    // =========================================================================
    // Tests: execute_triggered_abilities — CLIPS integration (M4)
    // =========================================================================

    /// Build a creature card that has an ETB trigger ("When ~ enters the
    /// battlefield, draw a card").
    fn make_etb_draw_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::effects::Effect;
        use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::draw_cards(1),
        );
        let def = CardDefinition::new("etb-draw", "ETB Draw Creature", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_trigger(trigger);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Build a creature card that has a death trigger ("When ~ dies, draw a card").
    fn make_death_draw_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::effects::Effect;
        use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::SourceDies,
            Effect::draw_cards(1),
        );
        let def = CardDefinition::new("death-draw", "Death Draw Creature", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_trigger(trigger);
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn etb_trigger_calls_clips_and_applies_draw_cards_action() {
        use crate::domain::game::test_helpers::{make_game_in_first_main, add_card_to_hand};

        let (mut game, p1, p2) = make_game_in_first_main();

        // Give p1 a 5-card library so draw effects work.
        for i in 0..5 {
            let def = CardDefinition::new(
                &format!("lib-card-{i}"),
                &format!("Library Card {i}"),
                vec![CardType::Land],
            );
            let card = CardInstance::new(&format!("lib-card-{i}"), def, &p1);
            game.player_state_mut(&p1).unwrap().library.push(card);
        }

        // MockRulesEngine: on any evaluate(), draw 1 card for p1.
        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::DrawCards {
                player: p1.clone(),
                amount: 1,
            },
        ])));

        let creature = make_etb_draw_creature("etb-1", &p1);
        add_card_to_hand(&mut game, &p1, creature);

        let hand_before = game.hand(&p1).unwrap().len();

        // Cast the creature and resolve it onto the battlefield.
        cast_and_resolve_sorcery(&mut game, &p1, &p2, "etb-1");

        // After the spell resolves and the ETB trigger fires, the CLIPS engine is
        // called twice — once for SpellResolved (draws 1) and once for
        // TriggeredAbilityFires/ETB (draws 1).
        // hand_before includes the creature. After cast: -1. After spell CLIPS: +1.
        // After ETB CLIPS: +1. Net: hand_before + 1.
        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            1,
            "creature should be on the battlefield"
        );

        assert_eq!(
            game.hand(&p1).unwrap().len(),
            hand_before + 1,
            "ETB trigger should have drawn 1 card via CLIPS (on top of spell resolve draw)"
        );
    }

    #[test]
    fn death_trigger_calls_clips_and_applies_draw_cards_action() {
        use crate::domain::enums::GraveyardReason;
        use crate::domain::game::test_helpers::{add_permanent_to_battlefield, make_game_in_first_main};

        let (mut game, p1, _p2) = make_game_in_first_main();

        // Give p1 a 5-card library.
        for i in 0..5 {
            let def = CardDefinition::new(
                &format!("lib-{i}"),
                &format!("Lib {i}"),
                vec![CardType::Land],
            );
            let card = CardInstance::new(&format!("lib-{i}"), def, &p1);
            game.player_state_mut(&p1).unwrap().library.push(card);
        }

        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::DrawCards {
                player: p1.clone(),
                amount: 1,
            },
        ])));

        let creature = make_death_draw_creature("death-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        let hand_before = game.hand(&p1).unwrap().len();

        // Kill the creature — this should fire its death trigger via CLIPS.
        game.move_permanent_to_graveyard("death-1", GraveyardReason::Destroy)
            .expect("creature should be moved to graveyard");

        assert_eq!(
            game.graveyard(&p1).unwrap().len(),
            1,
            "creature should be in graveyard"
        );
        assert_eq!(
            game.hand(&p1).unwrap().len(),
            hand_before + 1,
            "death trigger should have drawn 1 card via CLIPS"
        );
    }

    #[test]
    fn triggered_abilities_are_noop_without_rules_engine() {
        use crate::domain::enums::GraveyardReason;
        use crate::domain::game::test_helpers::{add_permanent_to_battlefield, make_game_in_first_main};

        let (mut game, p1, _p2) = make_game_in_first_main();

        // No rules engine set.
        let creature = make_death_draw_creature("death-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        let hand_before = game.hand(&p1).unwrap().len();

        // Kill the creature — no engine, so trigger should be a no-op.
        game.move_permanent_to_graveyard("death-1", GraveyardReason::Destroy)
            .expect("creature should be moved to graveyard");

        assert_eq!(
            game.hand(&p1).unwrap().len(),
            hand_before,
            "without a rules engine, triggers should be no-ops"
        );
    }

    #[test]
    fn triggered_abilities_engine_error_does_not_crash_game() {
        use crate::domain::enums::GraveyardReason;
        use crate::domain::game::test_helpers::{add_permanent_to_battlefield, make_game_in_first_main};

        let (mut game, p1, _p2) = make_game_in_first_main();

        game.set_rules_engine(Box::new(MockRulesEngine::returning_error()));

        let creature = make_death_draw_creature("death-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        // Kill the creature — CLIPS errors should not crash the game.
        let result = game.move_permanent_to_graveyard("death-1", GraveyardReason::Destroy);
        assert!(result.is_ok(), "CLIPS error during trigger should not crash game");
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    #[test]
    fn multiple_triggers_fire_in_sequence_via_clips() {
        use crate::domain::effects::Effect;
        use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};
        use crate::domain::game::test_helpers::{add_permanent_to_battlefield, make_game_in_first_main};
        use crate::domain::enums::GraveyardReason;

        let (mut game, p1, _p2) = make_game_in_first_main();

        // Give p1 a 10-card library.
        for i in 0..10 {
            let def = CardDefinition::new(
                &format!("lib-{i}"),
                &format!("Lib {i}"),
                vec![CardType::Land],
            );
            let card = CardInstance::new(&format!("lib-{i}"), def, &p1);
            game.player_state_mut(&p1).unwrap().library.push(card);
        }

        // MockRulesEngine: always draws 1 card for p1.
        game.set_rules_engine(Box::new(MockRulesEngine::returning(vec![
            RulesAction::DrawCards {
                player: p1.clone(),
                amount: 1,
            },
        ])));

        // Two creatures with death triggers — when both die, p1 should draw 2 cards.
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::SourceDies,
            Effect::draw_cards(1),
        );
        let def1 = CardDefinition::new("death-draw", "Death Draw", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_trigger(trigger.clone());
        let creature1 = CardInstance::new("death-1", def1, &p1);

        let def2 = CardDefinition::new("death-draw", "Death Draw", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_trigger(trigger);
        let creature2 = CardInstance::new("death-2", def2, &p1);

        add_permanent_to_battlefield(&mut game, &p1, creature1);
        add_permanent_to_battlefield(&mut game, &p1, creature2);

        let hand_before = game.hand(&p1).unwrap().len();

        // Kill both creatures in sequence.
        game.move_permanent_to_graveyard("death-1", GraveyardReason::Destroy)
            .expect("first death should succeed");
        game.move_permanent_to_graveyard("death-2", GraveyardReason::Destroy)
            .expect("second death should succeed");

        assert_eq!(
            game.hand(&p1).unwrap().len(),
            hand_before + 2,
            "each death trigger should draw 1 card via CLIPS"
        );
    }

    #[test]
    fn triggered_ability_fires_event_serializes_in_bridge() {
        use crate::infrastructure::clips::bridge::serialize_game_event;

        let event = GameEvent::TriggeredAbilityFires {
            source: crate::domain::events::CardInstanceSnapshot {
                instance_id: crate::domain::types::CardInstanceId::new("etb-1"),
                definition_id: crate::domain::types::CardDefinitionId::new("etb-draw"),
                owner_id: crate::domain::types::PlayerId::new("p1"),
            },
            controller_id: crate::domain::types::PlayerId::new("p1"),
            trigger_type: "ETB".to_owned(),
        };
        let fact = serialize_game_event(&event);
        assert!(fact.contains("(type TRIGGERED_ABILITY_FIRES)"), "should use TRIGGERED_ABILITY_FIRES type, got: {fact}");
        assert!(fact.contains(r#"(source-id "etb-1")"#), "should have source id");
        assert!(fact.contains(r#"(controller "p1")"#), "should have controller");
    }
}
