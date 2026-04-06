//! Internal mutation helpers for the `Game` aggregate.
//!
//! All methods in this file take `&mut self` and directly mutate game state.
//! They are called by command handlers and by other internal helpers. They are
//! NOT part of the public API — all are `pub(crate)` or `fn`.
//!
//! This file contains the core internal helpers: draw, mana, damage, step
//! advancement, and combat. Responsibility-specific helpers live in sibling
//! modules:
//!
//! - `stack_resolution` — resolving spells/abilities, CLIPS integration
//! - `zone_transitions` — entering battlefield, moving to graveyard, tap/untap
//! - `sba` — state-based actions
//! - `priority` — priority assignment and passing

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::entities::the_stack::StackItem;
use crate::domain::enums::{CardType, ManaColor, Step, ZoneName};
use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::services::mana_payment::pay_cost;
use crate::domain::services::step_machine::advance;
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::{EffectDuration, PermanentState};

use super::Game;

impl Game {
    // =========================================================================
    // Draw
    // =========================================================================

    /// Draw `amount` cards from a player's library to their hand.
    ///
    /// If the library is empty, the "attempted draw from empty library" flag is
    /// set. State-based actions will check this flag.
    ///
    /// Returns the events produced (one `ZoneChanged` per card drawn).
    pub(crate) fn draw_cards_internal(
        &mut self,
        player_id: &str,
        amount: u32,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        for _ in 0..amount {
            // We need the library length first, then draw
            let library_empty = self
                .player_state(player_id)
                .map(|p| p.library.is_empty())
                .unwrap_or(true);

            if library_empty {
                self.players_who_attempted_empty_library_draw
                    .insert(player_id.to_owned());
                continue;
            }

            // Draw the top card
            let (card_snapshot, player_id_owned) =
                match self.player_state_mut(player_id) {
                    Ok(player) => {
                        let card = player.library.remove(0);
                        let snapshot = CardInstanceSnapshot {
                            instance_id: CardInstanceId::new(card.instance_id()),
                            definition_id: CardDefinitionId::new(card.definition().id()),
                            owner_id: PlayerId::new(card.owner_id()),
                        };
                        player.hand.push(card);
                        (snapshot, player.player_id.as_str().to_owned())
                    }
                    Err(_) => continue,
                };

            let event = GameEvent::ZoneChanged {
                card: card_snapshot,
                from_zone: ZoneName::Library,
                to_zone: ZoneName::Hand,
                controller_id: PlayerId::new(&player_id_owned),
            };
            events.push(event.clone());
            // Evaluate triggers for this draw event
            let triggered = self.collect_triggered_abilities(&event);
            self.execute_triggered_abilities(triggered);
        }
        events
    }

    // =========================================================================
    // Stack
    // =========================================================================

    /// Add an item to the top of the stack.
    pub(crate) fn push_stack(&mut self, item: StackItem) {
        self.stack.push(item);
    }

    // =========================================================================
    // Turn bookkeeping
    // =========================================================================

    /// Record that the current player has played their land this turn.
    pub(crate) fn record_land_played(&mut self) {
        self.turn_state = self.turn_state.with_land_played();
    }

    /// Mark a player as having expressed intent to auto-pass through the end of their turn.
    pub(crate) fn set_auto_pass(&mut self, player_id: &str) {
        self.auto_pass_players.insert(player_id.to_owned());
    }

    /// Clear the set of players who have passed priority in the current window.
    pub(crate) fn clear_passed_priority(&mut self) {
        self.players_who_passed_priority.clear();
    }

    // =========================================================================
    // Mana
    // =========================================================================

    /// Add mana to a player's pool.
    ///
    /// Internal helper. In production, mana is only added via `ActivateAbility`
    /// (mana abilities). Public callers should use `Game::add_mana` instead.
    pub(crate) fn add_mana_to_pool(
        &mut self,
        player_id: &str,
        color: ManaColor,
        amount: u32,
    ) -> Result<(), GameError> {
        if amount == 0 {
            return Err(GameError::InvalidManaAmount { amount: 0 });
        }
        let player = self.player_state_mut(player_id)?;
        player.mana_pool = player
            .mana_pool
            .add(color, amount)
            .map_err(|_| GameError::InsufficientMana {
                player_id: PlayerId::new(player_id),
                color: color.to_string(),
                requested: amount,
                available: 0,
            })?;
        Ok(())
    }

    /// Clear a player's mana pool.
    #[allow(dead_code)]
    pub(crate) fn clear_mana_pool(&mut self, player_id: &str) -> Result<(), GameError> {
        let player = self.player_state_mut(player_id)?;
        player.mana_pool = ManaPool::empty();
        Ok(())
    }

    /// Clear all players' mana pools (called at CLEANUP step).
    pub(crate) fn clear_all_mana_pools(&mut self) {
        for player in &mut self.players {
            player.mana_pool = ManaPool::empty();
        }
    }

    /// Pay the mana cost for a spell, resolving X to the chosen value (CR 107.3).
    ///
    /// If the spell has X in its cost, the effective cost is computed by replacing
    /// each X with `x_value` generic mana before payment.
    ///
    /// # Errors
    ///
    /// Returns `GameError::InsufficientManaForSpell` if the cost cannot be paid.
    pub(crate) fn pay_mana_cost_for_spell_with_x(
        &mut self,
        player_id: &str,
        card: &CardInstance,
        x_value: u32,
    ) -> Result<(), GameError> {
        let mana_cost = match card.definition().mana_cost() {
            Some(cost) => {
                // Resolve X to a concrete generic cost for payment.
                if cost.has_x() {
                    cost.with_x_value(x_value)
                } else {
                    cost.clone()
                }
            }
            None => return Ok(()), // Free spell
        };

        let player = self.player_state_mut(player_id)?;
        let new_pool = pay_cost(player.mana_pool.clone(), &mana_cost)
            .map_err(|e| GameError::InsufficientManaForSpell {
                message: e.to_string(),
            })?;
        player.mana_pool = new_pool;
        Ok(())
    }

    // =========================================================================
    // Damage
    // =========================================================================

    /// Deal damage to a player by reducing their life total.
    pub(crate) fn deal_damage_to_player(&mut self, player_id: &str, damage: i32) {
        if let Ok(player) = self.player_state_mut(player_id) {
            player.life_total -= damage;
        }
    }

    /// Add life to a player's total (used for Lifelink and other life-gain effects).
    pub(crate) fn gain_life(&mut self, player_id: &str, amount: i32) {
        if let Ok(player) = self.player_state_mut(player_id) {
            player.life_total += amount;
        }
    }

    /// Mark damage on a creature (accumulates in `damage_marked_this_turn`).
    ///
    /// If `is_deathtouch` is `true`, also sets `has_deathtouch_damage = true`
    /// on the creature's state (CR 702.2 — any non-zero deathtouch damage is lethal).
    pub(crate) fn mark_damage_on_creature(
        &mut self,
        creature_id: &str,
        damage: i32,
        is_deathtouch: bool,
    ) {
        if let Some(state) = self.permanent_states.get(creature_id) {
            if let Some(cs) = state.creature_state() {
                let new_damage = cs.damage_marked_this_turn() + damage;
                if let Ok(new_state) = state.with_damage(new_damage) {
                    let final_state = if is_deathtouch && damage > 0 {
                        new_state.with_deathtouch_damage()
                    } else {
                        new_state
                    };
                    self.permanent_states.insert(creature_id.to_owned(), final_state);
                }
            }
        }
    }

    // =========================================================================
    // Step advancement
    // =========================================================================

    /// Advance to the next step/phase of the current turn.
    pub(crate) fn perform_step_advance(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // CR 106.4: Mana pools empty at the end of each step and phase.
        // Clear before advancing so mana from one step cannot be used in the next.
        self.clear_all_mana_pools();

        // Emit combat ended event and clear isAttacking when leaving END_OF_COMBAT
        if self.turn_state.current_step() == Step::EndOfCombat {
            let active_player = self.turn_state.current_player_id().clone();
            let event = GameEvent::CombatEnded {
                active_player_id: active_player,
            };
            events.push(event.clone());
            let triggered = self.collect_triggered_abilities(&event);
            self.execute_triggered_abilities(triggered);
            self.clear_attacking_state();
        }

        // Consume scheduled phases first
        if !self.scheduled_steps.is_empty() {
            let next_step = self.scheduled_steps.remove(0);
            events.extend(self.set_current_step(next_step));
            return events;
        }

        // Jump to resume step after scheduled steps
        if let Some(resume_step) = self.resume_step_after_scheduled.take() {
            events.extend(self.set_current_step(resume_step));
            return events;
        }

        // Normal flow
        let step_result = advance(self.turn_state.current_step());
        if step_result.should_advance_player {
            self.advance_to_next_player();
        }
        events.extend(self.set_current_step(step_result.next_step));

        // CR 117.3a: active player receives priority at the beginning of all
        // interactive steps (every step except Untap and Cleanup).
        if !super::automation::is_non_interactive_step(self.turn_state.current_step()) {
            self.players_who_passed_priority.clear();
            let current_player = self.turn_state.current_player_id().as_str().to_owned();
            events.extend(self.assign_priority_to(&current_player));
        }

        events
    }

    // =========================================================================
    // Type helpers
    // =========================================================================

    pub(crate) fn is_permanent_type(&self, card: &CardInstance) -> bool {
        let permanent_types = [
            CardType::Creature,
            CardType::Artifact,
            CardType::Enchantment,
            CardType::Planeswalker,
            CardType::Land,
        ];
        card.definition()
            .types()
            .iter()
            .any(|t| permanent_types.contains(t))
    }

    // =========================================================================
    // Private helpers
    // =========================================================================

    fn set_current_step(&mut self, next_step: Step) -> Vec<GameEvent> {
        self.turn_state = self.turn_state.with_step(next_step);
        self.on_enter_step(next_step)
    }

    fn on_enter_step(&mut self, step: Step) -> Vec<GameEvent> {
        let mut events = match step {
            Step::Untap => self.on_enter_untap_step(),
            Step::Draw => self.on_enter_draw_step(),
            Step::FirstStrikeDamage => self.on_enter_first_strike_damage(),
            Step::CombatDamage => self.on_enter_combat_damage(),
            Step::Cleanup => self.on_enter_cleanup_step(),
            _ => Vec::new(),
        };

        // Emit step started event and evaluate triggers
        let active_player = self.turn_state.current_player_id().clone();
        let step_event = GameEvent::StepStarted {
            step,
            active_player_id: active_player,
        };
        events.push(step_event.clone());
        let triggered = self.collect_triggered_abilities(&step_event);
        self.execute_triggered_abilities(triggered);

        events
    }

    /// CR 502.2: Untap step — clear auto-pass flags and untap all permanents.
    fn on_enter_untap_step(&mut self) -> Vec<GameEvent> {
        self.auto_pass_players.clear();
        self.auto_untap_for_current_player();
        Vec::new()
    }

    /// CR 504.1: Draw step — active player draws one card (except starting player on turn 1).
    fn on_enter_draw_step(&mut self) -> Vec<GameEvent> {
        // MTG Rule 103.7a: only the STARTING player skips their draw on the very
        // first Draw step of the game. All other players (including P2 on turn 1)
        // draw normally.
        let is_starting_player_first_turn = self.turn_state.turn_number() == 1
            && self.turn_state.current_player_id().as_str() == self.starting_player_id;

        if is_starting_player_first_turn {
            return Vec::new();
        }

        let current_player = self.turn_state.current_player_id().as_str().to_owned();
        let mut events = self.draw_cards_internal(&current_player, 1);
        events.extend(self.perform_state_based_actions());
        events
    }

    /// CR 510.1: First strike damage step — only creatures with FirstStrike deal damage.
    fn on_enter_first_strike_damage(&mut self) -> Vec<GameEvent> {
        self.resolve_first_strike_damage();
        self.perform_state_based_actions()
    }

    /// CR 510.2: Combat damage step — creatures that have not already dealt first strike damage.
    fn on_enter_combat_damage(&mut self) -> Vec<GameEvent> {
        self.resolve_regular_combat_damage();
        self.perform_state_based_actions()
    }

    /// CR 514.1–514.3: Cleanup step — clear mana, damage, expire effects, enforce hand size.
    fn on_enter_cleanup_step(&mut self) -> Vec<GameEvent> {
        self.clear_all_mana_pools();
        self.clear_damage_on_all_creatures();
        self.expire_continuous_effects(EffectDuration::UntilEndOfTurn);
        self.enforce_hand_size_limit();
        Vec::new()
    }

    fn auto_untap_for_current_player(&mut self) {
        use crate::domain::enums::StaticAbility;

        let current_player = self.turn_state.current_player_id().as_str().to_owned();

        // Collect instance IDs and creature flags first (immutable borrow).
        let card_base_info: Vec<(String, bool)> = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() == current_player)
            .map(|p| {
                p.battlefield
                    .iter()
                    .map(|c| (c.instance_id().to_owned(), c.definition().is_creature()))
                    .collect()
            })
            .unwrap_or_default();

        // Now consult the layer pipeline for each card (requires &self, separate step).
        // CR 613.1f: Layer 6 effects can remove DoesNotUntap from a permanent.
        let card_instances: Vec<(String, bool, bool)> = card_base_info
            .into_iter()
            .map(|(instance_id, is_creature)| {
                let does_not_untap = self
                    .effective_abilities(&instance_id)
                    .map(|a| a.contains(&StaticAbility::DoesNotUntap))
                    .unwrap_or(false);
                (instance_id, is_creature, does_not_untap)
            })
            .collect();

        for (instance_id, is_creature, does_not_untap) in card_instances {
            // CR 302.6: permanents with "does not untap" skip the untap step.
            if does_not_untap {
                // Still clear summoning sickness for creatures.
                if is_creature {
                    if let Some(state) = self.permanent_states.get(&instance_id) {
                        let new_state = state
                            .with_summoning_sickness(false)
                            .unwrap_or_else(|_| state.clone());
                        self.permanent_states.insert(instance_id, new_state);
                    }
                }
                continue;
            }

            if let Some(state) = self.permanent_states.get(&instance_id) {
                let new_state = if is_creature {
                    // Creatures: untap and clear summoning sickness
                    let untapped = state.with_tapped(false);
                    untapped
                        .with_summoning_sickness(false)
                        .unwrap_or_else(|_| untapped.with_tapped(false))
                } else {
                    state.with_tapped(false)
                };
                self.permanent_states.insert(instance_id, new_state);
            }
        }
    }

    fn advance_to_next_player(&mut self) {
        let current_player = self.turn_state.current_player_id().as_str().to_owned();
        let current_index = self
            .players
            .iter()
            .position(|p| p.player_id.as_str() == current_player)
            .unwrap_or_else(|| {
                debug_assert!(
                    false,
                    "Current player '{current_player}' not found in player list — invariant violated"
                );
                0
            });

        let next_index = (current_index + 1) % self.players.len();
        let next_player_id = self.players[next_index].player_id.clone();

        self.turn_state = self.turn_state.for_new_turn(next_player_id);

        // Increment turn number when wrapping around to first player
        if next_index == 0 {
            self.turn_state = self.turn_state.with_incremented_turn_number();
        }

        // Reset creature states when turn changes
        self.reset_creature_states_for_new_turn();
    }

    fn clear_attacking_state(&mut self) {
        let ids_to_update: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .filter_map(|(id, s)| {
                s.clear_combat_state()
                    .ok()
                    .map(|new_state| (id.clone(), new_state))
            })
            .collect();

        for (id, new_state) in ids_to_update {
            self.permanent_states.insert(id, new_state);
        }
    }

    fn reset_creature_states_for_new_turn(&mut self) {
        let ids_to_update: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .filter_map(|(id, s)| {
                s.clear_combat_state()
                    .ok()
                    .and_then(|cleared| cleared.with_has_attacked_this_turn(false).ok())
                    .map(|new_state| (id.clone(), new_state))
            })
            .collect();

        for (id, new_state) in ids_to_update {
            self.permanent_states.insert(id, new_state);
        }
    }

    fn clear_damage_on_all_creatures(&mut self) {
        let ids_to_update: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .filter_map(|(id, s)| {
                s.clear_damage().ok().map(|new_state| (id.clone(), new_state))
            })
            .collect();

        for (id, new_state) in ids_to_update {
            self.permanent_states.insert(id, new_state);
        }
    }

    /// Remove all global continuous effects with the given duration (LS1).
    ///
    /// Called at Cleanup step to expire "until end of turn" effects.
    pub(crate) fn expire_continuous_effects(&mut self, duration: EffectDuration) {
        self.global_continuous_effects.retain(|e| e.duration != duration);
    }

    /// CR 514.1: At Cleanup, each player with more than 7 cards in hand
    /// discards down to 7. For MVP, the last cards added are discarded
    /// automatically (player doesn't choose).
    fn enforce_hand_size_limit(&mut self) {
        const MAX_HAND_SIZE: usize = 7;
        for player in &mut self.players {
            while player.hand.len() > MAX_HAND_SIZE {
                if let Some(card) = player.hand.pop() {
                    player.graveyard.push(card);
                }
            }
        }
    }
}

// ============================================================================
// Tests for resolve_combat_damage (bidirectional, source fields)
// ============================================================================


// ============================================================================
// Tests for auto_untap_for_current_player
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::domain::actions::Action;
    use crate::domain::enums::Step;
    use crate::domain::game::test_helpers::{
        add_permanent_to_battlefield, clear_summoning_sickness, make_creature_card,
        make_started_game,
    };
    use crate::domain::types::PlayerId;

    #[test]
    fn does_not_untap_creature_stays_tapped_on_untap_step() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::{CardType, StaticAbility};
        use crate::domain::game::test_helpers::make_game_in_first_main;

        let (mut game, p1, p2) = make_game_in_first_main();

        // Add cards to library so draw step doesn't kill the game
        for i in 0..5 {
            let filler = CardInstance::new(
                format!("filler-p1-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            game.add_card_to_library_top(&p1, filler).unwrap();
            let filler2 = CardInstance::new(
                format!("filler-p2-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p2,
            );
            game.add_card_to_library_top(&p2, filler2).unwrap();
        }

        // Add a creature with DoesNotUntap, tap it
        let def = CardDefinition::new("frozen", "Frozen Creature", vec![CardType::Creature])
            .with_power_toughness(3, 3)
            .with_static_ability(StaticAbility::DoesNotUntap);
        let card = CardInstance::new("frozen-1", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);
        game.tap_permanent("frozen-1").unwrap();

        // Also add a normal creature, tapped
        let normal = make_creature_card("normal-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, normal);
        game.tap_permanent("normal-1").unwrap();

        // End P1's turn, then P2's turn → back to P1's Untap
        game.apply(Action::EndTurn {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        game.apply(Action::EndTurn {
            player_id: PlayerId::new(&p2),
        })
        .unwrap();

        // DoesNotUntap creature stays tapped
        assert!(
            game.permanent_state("frozen-1").unwrap().is_tapped(),
            "DoesNotUntap creature should remain tapped"
        );
        // Normal creature is untapped
        assert!(
            !game.permanent_state("normal-1").unwrap().is_tapped(),
            "normal creature should be untapped"
        );
    }

    #[test]
    fn does_not_untap_removed_by_layer_system_allows_untapping() {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::game::test_helpers::make_creature_with_ability;
        use crate::domain::value_objects::permanent_state::EffectDuration;

        // Start in FirstMain so we have time to set up before untap.
        // Add filler land cards to both libraries so draw steps don't end the game.
        let (mut game, p1, p2) = {
            let (mut g, p1, p2) = make_started_game();
            // Add 20 filler lands so each player can draw without losing.
            for i in 0..20 {
                let land = {
                    use crate::domain::cards::card_definition::CardDefinition;
                    use crate::domain::cards::card_instance::CardInstance;
                    use crate::domain::enums::CardType;
                    let def = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
                    CardInstance::new(&format!("land-p1-{i}"), def, &p1)
                };
                if let Ok(player) = g.player_state_mut(&p1) {
                    player.library.push(land);
                }
            }
            for i in 0..20 {
                let land = {
                    use crate::domain::cards::card_definition::CardDefinition;
                    use crate::domain::cards::card_instance::CardInstance;
                    use crate::domain::enums::CardType;
                    let def = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
                    CardInstance::new(&format!("land-p2-{i}"), def, &p2)
                };
                if let Ok(player) = g.player_state_mut(&p2) {
                    player.library.push(land);
                }
            }
            // Advance to FirstMain (Untap→Upkeep→Draw→FirstMain = 3 advances).
            for _ in 0..3 {
                let cur = g.current_player_id().to_owned();
                g.apply(Action::AdvanceStep { player_id: PlayerId::new(&cur) }).unwrap();
            }
            (g, p1, p2)
        };
        assert_eq!(game.current_step(), Step::FirstMain);

        // Add a creature with DoesNotUntap to p1's battlefield.
        let creature = make_creature_with_ability("frozen-1", &p1, 2, 2, StaticAbility::DoesNotUntap);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, "frozen-1");

        // Tap the creature manually (simulates it having attacked).
        if let Some(state) = game.permanent_states.get("frozen-1").cloned() {
            game.permanent_states.insert("frozen-1".to_owned(), state.with_tapped(true));
        }
        assert!(
            game.permanent_state("frozen-1").unwrap().is_tapped(),
            "Setup: creature should be tapped"
        );

        // Remove all abilities via the layer system (simulates "Turn to Frog" removing DoesNotUntap).
        // Use WhileSourceOnBattlefield so the effect persists across turns (it expires only when
        // the source permanent leaves the battlefield).
        let remove_abilities = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::RemoveAllAbilities,
            duration: EffectDuration::WhileSourceOnBattlefield("frog-enchantment".to_owned()),
            timestamp: 200,
            source_id: "frog-enchantment".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["frozen-1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(remove_abilities);

        // Verify the layer system no longer reports DoesNotUntap.
        let abilities = game.effective_abilities("frozen-1").expect("should find creature");
        assert!(
            !abilities.contains(&StaticAbility::DoesNotUntap),
            "effective_abilities must not include DoesNotUntap after RemoveAllAbilities"
        );

        // End p1's turn — this advances to p2's FirstMain.
        game.apply(Action::EndTurn {
            player_id: crate::domain::types::PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_player_id(), &p2);

        // End p2's turn — this advances to p1's next Untap step, which should untap the creature.
        game.apply(Action::EndTurn {
            player_id: crate::domain::types::PlayerId::new(&p2),
        })
        .unwrap();
        assert_eq!(game.current_player_id(), &p1);

        // The creature should now be untapped because DoesNotUntap was removed by the layer system.
        assert!(
            !game.permanent_state("frozen-1").unwrap().is_tapped(),
            "Creature with DoesNotUntap removed by layer system should untap normally"
        );
    }
}
