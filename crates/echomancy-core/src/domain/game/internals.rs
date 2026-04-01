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
use crate::domain::enums::{CardType, ManaColor, StaticAbility, Step, ZoneName};
use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::services::combat_resolution::{
    calculate_all_combat_damage, calculate_first_strike_combat_damage, CreatureCombatEntry,
};
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
        let mut events = Vec::new();

        // Clear ALL auto-pass flags at Untap (new turn = fresh start).
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

        // First strike damage resolution at FIRST_STRIKE_DAMAGE step.
        // Only creatures with FirstStrike deal damage in this step.
        if step == Step::FirstStrikeDamage {
            events.extend(self.resolve_first_strike_damage());
            events.extend(self.perform_state_based_actions());
        }

        // Combat damage resolution at COMBAT_DAMAGE step.
        // Only creatures that have NOT already dealt first strike damage deal damage here.
        if step == Step::CombatDamage {
            events.extend(self.resolve_regular_combat_damage());
            events.extend(self.perform_state_based_actions());
        }

        // Clear mana pools and damage at CLEANUP, and expire timed effects.
        // CR 514.1: discard down to maximum hand size (7).
        if step == Step::Cleanup {
            self.clear_all_mana_pools();
            self.clear_damage_on_all_creatures();
            self.expire_continuous_effects(EffectDuration::UntilEndOfTurn);
            self.enforce_hand_size_limit();
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
        use crate::domain::enums::StaticAbility;

        let current_player = self.turn_state.current_player_id().as_str().to_owned();

        let card_instances: Vec<(String, bool, bool)> = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() == current_player)
            .map(|p| {
                p.battlefield
                    .iter()
                    .map(|c| (
                        c.instance_id().to_owned(),
                        c.definition().is_creature(),
                        c.definition().has_static_ability(StaticAbility::DoesNotUntap),
                    ))
                    .collect()
            })
            .unwrap_or_default();

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

    /// Collect all creatures currently in combat (attacking or blocking).
    ///
    /// Returns a list of `(instance_id, controller_id, PermanentState, has_trample, has_deathtouch, has_lifelink)` tuples.
    fn collect_combat_creatures(&self) -> Vec<(String, String, PermanentState, bool, bool, bool, bool)> {
        let mut result = Vec::new();

        for player in &self.players {
            let controller = player.player_id.as_str().to_owned();
            for card in &player.battlefield {
                let id = card.instance_id().to_owned();
                let state = match self.permanent_states.get(&id) {
                    Some(s) => s.clone(),
                    None => continue,
                };
                let cs = match state.creature_state() {
                    Some(cs) => cs,
                    None => continue,
                };
                if !cs.is_attacking() && cs.blocking_creature_id().is_none() {
                    continue;
                }
                let def = card.definition();
                let has_trample = def.has_static_ability(StaticAbility::Trample);
                let has_deathtouch = def.has_static_ability(StaticAbility::Deathtouch);
                let has_lifelink = def.has_static_ability(StaticAbility::Lifelink);
                let has_menace = def.has_static_ability(StaticAbility::Menace);
                result.push((id, controller.clone(), state, has_trample, has_deathtouch, has_lifelink, has_menace));
            }
        }

        result
    }

    /// Returns `true` if the creature with the given instance_id has `FirstStrike`.
    ///
    /// Looks up the creature's card definition on both battlefields.
    fn creature_has_first_strike(&self, instance_id: &str) -> bool {
        self.players.iter().any(|player| {
            player
                .battlefield
                .iter()
                .any(|card| {
                    card.instance_id() == instance_id
                        && (card.definition().has_static_ability(StaticAbility::FirstStrike)
                            || card.definition().has_static_ability(StaticAbility::DoubleStrike))
                })
        })
    }

    fn creature_has_double_strike(&self, instance_id: &str) -> bool {
        self.players.iter().any(|player| {
            player
                .battlefield
                .iter()
                .any(|card| {
                    card.instance_id() == instance_id
                        && card.definition().has_static_ability(StaticAbility::DoubleStrike)
                })
        })
    }

    /// Resolve damage for the `FirstStrikeDamage` step.
    ///
    /// Only creatures with `FirstStrike` deal damage in this step.
    /// Each creature that deals damage gets `dealt_first_strike_damage = true`.
    fn resolve_first_strike_damage(&mut self) -> Vec<GameEvent> {
        let active_player = self.turn_state.current_player_id().as_str().to_owned();
        let defending_player_id = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() != active_player)
            .map(|p| p.player_id.as_str().to_owned())
            .unwrap_or_default();

        let all_combat = self.collect_combat_creatures();

        // Determine which combat creatures have FirstStrike.
        let first_strikers: Vec<(String, String, PermanentState, bool, bool, bool, bool)> = all_combat
            .iter()
            .filter(|(id, _, _, _, _, _, _)| self.creature_has_first_strike(id))
            .cloned()
            .collect();

        // If no first strikers, nothing to do.
        if first_strikers.is_empty() {
            return Vec::new();
        }

        // Collect IDs that have first strike (both attackers and blockers).
        let first_striker_ids: Vec<String> = first_strikers
            .iter()
            .map(|(id, _, _, _, _, _, _)| id.clone())
            .collect();

        // Build snapshots for the full combat pool (needed for blocker lookups) and for
        // first strikers only (the damage sources in this step).
        let all_entries: Vec<CreatureCombatEntry<'_>> = all_combat
            .iter()
            .map(|(id, controller, state, has_trample, has_deathtouch, has_lifelink, has_menace)| CreatureCombatEntry {
                instance_id: id.as_str(),
                controller_id: controller.as_str(),
                state,
                has_trample: *has_trample,
                has_deathtouch: *has_deathtouch,
                has_lifelink: *has_lifelink,
                has_menace: *has_menace,
            })
            .collect();

        let fs_entries: Vec<CreatureCombatEntry<'_>> = first_strikers
            .iter()
            .map(|(id, controller, state, has_trample, has_deathtouch, has_lifelink, has_menace)| CreatureCombatEntry {
                instance_id: id.as_str(),
                controller_id: controller.as_str(),
                state,
                has_trample: *has_trample,
                has_deathtouch: *has_deathtouch,
                has_lifelink: *has_lifelink,
                has_menace: *has_menace,
            })
            .collect();

        let assignments =
            calculate_first_strike_combat_damage(&fs_entries, &all_entries, &defending_player_id);

        // Collect source IDs that actually dealt damage.
        let dealing_ids: Vec<String> = assignments
            .iter()
            .filter(|a| a.amount > 0)
            .map(|a| a.source_id.clone())
            .collect();

        // Apply damage; also handle Lifelink life gain.
        for assignment in &assignments {
            if assignment.is_player {
                self.deal_damage_to_player(&assignment.target_id, assignment.amount);
            } else {
                self.mark_damage_on_creature(
                    &assignment.target_id,
                    assignment.amount,
                    assignment.is_deathtouch,
                );
            }
            // Lifelink: source controller gains life equal to damage dealt.
            if assignment.has_lifelink && assignment.amount > 0 {
                self.gain_life(&assignment.source_controller_id, assignment.amount);
            }
        }

        // Mark dealt_first_strike_damage on all first strikers that dealt damage.
        for id in &first_striker_ids {
            if dealing_ids.contains(id) {
                if let Some(state) = self.permanent_states.get(id).cloned() {
                    if let Ok(new_state) = state.with_dealt_first_strike_damage(true) {
                        self.permanent_states.insert(id.clone(), new_state);
                    }
                }
            }
        }

        Vec::new()
    }

    /// Resolve damage for the regular `CombatDamage` step.
    ///
    /// Only creatures that have NOT already dealt first strike damage participate.
    fn resolve_regular_combat_damage(&mut self) -> Vec<GameEvent> {
        let active_player = self.turn_state.current_player_id().as_str().to_owned();

        let defending_player_id = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() != active_player)
            .map(|p| p.player_id.as_str().to_owned())
            .unwrap_or_default();

        let all_combat = self.collect_combat_creatures();

        // Filter out creatures that already dealt first strike damage,
        // UNLESS they have Double Strike (CR 702.4: deal damage in both steps).
        let regular_combat: Vec<(String, String, PermanentState, bool, bool, bool, bool)> = all_combat
            .into_iter()
            .filter(|(id, _, state, _, _, _, _)| {
                let dealt_fs = state
                    .creature_state()
                    .map(|cs| cs.dealt_first_strike_damage())
                    .unwrap_or(false);
                if !dealt_fs {
                    return true; // Didn't deal first strike damage → participates
                }
                // Dealt first strike damage but has Double Strike → still participates
                self.creature_has_double_strike(id)
            })
            .collect();

        let combat_entries: Vec<CreatureCombatEntry<'_>> = regular_combat
            .iter()
            .map(|(id, controller, state, has_trample, has_deathtouch, has_lifelink, has_menace)| CreatureCombatEntry {
                instance_id: id.as_str(),
                controller_id: controller.as_str(),
                state,
                has_trample: *has_trample,
                has_deathtouch: *has_deathtouch,
                has_lifelink: *has_lifelink,
                has_menace: *has_menace,
            })
            .collect();

        let assignments = calculate_all_combat_damage(&combat_entries, &defending_player_id);

        // Apply all damage; also handle Lifelink life gain.
        for assignment in &assignments {
            if assignment.is_player {
                self.deal_damage_to_player(&assignment.target_id, assignment.amount);
            } else {
                self.mark_damage_on_creature(
                    &assignment.target_id,
                    assignment.amount,
                    assignment.is_deathtouch,
                );
            }
            // Lifelink: source controller gains life equal to damage dealt.
            if assignment.has_lifelink && assignment.amount > 0 {
                self.gain_life(&assignment.source_controller_id, assignment.amount);
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

    /// Remove all continuous effects with the given duration from every permanent.
    ///
    /// Called at Cleanup step to expire "until end of turn" effects.
    pub(crate) fn expire_continuous_effects(&mut self, duration: EffectDuration) {
        let ids: Vec<String> = self.permanent_states.keys().cloned().collect();
        for id in ids {
            if let Some(state) = self.permanent_states.get(&id).cloned() {
                let new_state = state.without_expired_effects(duration.clone());
                self.permanent_states.insert(id, new_state);
            }
        }
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

#[cfg(test)]
mod tests {
    use crate::domain::actions::Action;
    use crate::domain::enums::Step;
    use crate::domain::game::test_helpers::{
        add_permanent_to_battlefield, clear_summoning_sickness, make_creature_card,
        make_started_game,
    };
    use crate::domain::types::{CardInstanceId, PlayerId};

    /// Advance the game to the CombatDamage step with a declared attacker and
    /// an optional blocker. Returns the game plus the two player IDs.
    ///
    /// `blocker_id` is only added to the battlefield and declared as a blocker
    /// when `Some` is passed.
    fn setup_combat_damage(
        attacker_power: u32,
        attacker_toughness: u32,
        blocker: Option<(u32, u32)>, // (power, toughness)
    ) -> (crate::domain::game::Game, String, String) {
        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers (5 steps from Untap)
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        let attacker = make_creature_card("attacker-1", &p1, attacker_power, attacker_toughness);
        add_permanent_to_battlefield(&mut game, &p1, attacker);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance to DeclareBlockers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        if let Some((bp, bt)) = blocker {
            let blocker_card = make_creature_card("blocker-1", &p2, bp, bt);
            add_permanent_to_battlefield(&mut game, &p2, blocker_card);

            game.apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("attacker-1"),
            })
            .unwrap();
        }

        // Advance through FirstStrikeDamage to CombatDamage.
        // (No first strikers in this setup, so FirstStrikeDamage does nothing.)
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        (game, p1, p2)
    }

    #[test]
    fn unblocked_attacker_deals_damage_to_defending_player() {
        let (game, _p1, p2) = setup_combat_damage(3, 3, None);
        // Defending player should have taken 3 damage (20 - 3 = 17)
        assert_eq!(game.player_life_total(&p2).unwrap(), 17);
    }

    #[test]
    fn blocker_deals_damage_back_to_attacker_bidirectional() {
        // Attacker: 3/3  Blocker: 2/5
        // Expected: attacker takes 2 damage, blocker takes 3 damage
        let (game, _p1, p2) = setup_combat_damage(3, 3, Some((2, 5)));

        // Defending player should take NO damage (attacker is blocked)
        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            20,
            "Blocked attacker should not deal damage to defending player"
        );

        // Attacker should have 2 damage marked (from blocker's power)
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let attacker_cs = attacker_state.creature_state().unwrap();
        assert_eq!(
            attacker_cs.damage_marked_this_turn(),
            2,
            "Attacker should receive damage from blocker"
        );

        // Blocker should have 3 damage marked (from attacker's power)
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        let blocker_cs = blocker_state.creature_state().unwrap();
        assert_eq!(
            blocker_cs.damage_marked_this_turn(),
            3,
            "Blocker should receive damage from attacker"
        );
    }

    #[test]
    fn lethal_blocker_kills_attacker_via_bidirectional_damage() {
        // Attacker: 1/1  Blocker: 3/3
        // Attacker takes 3 damage (lethal) → SBA should destroy it
        let (mut game, _p1, _p2) = setup_combat_damage(1, 1, Some((3, 3)));

        game.perform_state_based_actions();

        // Attacker (1 toughness, took 3 damage) should be destroyed
        let attacker_state = game.permanent_state("attacker-1");
        assert!(
            attacker_state.is_none(),
            "Attacker should be destroyed after taking lethal damage from blocker"
        );
    }

    // =========================================================================
    // First Strike tests
    // =========================================================================

    /// Advance the game to the FirstStrikeDamage step with creatures in combat.
    ///
    /// The attacker is optionally given First Strike via a custom card.
    /// The blocker is optionally placed and declared.
    fn setup_first_strike_damage(
        attacker_power: u32,
        attacker_toughness: u32,
        attacker_has_first_strike: bool,
        blocker: Option<(u32, u32, bool)>, // (power, toughness, has_first_strike)
    ) -> (crate::domain::game::Game, String, String) {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::test_helpers::make_creature_with_ability;

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        let attacker = if attacker_has_first_strike {
            make_creature_with_ability(
                "attacker-1",
                &p1,
                attacker_power,
                attacker_toughness,
                StaticAbility::FirstStrike,
            )
        } else {
            make_creature_card("attacker-1", &p1, attacker_power, attacker_toughness)
        };
        add_permanent_to_battlefield(&mut game, &p1, attacker);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance to DeclareBlockers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        if let Some((bp, bt, blocker_fs)) = blocker {
            let blocker_card = if blocker_fs {
                make_creature_with_ability(
                    "blocker-1",
                    &p2,
                    bp,
                    bt,
                    StaticAbility::FirstStrike,
                )
            } else {
                make_creature_card("blocker-1", &p2, bp, bt)
            };
            add_permanent_to_battlefield(&mut game, &p2, blocker_card);

            game.apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("attacker-1"),
            })
            .unwrap();
        }

        // Advance to FirstStrikeDamage
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        (game, p1, p2)
    }

    #[test]
    fn first_strike_attacker_deals_damage_in_first_strike_step() {
        // 2/1 First Strike attacker vs 2/3 regular blocker (survives with 2 damage).
        // After FirstStrikeDamage: blocker should have 2 damage, attacker has 0.
        let (game, _p1, _p2) =
            setup_first_strike_damage(2, 1, true, Some((2, 3, false)));

        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(
            blocker_state.creature_state().unwrap().damage_marked_this_turn(),
            2,
            "Blocker should have 2 damage from first strike attacker"
        );

        let attacker_state = game.permanent_state("attacker-1").unwrap();
        assert_eq!(
            attacker_state.creature_state().unwrap().damage_marked_this_turn(),
            0,
            "Attacker should have no damage in first strike step (blocker has no first strike)"
        );
    }

    #[test]
    fn first_strike_attacker_gets_dealt_first_strike_damage_flag() {
        // After dealing first strike damage, the flag must be set.
        let (game, _p1, _p2) =
            setup_first_strike_damage(2, 1, true, Some((2, 2, false)));

        let attacker_state = game.permanent_state("attacker-1").unwrap();
        assert!(
            attacker_state
                .creature_state()
                .unwrap()
                .dealt_first_strike_damage(),
            "First Strike attacker should have dealt_first_strike_damage = true"
        );
    }

    #[test]
    fn first_strike_attacker_does_not_deal_damage_in_regular_combat_step() {
        // The attacker already dealt its damage in FirstStrikeDamage.
        // In CombatDamage it must not deal damage again.
        let (mut game, p1, _p2) =
            setup_first_strike_damage(2, 1, true, Some((2, 2, false)));

        // SBAs: blocker received 2 damage (lethal at 2 toughness) → destroyed.
        game.perform_state_based_actions();

        // Advance to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        // Attacker should NOT deal more damage (it already dealt first strike damage).
        // The blocker is dead so no one takes damage.
        // The attacker had dealt_first_strike_damage = true so it skips CombatDamage.
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        assert_eq!(
            attacker_state.creature_state().unwrap().damage_marked_this_turn(),
            0,
            "Attacker should have 0 damage marked (blocker had no first strike)"
        );
    }

    #[test]
    fn first_strike_attacker_kills_blocker_before_regular_damage_step() {
        // 2/1 First Strike attacker vs 2/2 regular blocker.
        // Blocker takes 2 lethal damage in first strike step → destroyed.
        // In regular CombatDamage: blocker is gone, so attacker dealt no more damage.
        let (mut game, p1, p2) =
            setup_first_strike_damage(2, 1, true, Some((2, 2, false)));

        // Run SBAs — blocker should be destroyed.
        game.perform_state_based_actions();
        assert!(
            game.permanent_state("blocker-1").is_none(),
            "Blocker should be destroyed by SBAs after first strike damage"
        );

        // Advance to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Attacker had first strike and already dealt damage — no second damage.
        // Defender should still be at 20 life (attacker was blocked, no trample).
        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            20,
            "Defending player should not take damage (attacker was blocked, no trample)"
        );

        // Attacker survived (blocker died before dealing damage).
        assert!(
            game.permanent_state("attacker-1").is_some(),
            "Attacker should survive (blocker died in first strike step)"
        );
    }

    #[test]
    fn regular_attacker_does_not_deal_damage_in_first_strike_step() {
        // Regular attacker vs regular blocker.
        // In FirstStrikeDamage: no damage is dealt.
        let (game, _p1, _p2) =
            setup_first_strike_damage(3, 3, false, Some((2, 2, false)));

        // Neither creature should have any damage after FirstStrikeDamage.
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(
            attacker_state.creature_state().unwrap().damage_marked_this_turn(),
            0,
            "Regular attacker should not deal damage in first strike step"
        );
        assert_eq!(
            blocker_state.creature_state().unwrap().damage_marked_this_turn(),
            0,
            "Regular blocker should not take damage in first strike step"
        );
    }

    #[test]
    fn regular_attacker_deals_damage_in_combat_damage_step() {
        // Regular 3/3 attacker vs regular 2/5 blocker — both deal damage in CombatDamage.
        // Blocker survives (5 toughness > 3 damage), attacker survives (3 toughness > 2 damage).
        let (mut game, p1, _p2) =
            setup_first_strike_damage(3, 3, false, Some((2, 5, false)));

        // Advance to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(
            blocker_state.creature_state().unwrap().damage_marked_this_turn(),
            3,
            "Blocker should have 3 damage from regular attacker in CombatDamage"
        );
        assert_eq!(
            attacker_state.creature_state().unwrap().damage_marked_this_turn(),
            2,
            "Attacker should have 2 damage from regular blocker in CombatDamage"
        );
    }

    #[test]
    fn both_have_first_strike_both_deal_damage_in_first_strike_step() {
        // 2/5 First Strike attacker vs 2/5 First Strike blocker (both survive with 2 damage).
        // Both deal damage simultaneously in FirstStrikeDamage.
        let (game, _p1, _p2) =
            setup_first_strike_damage(2, 5, true, Some((2, 5, true)));

        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(
            blocker_state.creature_state().unwrap().damage_marked_this_turn(),
            2,
            "Blocker should have 2 damage from first strike attacker"
        );
        assert_eq!(
            attacker_state.creature_state().unwrap().damage_marked_this_turn(),
            2,
            "Attacker should have 2 damage from first strike blocker"
        );
    }

    #[test]
    fn both_have_first_strike_no_damage_in_regular_step() {
        // When both creatures have First Strike, both get the flag set.
        // In the regular CombatDamage step neither should deal damage again.
        let (mut game, p1, _p2) =
            setup_first_strike_damage(2, 2, true, Some((2, 2, true)));

        // Run SBAs — both creatures took lethal damage.
        game.perform_state_based_actions();

        // Advance to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        // Both are destroyed; they should not be on the battlefield.
        assert!(
            game.permanent_state("attacker-1").is_none(),
            "First Strike attacker should be destroyed (took 2 damage, 2 toughness)"
        );
        assert!(
            game.permanent_state("blocker-1").is_none(),
            "First Strike blocker should be destroyed (took 2 damage, 2 toughness)"
        );
    }

    #[test]
    fn no_first_strikers_first_strike_step_does_nothing() {
        // No creatures have First Strike → FirstStrikeDamage step does nothing.
        let (game, _p1, _p2) =
            setup_first_strike_damage(3, 3, false, Some((2, 2, false)));

        // Neither creature should have any damage after the first strike step.
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(attacker_state.creature_state().unwrap().damage_marked_this_turn(), 0);
        assert_eq!(blocker_state.creature_state().unwrap().damage_marked_this_turn(), 0);
    }

    #[test]
    fn first_strike_blocker_deals_damage_to_attacker_in_first_strike_step() {
        // Regular attacker vs First Strike blocker.
        // In FirstStrikeDamage: blocker deals damage to attacker.
        let (game, _p1, _p2) =
            setup_first_strike_damage(3, 3, false, Some((2, 2, true)));

        // Attacker should have taken 2 damage from first strike blocker.
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        assert_eq!(
            attacker_state.creature_state().unwrap().damage_marked_this_turn(),
            2,
            "Attacker should receive 2 damage from first strike blocker"
        );

        // Blocker should have taken 0 damage (regular attacker has no first strike).
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(
            blocker_state.creature_state().unwrap().damage_marked_this_turn(),
            0,
            "Blocker should not take damage from regular attacker in first strike step"
        );
    }

    #[test]
    fn step_sequence_declare_blockers_to_first_strike_damage_to_combat_damage() {
        let (mut game, p1, _p2) = make_started_game();

        // Advance to DeclareBlockers
        for _ in 0..6 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        // Next step: FirstStrikeDamage
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        // Next step: CombatDamage
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);
    }

    // =========================================================================
    // Double Strike tests (CR 702.4)
    // =========================================================================

    #[test]
    fn double_strike_unblocked_deals_damage_twice_to_defending_player() {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::test_helpers::make_creature_with_ability;

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&current) }).unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        // 3/3 Double Strike attacker
        let attacker = make_creature_with_ability("ds-attacker", &p1, 3, 3, StaticAbility::DoubleStrike);
        add_permanent_to_battlefield(&mut game, &p1, attacker);
        clear_summoning_sickness(&mut game, "ds-attacker");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("ds-attacker"),
        }).unwrap();

        let life_before = game.player_life_total(&p2).unwrap();

        // Advance through DeclareBlockers → FirstStrikeDamage
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);
        // FirstStrikeDamage resolves — 3 damage to defender
        let life_after_fs = game.player_life_total(&p2).unwrap();
        assert_eq!(life_after_fs, life_before - 3, "First strike damage should deal 3");

        // CombatDamage step — Double Strike deals damage again
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);
        let life_after_cd = game.player_life_total(&p2).unwrap();
        assert_eq!(life_after_cd, life_before - 6,
            "Double Strike should deal damage in BOTH steps (3+3=6)");
    }

    // =========================================================================
    // Trample tests
    // =========================================================================

    /// Set up combat reaching CombatDamage with a trample attacker.
    fn setup_trample_combat(
        attacker_power: u32,
        attacker_toughness: u32,
        blocker: Option<(u32, u32)>,
        attacker_has_deathtouch: bool,
    ) -> (crate::domain::game::Game, String, String) {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::test_helpers::make_creature_with_ability;
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        // Build attacker with Trample (+ optionally Deathtouch).
        let attacker_card = if attacker_has_deathtouch {
            let def = CardDefinition::new("creature", "Creature", vec![CardType::Creature])
                .with_power_toughness(attacker_power, attacker_toughness)
                .with_static_ability(StaticAbility::Trample)
                .with_static_ability(StaticAbility::Deathtouch);
            CardInstance::new("attacker-1", def, &p1)
        } else {
            make_creature_with_ability(
                "attacker-1",
                &p1,
                attacker_power,
                attacker_toughness,
                StaticAbility::Trample,
            )
        };
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance to DeclareBlockers.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        if let Some((bp, bt)) = blocker {
            let blocker_card = make_creature_card("blocker-1", &p2, bp, bt);
            add_permanent_to_battlefield(&mut game, &p2, blocker_card);

            game.apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("attacker-1"),
            })
            .unwrap();
        }

        // Advance through FirstStrikeDamage to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        (game, p1, p2)
    }

    #[test]
    fn trample_6_6_blocked_by_1_1_deals_1_to_blocker_5_to_player() {
        // 6/6 Trample blocked by 1/1: 1 lethal to blocker (killed), 5 to player
        let (game, _p1, p2) = setup_trample_combat(6, 6, Some((1, 1)), false);

        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            15,
            "Defending player should take 5 trample damage"
        );

        // The 1/1 blocker receives lethal damage (1 >= 1 toughness) and is destroyed by SBAs
        // which run automatically as part of the CombatDamage step.
        assert!(
            game.permanent_state("blocker-1").is_none(),
            "1/1 blocker should be destroyed after receiving 1 lethal damage"
        );
    }

    #[test]
    fn trample_3_3_blocked_by_3_3_no_damage_to_player() {
        // 3/3 Trample blocked by 3/3: all 3 goes to blocker, 0 to player
        let (game, _p1, p2) = setup_trample_combat(3, 3, Some((3, 3)), false);

        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            20,
            "Defending player should take 0 damage when all damage is needed to kill blocker"
        );
    }

    #[test]
    fn trample_deathtouch_6_6_blocked_by_5_5_deals_1_to_blocker_5_to_player() {
        // 6/6 Trample+Deathtouch blocked by 1/5: 1 lethal (deathtouch), 5 trample through
        // The 1/5 blocker is killed by SBAs (has_deathtouch_damage + damage > 0).
        let (game, _p1, p2) = setup_trample_combat(6, 6, Some((1, 5)), true);

        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            15,
            "Defending player should take 5 trample damage (deathtouch means 1 is lethal)"
        );

        // Blocker is destroyed by SBAs (deathtouch damage is lethal regardless of toughness).
        assert!(
            game.permanent_state("blocker-1").is_none(),
            "1/5 blocker should be destroyed by deathtouch damage (SBA)"
        );
    }

    // =========================================================================
    // Deathtouch tests (via full game flow)
    // =========================================================================

    /// Set up combat reaching CombatDamage with a deathtouch attacker.
    fn setup_deathtouch_combat(
        attacker_power: u32,
        attacker_toughness: u32,
        blocker_power: u32,
        blocker_toughness: u32,
    ) -> (crate::domain::game::Game, String, String) {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::test_helpers::make_creature_with_ability;

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        let attacker_card = make_creature_with_ability(
            "attacker-1",
            &p1,
            attacker_power,
            attacker_toughness,
            StaticAbility::Deathtouch,
        );
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        let blocker_card = make_creature_card("blocker-1", &p2, blocker_power, blocker_toughness);
        add_permanent_to_battlefield(&mut game, &p2, blocker_card);

        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance through FirstStrikeDamage to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        (game, p1, p2)
    }

    #[test]
    fn deathtouch_1_1_vs_5_5_blocker_is_destroyed_by_sba() {
        // 1/1 Deathtouch vs 5/5: blocker gets 1 deathtouch damage → SBA kills it
        let (mut game, _p1, _p2) = setup_deathtouch_combat(1, 1, 5, 5);

        game.perform_state_based_actions();

        assert!(
            game.permanent_state("blocker-1").is_none(),
            "5/5 blocker should be destroyed by deathtouch damage"
        );
    }

    #[test]
    fn deathtouch_1_1_vs_5_5_blocker_moves_to_graveyard() {
        // 1/1 Deathtouch vs 5/5: the deathtouch flag causes the 5/5 to be destroyed by SBA.
        // SBAs fire automatically as part of the CombatDamage step, so by the time we check,
        // the blocker is already in the graveyard.
        let (game, _p1, p2) = setup_deathtouch_combat(1, 1, 5, 5);

        // Blocker should be in graveyard (destroyed by deathtouch SBA)
        assert!(
            game.permanent_state("blocker-1").is_none(),
            "5/5 blocker should be destroyed by deathtouch (SBA)"
        );
        assert_eq!(
            game.graveyard(&p2).unwrap().len(),
            1,
            "Blocker should be in p2's graveyard"
        );
    }

    // =========================================================================
    // Lifelink tests (via full game flow)
    // =========================================================================

    /// Set up combat reaching CombatDamage with a lifelink attacker.
    fn setup_lifelink_combat(
        attacker_power: u32,
        attacker_toughness: u32,
        blocker: Option<(u32, u32)>,
    ) -> (crate::domain::game::Game, String, String) {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::test_helpers::make_creature_with_ability;

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        let attacker_card = make_creature_with_ability(
            "attacker-1",
            &p1,
            attacker_power,
            attacker_toughness,
            StaticAbility::Lifelink,
        );
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        if let Some((bp, bt)) = blocker {
            let blocker_card = make_creature_card("blocker-1", &p2, bp, bt);
            add_permanent_to_battlefield(&mut game, &p2, blocker_card);

            game.apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("attacker-1"),
            })
            .unwrap();
        }

        // Advance through FirstStrikeDamage to CombatDamage.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        (game, p1, p2)
    }

    #[test]
    fn lifelink_3_3_unblocked_controller_gains_3_life() {
        // 3/3 Lifelink unblocked: deals 3 to player, controller gains 3 life
        let (game, p1, p2) = setup_lifelink_combat(3, 3, None);

        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            17,
            "Defending player should take 3 damage"
        );
        assert_eq!(
            game.player_life_total(&p1).unwrap(),
            23,
            "Attacking player should gain 3 life from Lifelink"
        );
    }

    #[test]
    fn lifelink_3_3_blocked_by_2_2_controller_gains_3_life_from_creature_damage() {
        // 3/3 Lifelink blocked by 2/2: damage goes to creature, but controller still gains life
        let (game, p1, p2) = setup_lifelink_combat(3, 3, Some((2, 2)));

        // Defender takes no player damage (attacker is blocked)
        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            20,
            "Defending player should not take direct damage when attacker is blocked"
        );

        // Attacker controller gains 3 life (from 3 damage dealt to the blocker)
        assert_eq!(
            game.player_life_total(&p1).unwrap(),
            23,
            "Attacking player should gain 3 life from Lifelink (damage to creature still counts)"
        );
    }

    #[test]
    fn first_strike_lifelink_gains_life_in_first_strike_step() {
        // First Strike + Lifelink: life is gained during the first strike damage step
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::{CardType, StaticAbility};

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        // 3/3 with both FirstStrike and Lifelink
        let def = CardDefinition::new("creature", "Creature", vec![CardType::Creature])
            .with_power_toughness(3, 3)
            .with_static_ability(StaticAbility::FirstStrike)
            .with_static_ability(StaticAbility::Lifelink);
        let attacker_card = CardInstance::new("attacker-1", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Add 3/5 blocker (survives first strike damage)
        let blocker_card = make_creature_card("blocker-1", &p2, 3, 5);
        add_permanent_to_battlefield(&mut game, &p2, blocker_card);
        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance to FirstStrikeDamage — first strike deals 3 damage AND gains 3 life
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        // After first strike step: p1 should have gained 3 life
        assert_eq!(
            game.player_life_total(&p1).unwrap(),
            23,
            "First Strike + Lifelink: life should be gained in the first strike step"
        );
    }

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
}
