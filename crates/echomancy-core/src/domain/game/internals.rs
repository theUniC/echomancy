//! Internal mutation helpers for the `Game` aggregate.
//!
//! All methods in this file take `&mut self` and directly mutate game state.
//! They are called by command handlers and by other internal helpers. They are
//! NOT part of the public API — all are `pub(crate)` or `fn`.

use std::collections::HashSet;

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::entities::the_stack::{AbilityOnStack, SpellOnStack, StackItem};
use crate::domain::enums::{CardType, GraveyardReason, ManaColor, Step, ZoneName};
use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::services::combat_resolution::{calculate_damage_assignments, CreatureCombatEntry};
use crate::domain::services::mana_payment::pay_cost;
use crate::domain::services::state_based_actions::{
    CreatureSbaEntry, PlayerSbaEntry, find_creatures_to_destroy,
    find_players_who_attempted_empty_library_draw, find_players_with_zero_or_less_life,
};
use crate::domain::services::step_machine::advance;
use crate::domain::services::trigger_evaluation::{
    find_matching_triggers, PermanentOnBattlefield, TriggeredAbilityInfo,
};
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;

use super::{Game, GameEndReason, GameOutcome};

impl Game {
    // =========================================================================
    // Internal mutations (called by handlers)
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

    /// Enter a permanent onto a player's battlefield and initialize its state.
    ///
    /// This is the single entry point for ALL permanents entering the battlefield.
    /// After moving the card:
    /// 1. Initializes permanent state (creature or non-creature).
    /// 2. Evaluates ETB triggers.
    pub(crate) fn enter_battlefield(
        &mut self,
        permanent: CardInstance,
        controller_id: &str,
        from_zone: ZoneName,
    ) -> Vec<GameEvent> {
        let snapshot = CardInstanceSnapshot {
            instance_id: CardInstanceId::new(permanent.instance_id()),
            definition_id: CardDefinitionId::new(permanent.definition().id()),
            owner_id: PlayerId::new(permanent.owner_id()),
        };

        // Initialize permanent state
        if permanent.definition().is_creature() {
            let power = permanent.definition().power().unwrap_or(0) as i32;
            let toughness = permanent.definition().toughness().unwrap_or(0) as i32;
            self.permanent_states.insert(
                permanent.instance_id().to_owned(),
                PermanentState::for_creature(power, toughness),
            );
        } else if self.is_permanent_type(&permanent) {
            self.permanent_states.insert(
                permanent.instance_id().to_owned(),
                PermanentState::for_non_creature(),
            );
        }

        // Add to battlefield
        if let Ok(player) = self.player_state_mut(controller_id) {
            player.battlefield.push(permanent);
        }

        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone,
            to_zone: ZoneName::Battlefield,
            controller_id: PlayerId::new(controller_id),
        };
        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);
        vec![event]
    }

    /// Move a permanent from any battlefield to its owner's graveyard.
    ///
    /// Cleans up permanent state and evaluates dies/LTB triggers.
    pub(crate) fn move_permanent_to_graveyard(
        &mut self,
        permanent_id: &str,
        _reason: GraveyardReason,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Find which player controls this permanent
        let (controller_id, card_idx) = {
            let mut found = None;
            for player in &self.players {
                if let Some(idx) = player
                    .battlefield
                    .iter()
                    .position(|c| c.instance_id() == permanent_id)
                {
                    found = Some((player.player_id.as_str().to_owned(), idx));
                    break;
                }
            }
            found.ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            })?
        };

        // Remove from battlefield
        let card = {
            let player = self.player_state_mut(&controller_id)?;
            player.battlefield.remove(card_idx)
        };

        // Clean up permanent state
        self.permanent_states.remove(permanent_id);

        // Add to owner's graveyard
        let owner_id = card.owner_id().to_owned();
        let snapshot = CardInstanceSnapshot {
            instance_id: CardInstanceId::new(card.instance_id()),
            definition_id: CardDefinitionId::new(card.definition().id()),
            owner_id: PlayerId::new(&owner_id),
        };
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.graveyard.push(card);
        }

        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone: ZoneName::Battlefield,
            to_zone: ZoneName::Graveyard,
            controller_id: PlayerId::new(&controller_id),
        };
        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);
        Ok(vec![event])
    }

    /// Assign priority to a player, triggering auto-pass logic if applicable.
    pub(crate) fn assign_priority_to(&mut self, player_id: &str) -> Vec<GameEvent> {
        self.priority_player_id = Some(PlayerId::new(player_id));

        // Auto-pass: if player is in auto-pass mode
        if self.auto_pass_players.contains(player_id) {
            if self.stack_has_items() {
                // Auto-pass priority when stack is non-empty
                return self.perform_internal_pass(player_id);
            } else if player_id == self.turn_state.current_player_id().as_str() {
                // Auto-advance steps when stack is empty and they're the active player
                return self.process_auto_pass();
            }
        }
        Vec::new()
    }

    /// Give priority to the opponent of the given player.
    pub(crate) fn give_priority_to_opponent_of(
        &mut self,
        player_id: &str,
    ) -> Vec<GameEvent> {
        self.players_who_passed_priority.clear();
        if let Ok(opponent_id) = self.opponent_of(player_id).map(str::to_owned) {
            self.assign_priority_to(&opponent_id)
        } else {
            Vec::new()
        }
    }

    /// Record that a player has passed priority.
    pub(crate) fn record_passed_priority(&mut self, player_id: &str) {
        self.players_who_passed_priority.insert(player_id.to_owned());
    }

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

    /// Add an item to the top of the stack.
    pub(crate) fn push_stack(&mut self, item: StackItem) {
        self.stack.push(item);
    }

    /// Add mana to a player's pool.
    pub(crate) fn add_mana(
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

    /// Tap a permanent.
    pub(crate) fn tap_permanent(&mut self, permanent_id: &str) -> Result<(), GameError> {
        let state = self.permanent_states.get(permanent_id).ok_or_else(|| {
            GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            }
        })?;
        let new_state = state.with_tapped(true);
        self.permanent_states.insert(permanent_id.to_owned(), new_state);
        Ok(())
    }

    /// Untap a permanent.
    #[allow(dead_code)]
    pub(crate) fn untap_permanent(&mut self, permanent_id: &str) -> Result<(), GameError> {
        let state = self.permanent_states.get(permanent_id).ok_or_else(|| {
            GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            }
        })?;
        let new_state = state.with_tapped(false);
        self.permanent_states.insert(permanent_id.to_owned(), new_state);
        Ok(())
    }

    /// Update the `PermanentState` for a permanent.
    pub(crate) fn set_permanent_state(&mut self, permanent_id: &str, state: PermanentState) {
        self.permanent_states.insert(permanent_id.to_owned(), state);
    }

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
                let new_damage = cs.damage_marked_this_turn + damage;
                if let Ok(new_state) = state.with_damage(new_damage) {
                    self.permanent_states.insert(creature_id.to_owned(), new_state);
                }
            }
        }
    }

    /// Perform state-based actions (SBA).
    ///
    /// Destroys creatures with lethal damage or zero toughness, and ends the
    /// game if a player has lost their win condition.
    pub(crate) fn perform_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // 1. Destroy creatures with lethal damage or zero toughness
        let creature_entries: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .map(|(id, s)| (id.clone(), s.clone()))
            .collect();

        let sba_entries: Vec<CreatureSbaEntry<'_>> = creature_entries
            .iter()
            .map(|(id, s)| CreatureSbaEntry {
                instance_id: id.as_str(),
                state: s,
            })
            .collect();

        let to_destroy = find_creatures_to_destroy(&sba_entries);
        for creature_id in to_destroy {
            if let Ok(evts) =
                self.move_permanent_to_graveyard(&creature_id, GraveyardReason::StateBased)
            {
                events.extend(evts);
            }
        }

        // 2. Check player loss conditions
        let player_entries: Vec<(String, i32, bool)> = self
            .players
            .iter()
            .map(|p| {
                let attempted =
                    self.players_who_attempted_empty_library_draw.contains(p.player_id.as_str());
                (p.player_id.as_str().to_owned(), p.life_total, attempted)
            })
            .collect();

        let sba_player_entries: Vec<PlayerSbaEntry<'_>> = player_entries
            .iter()
            .map(|(id, life, attempted)| PlayerSbaEntry {
                player_id: id.as_str(),
                life_total: *life,
                attempted_empty_library_draw: *attempted,
            })
            .collect();

        let losers_by_life = find_players_with_zero_or_less_life(&sba_player_entries);
        let losers_by_library =
            find_players_who_attempted_empty_library_draw(&sba_player_entries);

        // Clear empty library draw flags
        for pid in &losers_by_library {
            self.players_who_attempted_empty_library_draw.remove(pid.as_str());
        }

        let all_losers: HashSet<&str> = losers_by_life
            .iter()
            .map(String::as_str)
            .chain(losers_by_library.iter().map(String::as_str))
            .collect();

        if !all_losers.is_empty() {
            let reason = if !losers_by_life.is_empty() && !losers_by_library.is_empty() {
                GameEndReason::SimultaneousLoss
            } else if !losers_by_life.is_empty() {
                GameEndReason::LifeTotal
            } else {
                GameEndReason::EmptyLibrary
            };

            if all_losers.len() >= self.players.len() {
                // All players lost simultaneously — draw
                self.outcome = Some(GameOutcome::Draw {
                    reason: GameEndReason::SimultaneousLoss,
                });
                self.lifecycle = crate::domain::enums::GameLifecycleState::Finished;
            } else {
                // The remaining player wins
                let winner_id = self
                    .players
                    .iter()
                    .find(|p| !all_losers.contains(p.player_id.as_str()))
                    .map(|p| p.player_id.clone());
                if let Some(winner_id) = winner_id {
                    self.outcome = Some(GameOutcome::Win {
                        winner_id,
                        reason,
                    });
                    self.lifecycle = crate::domain::enums::GameLifecycleState::Finished;
                }
            }
        }

        events
    }

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
    // Private helpers
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

    fn resolve_spell(&mut self, spell: SpellOnStack) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Execute effect if present
        // Note: In the Rust port, effects are handled through the Effect enum.
        // For MVP, we just resolve the placement.
        // TODO: Execute spell.card.definition().effect when Effect::resolve() is added

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
        };
        events.push(event.clone());
        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);

        events
    }

    fn resolve_ability(&mut self, _ability: AbilityOnStack) -> Vec<GameEvent> {
        // Find the source permanent (for Last Known Information)
        // The effect was stored when activated, so it can resolve even if the source left.
        // In MVP: effects are simple and operate on the game state directly.
        // TODO: Call ability.effect.resolve() when Effect::resolve() signature is implemented
        Vec::new()
    }

    fn perform_internal_pass(&mut self, player_id: &str) -> Vec<GameEvent> {
        self.players_who_passed_priority.insert(player_id.to_owned());

        if self.both_players_have_passed() {
            self.resolve_top_of_stack()
        } else {
            let opponent_id = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() != player_id)
                .map(|p| p.player_id.as_str().to_owned())
                .unwrap_or_default();
            self.assign_priority_to(&opponent_id)
        }
    }

    pub(crate) fn process_auto_pass(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let max_iterations = 100;
        let mut iterations = 0;

        while iterations < max_iterations {
            iterations += 1;

            let current_player = self.turn_state.current_player_id().as_str().to_owned();

            // Stop if active player is not in auto-pass
            if !self.auto_pass_players.contains(&current_player) {
                break;
            }

            // Stop if there's something on the stack
            if self.stack_has_items() {
                break;
            }

            // Advance through cleanup to next turn
            if self.turn_state.current_step() == Step::Cleanup {
                events.extend(self.perform_step_advance());
                break; // Turn has ended
            }

            events.extend(self.perform_step_advance());
        }

        events
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
                    .map(|cs| cs.is_attacking)
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

    pub(crate) fn collect_triggered_abilities(&self, event: &GameEvent) -> Vec<TriggeredAbilityInfo> {
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
        for _ability in abilities {
            // TODO: Call ability.effect.resolve(self, context) when Effect trait is implemented
            // In MVP, triggered abilities execute immediately (not placed on stack)
        }
    }
}
