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
use crate::domain::services::combat_resolution::{calculate_damage_assignments, CreatureCombatEntry};
use crate::domain::services::mana_payment::pay_cost;
use crate::domain::services::step_machine::advance;
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;

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

    /// Pay the mana cost for a spell.
    ///
    /// Uses the auto-pay algorithm from `ManaPaymentService`.
    ///
    /// # Errors
    ///
    /// Returns `GameError::InsufficientManaForSpell` if the cost cannot be paid.
    pub(crate) fn pay_mana_cost_for_spell(
        &mut self,
        player_id: &str,
        card: &CardInstance,
    ) -> Result<(), GameError> {
        let mana_cost = match card.definition().mana_cost() {
            Some(cost) => cost.clone(),
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

    /// Mark damage on a creature (accumulates in `damage_marked_this_turn`).
    pub(crate) fn mark_damage_on_creature(&mut self, creature_id: &str, damage: i32) {
        if let Some(state) = self.permanent_states.get(creature_id) {
            if let Some(cs) = state.creature_state() {
                let new_damage = cs.damage_marked_this_turn() + damage;
                if let Ok(new_state) = state.with_damage(new_damage) {
                    self.permanent_states.insert(creature_id.to_owned(), new_state);
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

        if self.turn_state.is_main_phase() {
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
        let mut events = Vec::new();

        // Clear auto-pass intent and untap at the start of a new turn
        if step == Step::Untap {
            self.auto_pass_players.clear();
            self.auto_untap_for_current_player();
        }

        // Automatic draw during DRAW step (MTG 504.1 — not using stack).
        // MTG Rule 103.7a: only the STARTING player skips their draw on the very
        // first Draw step of the game. All other players (including P2 on turn 1)
        // draw normally.
        if step == Step::Draw {
            let is_starting_player_first_turn = self.turn_state.turn_number() == 1
                && self.turn_state.current_player_id().as_str() == self.starting_player_id;

            if !is_starting_player_first_turn {
                let current_player = self.turn_state.current_player_id().as_str().to_owned();
                let draw_events = self.draw_cards_internal(&current_player, 1);
                events.extend(draw_events);
                events.extend(self.perform_state_based_actions());
            }
        }

        // Combat damage resolution at COMBAT_DAMAGE step
        if step == Step::CombatDamage {
            events.extend(self.resolve_combat_damage());
            events.extend(self.perform_state_based_actions());
        }

        // Clear mana pools and damage at CLEANUP
        if step == Step::Cleanup {
            self.clear_all_mana_pools();
            self.clear_damage_on_all_creatures();
        }

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

    fn auto_untap_for_current_player(&mut self) {
        let current_player = self.turn_state.current_player_id().as_str().to_owned();

        let card_instances: Vec<(String, bool)> = self
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

        for (instance_id, is_creature) in card_instances {
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

    fn resolve_combat_damage(&mut self) -> Vec<GameEvent> {
        // Collect attacker entries
        let active_player = self.turn_state.current_player_id().as_str().to_owned();

        // We need to snapshot attackers before mutating
        let attacker_entries: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| {
                s.creature_state()
                    .map(|cs| cs.is_attacking())
                    .unwrap_or(false)
            })
            .map(|(id, s)| (id.clone(), s.clone()))
            .collect();

        let combat_entries: Vec<CreatureCombatEntry<'_>> = attacker_entries
            .iter()
            .map(|(id, s)| CreatureCombatEntry {
                instance_id: id.as_str(),
                state: s,
            })
            .collect();

        let defending_player_id = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() != active_player)
            .map(|p| p.player_id.as_str().to_owned())
            .unwrap_or_default();

        let assignments = calculate_damage_assignments(&combat_entries, &defending_player_id);

        // Apply all damage
        for assignment in assignments {
            if assignment.is_player {
                self.deal_damage_to_player(&assignment.target_id, assignment.amount);
            } else {
                self.mark_damage_on_creature(&assignment.target_id, assignment.amount);
            }
        }

        Vec::new()
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
}
