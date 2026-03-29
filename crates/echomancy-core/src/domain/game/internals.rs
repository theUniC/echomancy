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
use crate::domain::rules_engine::RulesAction;
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
use crate::domain::triggers::TriggerEventType;
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
        // Find which player controls this permanent and build the snapshot
        // before removing it so we have all card data available for event
        // construction and trigger collection.
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

        // Build the event snapshot while the card is still on the battlefield.
        let (snapshot, owner_id) = {
            let player = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() == controller_id)
                .ok_or_else(|| GameError::PlayerNotFound {
                    player_id: PlayerId::new(&controller_id),
                })?;
            let card = &player.battlefield[card_idx];
            let snap = CardInstanceSnapshot {
                instance_id: CardInstanceId::new(card.instance_id()),
                definition_id: CardDefinitionId::new(card.definition().id()),
                owner_id: PlayerId::new(card.owner_id()),
            };
            (snap, card.owner_id().to_owned())
        };

        // Build the zone-change event now so we can collect triggers BEFORE
        // removing the card from the battlefield. This is necessary because
        // trigger collection (`find_matching_triggers`) iterates battlefield
        // permanents — if we remove first, death-triggered cards won't be seen.
        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone: ZoneName::Battlefield,
            to_zone: ZoneName::Graveyard,
            controller_id: PlayerId::new(&controller_id),
        };

        // Collect triggers while the source permanent is still on the battlefield.
        let triggered = self.collect_triggered_abilities(&event);

        // Now perform the actual zone change.
        let card = {
            let player = self.player_state_mut(&controller_id)?;
            player.battlefield.remove(card_idx)
        };

        // Clean up permanent state.
        self.permanent_states.remove(permanent_id);

        // Add to owner's graveyard.
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.graveyard.push(card);
        }

        // Execute triggers now that the card is in the graveyard (so CLIPS can
        // see the updated battlefield state if needed).
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

    /// Add mana to a player's pool.
    ///
    /// `pub` because cross-crate tests need it for test setup.
    /// In production, mana is only added via `ActivateAbility` (mana abilities).
    pub fn add_mana(
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
                let new_damage = cs.damage_marked_this_turn() + damage;
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
    fn apply_rules_action(&mut self, action: &RulesAction) {
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
                    let _ = self.add_mana(player, mana_color, *amount);
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
        if abilities.is_empty() || self.rules_engine.is_none() {
            return;
        }

        for ability in abilities {
            // Build the `TriggeredAbilityFires` event that CLIPS will match on.
            // This gives CLIPS rules a specific event type distinct from spell
            // resolution so they can target trigger-only effects.
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

        // Opponent passes priority
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(opponent_id),
        })
        .expect("pass priority should succeed");

        // Active player passes priority — spell resolves
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(player_id),
        })
        .expect("pass priority should succeed");
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
