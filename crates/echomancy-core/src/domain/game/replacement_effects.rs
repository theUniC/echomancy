//! Replacement Effects Framework (CR 614) for the `Game` aggregate.
//!
//! Replacement effects intercept game events BEFORE they occur and modify them.
//! They do not use the stack and cannot be responded to.
//!
//! # Architecture
//!
//! Three event categories are supported (MVP):
//! - **Damage** — intercept before damage is marked on a creature or subtracted from a player.
//! - **Destroy** — intercept before a permanent is moved to the graveyard via a destroy event.
//! - **ETB** — intercept before a permanent enters the battlefield to modify how it enters.
//!
//! Each replacement effect carries a filter (what it watches for), an outcome
//! (what happens instead), a duration, and a timestamp.

use std::collections::HashSet;

// ============================================================================
// Duration for replacement effects
// ============================================================================

/// How long a replacement effect lasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplacementDuration {
    /// Effect is removed during the Cleanup step of the current turn.
    UntilEndOfTurn,
    /// Effect is removed when its source permanent leaves the battlefield.
    WhileSourceOnBattlefield,
    /// Effect applies once and is then removed (e.g. a single regeneration shield).
    NextOccurrence,
    /// Effect tracks a remaining damage budget; decremented on each use.
    /// Removed when `remaining` reaches zero.
    UntilDepleted { remaining: i32 },
}

// ============================================================================
// Event Filters — what events a replacement effect watches for
// ============================================================================

/// What kind of event a replacement effect intercepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplacementEventFilter {
    /// Intercept damage events targeting a specific permanent.
    DamageToPermanent { permanent_id: String },
    /// Intercept damage events targeting a specific player.
    DamageToPlayer { player_id: String },
    /// Intercept all damage events (global effect, both combat and non-combat).
    DamageToAny,
    /// Intercept all combat damage events only (e.g. Fog, CR 615.7a).
    ///
    /// Matches when `is_combat == true`. Does NOT match spell/ability damage
    /// (`is_combat == false`).
    AllCombatDamage,
    /// Intercept destroy events targeting a specific permanent.
    DestroyPermanent { permanent_id: String },
    /// Intercept ETB events for a specific permanent.
    EntersBattlefield { permanent_id: String },
}

// ============================================================================
// Replacement Outcomes — what happens instead
// ============================================================================

/// What happens as a replacement for the intercepted event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplacementOutcome {
    /// Prevent up to `amount` damage. If `amount == 0`, prevents all damage.
    PreventDamage { amount: i32 },
    /// Regenerate: tap, remove all damage, remove from combat. Shield is consumed.
    Regenerate,
    /// Exile the permanent instead of destroying it.
    ExileInstead,
    /// The permanent enters the battlefield tapped.
    EnterTapped,
    /// The permanent enters the battlefield with N counters of the given type.
    EnterWithCounters { counter_type: String, amount: u32 },
}

// ============================================================================
// ReplacementEffect — a registered replacement effect instance
// ============================================================================

/// A single registered replacement effect in the game's replacement registry.
///
/// Per CR 614.6, each effect tracks which event instance IDs it has already
/// been applied to, preventing it from applying twice to the same event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplacementEffect {
    /// Unique identifier for this replacement effect instance.
    pub(crate) effect_id: String,
    /// ID of the source permanent or spell that created this effect.
    pub(crate) source_id: String,
    /// Player who controls the source.
    pub(crate) controller_id: String,
    /// Which event category and target this effect watches for.
    pub(crate) event_filter: ReplacementEventFilter,
    /// What happens instead of the original event.
    pub(crate) replacement: ReplacementOutcome,
    /// When this effect expires.
    pub(crate) duration: ReplacementDuration,
    /// Monotonically increasing timestamp from the game-level counter.
    pub(crate) timestamp: u64,
    /// Event instance IDs this effect has already been applied to (CR 614.6).
    pub(crate) applied_to: HashSet<u64>,
}

impl ReplacementEffect {
    /// Create a new replacement effect.
    pub(crate) fn new(
        effect_id: impl Into<String>,
        source_id: impl Into<String>,
        controller_id: impl Into<String>,
        event_filter: ReplacementEventFilter,
        replacement: ReplacementOutcome,
        duration: ReplacementDuration,
        timestamp: u64,
    ) -> Self {
        Self {
            effect_id: effect_id.into(),
            source_id: source_id.into(),
            controller_id: controller_id.into(),
            event_filter,
            replacement,
            duration,
            timestamp,
            applied_to: HashSet::new(),
        }
    }
}

// ============================================================================
// DestroyReason — why a permanent is being destroyed
// ============================================================================

/// Why a permanent is being moved to the graveyard via a destroy event.
///
/// Only `LethalDamage` and `DestroyEffect` can be intercepted by replacement
/// effects (regeneration shields, etc.). `ZeroToughness` bypasses the
/// replacement framework entirely per CR 704.5f.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DestroyReason {
    /// The creature has damage >= toughness (lethal damage SBA, CR 704.5g).
    /// Interceptable by regeneration and other destroy-replacement effects.
    LethalDamage,
    /// An explicit "destroy" effect (spell or ability, e.g. "Destroy target creature").
    /// Interceptable by regeneration and other destroy-replacement effects.
    DestroyEffect,
    /// The creature has 0 or less toughness (CR 704.5f).
    /// NOT interceptable — this is "put into graveyard", not "destroy".
    ZeroToughness,
    /// The permanent was sacrificed (not destroy, not interceptable).
    Sacrifice,
}

impl DestroyReason {
    /// Returns `true` if replacement effects can intercept this destroy event.
    pub(crate) fn is_interceptable(self) -> bool {
        matches!(self, DestroyReason::LethalDamage | DestroyReason::DestroyEffect)
    }
}


// ============================================================================
// Game impl — replacement effect registry management
// ============================================================================

use super::Game;
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::enums::GraveyardReason;
use crate::domain::types::CardInstanceId;

impl Game {
    // =========================================================================
    // Registry Management
    // =========================================================================

    /// Register a regeneration shield on a target permanent (CR 701.15).
    ///
    /// The shield is a `NextOccurrence` replacement effect: when the target would
    /// next be destroyed (lethal damage or destroy effect), the shield fires and
    /// instead taps the creature, removes all damage, and removes it from combat.
    ///
    /// `source_id` — the permanent that generated the shield (e.g. the creature
    /// whose activated ability was used). `target_id` — the permanent to protect
    /// (for self-regenerate these are the same).
    pub(crate) fn register_regeneration_shield_for(&mut self, source_id: &str, target_id: &str) {
        // Only register a shield if the target is actually on the battlefield.
        if !self.permanent_states.contains_key(target_id) {
            return;
        }
        let controller_id = match self
            .players
            .iter()
            .find(|p| p.battlefield.iter().any(|c| c.instance_id() == target_id))
            .map(|p| p.player_id.as_str().to_owned())
        {
            Some(id) => id,
            None => return,
        };
        let ts = self.next_timestamp();
        let effect = ReplacementEffect::new(
            format!("regen-{ts}"),
            source_id,
            controller_id,
            ReplacementEventFilter::DestroyPermanent { permanent_id: target_id.to_owned() },
            ReplacementOutcome::Regenerate,
            ReplacementDuration::NextOccurrence,
            ts,
        );
        self.register_replacement_effect(effect);
    }

    /// Register a replacement effect in the game's replacement registry.
    pub(crate) fn register_replacement_effect(&mut self, effect: ReplacementEffect) {
        self.replacement_effects.push(effect);
    }

    /// Remove all replacement effects whose source is the given permanent ID.
    ///
    /// Removes both `WhileSourceOnBattlefield` effects (permanent's own static
    /// abilities) and `NextOccurrence` effects (single-use shields like
    /// regeneration) that were generated by this permanent. Without this,
    /// `NextOccurrence` shields on a permanent that has left the battlefield
    /// would linger as dangling entries in the registry.
    ///
    /// Called whenever a permanent leaves the battlefield.
    pub(crate) fn remove_replacement_effects_for_source(&mut self, permanent_id: &str) {
        self.replacement_effects.retain(|e| e.source_id != permanent_id);
    }

    /// Remove all `UntilEndOfTurn` replacement effects.
    ///
    /// Called during the Cleanup step.
    pub(crate) fn cleanup_expired_replacement_effects(&mut self) {
        self.replacement_effects
            .retain(|e| !matches!(e.duration, ReplacementDuration::UntilEndOfTurn));
    }

    /// Allocate a new unique event instance ID for the apply-once rule (CR 614.6).
    fn next_event_id(&mut self) -> u64 {
        let id = self.next_event_instance_id;
        self.next_event_instance_id += 1;
        id
    }

    // =========================================================================
    // Damage Interception (CR 614 — Category A)
    // =========================================================================

    /// Apply damage through the replacement framework.
    ///
    /// Checks for applicable replacement effects and applies them in timestamp
    /// order (oldest first). Returns the final damage amount after all
    /// replacements have been applied.
    ///
    /// This does NOT mark the damage — the caller is responsible for that.
    ///
    /// Parameters:
    /// - `source_id` — ID of the damage source.
    /// - `target_id` — ID of the target (creature or player).
    /// - `amount` — raw damage amount.
    /// - `is_deathtouch` — whether the source has Deathtouch.
    /// - `target_is_player` — true if target is a player, false if a creature.
    /// - `is_combat` — true if damage originates from the Combat Damage step
    ///   (FirstStrikeDamage or CombatDamage), false for spell/ability damage.
    ///   Used by `AllCombatDamage` filter (Fog, CR 615.7a).
    pub(crate) fn apply_damage_with_replacement(
        &mut self,
        _source_id: &str,
        target_id: &str,
        amount: i32,
        _is_deathtouch: bool,
        target_is_player: bool,
        is_combat: bool,
    ) -> i32 {
        let event_id = self.next_event_id();
        let mut remaining = amount;

        // Collect matching effect indices sorted by timestamp (oldest first).
        // We re-collect each iteration after modifications to the registry.
        loop {
            if remaining <= 0 {
                break;
            }

            // Find the oldest applicable damage-prevention effect that hasn't
            // been applied to this event yet.
            let found = self.replacement_effects
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    // Not already applied to this event.
                    if e.applied_to.contains(&event_id) {
                        return false;
                    }
                    // Check filter matches.
                    match &e.event_filter {
                        ReplacementEventFilter::DamageToPermanent { permanent_id } => {
                            !target_is_player && permanent_id == target_id
                        }
                        ReplacementEventFilter::DamageToPlayer { player_id } => {
                            target_is_player && player_id == target_id
                        }
                        ReplacementEventFilter::DamageToAny => true,
                        ReplacementEventFilter::AllCombatDamage => is_combat,
                        ReplacementEventFilter::DestroyPermanent { .. } => false,
                        ReplacementEventFilter::EntersBattlefield { .. } => false,
                    }
                })
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(idx, _)| idx);

            let Some(idx) = found else {
                break; // No more applicable effects
            };

            // Mark this event as applied to this effect.
            self.replacement_effects[idx].applied_to.insert(event_id);

            // Apply the replacement outcome.
            let prevented;
            match self.replacement_effects[idx].replacement.clone() {
                ReplacementOutcome::PreventDamage { amount: shield_amount } => {
                    if shield_amount == 0 {
                        // Prevent all damage.
                        prevented = remaining;
                        remaining = 0;
                    } else {
                        prevented = remaining.min(shield_amount);
                        remaining -= prevented;
                    }
                }
                ReplacementOutcome::Regenerate
                | ReplacementOutcome::ExileInstead
                | ReplacementOutcome::EnterTapped
                | ReplacementOutcome::EnterWithCounters { .. } => {
                    // Non-damage outcome — skip (shouldn't match damage events).
                    break;
                }
            }

            // Update duration: decrement UntilDepleted, consume NextOccurrence.
            let expired = match &mut self.replacement_effects[idx].duration {
                ReplacementDuration::UntilDepleted { remaining: ref mut r } => {
                    *r -= prevented;
                    *r <= 0
                }
                ReplacementDuration::NextOccurrence => true,
                ReplacementDuration::UntilEndOfTurn => false,
                ReplacementDuration::WhileSourceOnBattlefield => false,
            };

            if expired {
                self.replacement_effects.remove(idx);
            }
        }

        remaining
    }

    // =========================================================================
    // Destroy Interception (CR 614 — Category B)
    // =========================================================================

    /// Move a permanent to the graveyard, checking for destroy replacement effects.
    ///
    /// This is the primary entry point for all destroy events. The `reason`
    /// parameter determines whether replacement effects can intercept:
    /// - `LethalDamage` and `DestroyEffect` → interceptable (regeneration, exile instead, etc.)
    /// - `ZeroToughness` and `Sacrifice` → NOT interceptable (bypass the framework)
    ///
    /// Returns `Ok(Vec<GameEvent>)` on success. If a regeneration shield fires,
    /// the creature remains on the battlefield and no zone-change event is emitted.
    pub(crate) fn move_permanent_to_graveyard_with_reason(
        &mut self,
        permanent_id: &str,
        reason: DestroyReason,
    ) -> Result<Vec<GameEvent>, GameError> {
        if !reason.is_interceptable() {
            // Bypass the replacement framework entirely.
            let legacy_reason = match reason {
                DestroyReason::Sacrifice => GraveyardReason::Sacrifice,
                _ => GraveyardReason::StateBased,
            };
            return self.move_permanent_to_graveyard(permanent_id, legacy_reason);
        }

        // Check for applicable destroy replacement effects.
        let event_id = self.next_event_id();

        // Find the oldest applicable destroy-replacement effect.
        // We loop to handle multiple replacement effects applying in sequence.
        loop {
            let found = self.replacement_effects
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    if e.applied_to.contains(&event_id) {
                        return false;
                    }
                    match &e.event_filter {
                        ReplacementEventFilter::DestroyPermanent { permanent_id: pid } => {
                            pid == permanent_id
                        }
                        _ => false,
                    }
                })
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(idx, _)| idx);

            let Some(idx) = found else {
                break; // No more applicable effects — proceed with original event.
            };

            // Mark this event as applied.
            self.replacement_effects[idx].applied_to.insert(event_id);

            let outcome = self.replacement_effects[idx].replacement.clone();

            // Determine if this effect expires.
            let expired = matches!(self.replacement_effects[idx].duration, ReplacementDuration::NextOccurrence);
            if expired {
                self.replacement_effects.remove(idx);
            }

            // Apply the replacement outcome.
            match outcome {
                ReplacementOutcome::Regenerate => {
                    return self.apply_regeneration(permanent_id);
                }
                ReplacementOutcome::ExileInstead => {
                    return self.move_permanent_to_exile(permanent_id);
                }
                _ => {
                    // Not a destroy-applicable outcome — skip and continue loop.
                    continue;
                }
            }
        }

        // No replacement applied — proceed with the original destroy event.
        let legacy_reason = match reason {
            DestroyReason::LethalDamage => GraveyardReason::StateBased,
            DestroyReason::DestroyEffect => GraveyardReason::Destroy,
            _ => GraveyardReason::StateBased,
        };
        self.move_permanent_to_graveyard(permanent_id, legacy_reason)
    }

    /// Apply the regeneration replacement effect to a creature.
    ///
    /// Per CR 701.15:
    /// 1. Remove all damage from the creature.
    /// 2. Tap the creature.
    /// 3. Remove from combat (clear attacking/blocking state).
    /// 4. The creature stays on the battlefield.
    fn apply_regeneration(&mut self, creature_id: &str) -> Result<Vec<GameEvent>, GameError> {
        // Verify the permanent exists.
        if !self.permanent_states.contains_key(creature_id) {
            return Err(GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(creature_id),
            });
        }

        // Remove all damage, tap, and remove from combat in a single clone cycle.
        if let Some(state) = self.permanent_states.get(creature_id).cloned() {
            let mut updated = state;
            // (a) Remove all damage + (b) tap.
            if let Ok(cleared) = updated.with_damage(0) {
                updated = cleared.with_tapped(true);
            }
            // (c) Remove from combat: clear attacking and blocking state.
            if let Ok(no_attack) = updated.with_attacking(false) {
                updated = no_attack.with_blocking_creature_id(None).unwrap_or(no_attack);
            }
            self.permanent_states.insert(creature_id.to_owned(), updated);
        }

        // The creature stays on the battlefield — emit no zone-change event.
        Ok(vec![])
    }

    // =========================================================================
    // ETB Interception (CR 614 — Category C)
    // =========================================================================

    /// Check and apply ETB replacement effects for a permanent entering the battlefield.
    ///
    /// Called from `enter_battlefield` after the permanent state is initialized.
    /// Modifies the permanent state according to any applicable ETB replacement effects.
    pub(crate) fn apply_etb_replacements(&mut self, permanent_id: &str) {
        let event_id = self.next_event_id();

        loop {
            let found = self.replacement_effects
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    if e.applied_to.contains(&event_id) {
                        return false;
                    }
                    match &e.event_filter {
                        ReplacementEventFilter::EntersBattlefield { permanent_id: pid } => {
                            pid == permanent_id
                        }
                        _ => false,
                    }
                })
                .min_by_key(|(_, e)| e.timestamp)
                .map(|(idx, _)| idx);

            let Some(idx) = found else {
                break;
            };

            self.replacement_effects[idx].applied_to.insert(event_id);
            let outcome = self.replacement_effects[idx].replacement.clone();
            let expired = matches!(self.replacement_effects[idx].duration, ReplacementDuration::NextOccurrence);
            if expired {
                self.replacement_effects.remove(idx);
            }

            match outcome {
                ReplacementOutcome::EnterTapped => {
                    if let Some(state) = self.permanent_states.get(permanent_id).cloned() {
                        self.permanent_states.insert(permanent_id.to_owned(), state.with_tapped(true));
                    }
                }
                ReplacementOutcome::EnterWithCounters { counter_type, amount } => {
                    if let Some(state) = self.permanent_states.get(permanent_id).cloned() {
                        let new_state = state.add_counters(&counter_type, amount);
                        self.permanent_states.insert(permanent_id.to_owned(), new_state);
                    }
                }
                _ => {
                    // Not an ETB-applicable outcome.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::{CardType, ZoneName};

    // =========================================================================
    // Helper functions
    // =========================================================================

    fn make_game_in_first_main() -> (Game, String, String) {
        use crate::domain::actions::Action;
        use crate::domain::types::PlayerId;
        let mut game = Game::create("test-game");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(42)).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        (game, "p1".to_owned(), "p2".to_owned())
    }

    fn make_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_creature_pt(instance_id: &str, owner_id: &str, power: u32, toughness: u32) -> CardInstance {
        let def = CardDefinition::new("creature", "Creature", vec![CardType::Creature])
            .with_power_toughness(power, toughness);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn add_creature_to_battlefield(game: &mut Game, player_id: &str, card: CardInstance) {
        use crate::domain::value_objects::permanent_state::PermanentState;
        let power = card.definition().power().unwrap_or(0) as i32;
        let toughness = card.definition().toughness().unwrap_or(0) as i32;
        let id = card.instance_id().to_owned();
        if let Ok(player) = game.player_state_mut(player_id) {
            player.battlefield.push(card);
        }
        game.permanent_states.insert(id, PermanentState::for_creature(power, toughness));
    }

    fn register_regen(game: &mut Game, creature_id: &str, controller_id: &str) {
        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            format!("regen-{creature_id}"),
            creature_id,
            controller_id,
            ReplacementEventFilter::DestroyPermanent { permanent_id: creature_id.into() },
            ReplacementOutcome::Regenerate,
            ReplacementDuration::NextOccurrence,
            ts,
        ));
    }

    fn register_prevention(game: &mut Game, target_id: &str, controller_id: &str, amount: i32) {
        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            format!("shield-{target_id}"),
            format!("source-{target_id}"),
            controller_id,
            ReplacementEventFilter::DamageToPermanent { permanent_id: target_id.into() },
            ReplacementOutcome::PreventDamage { amount },
            ReplacementDuration::UntilDepleted { remaining: amount },
            ts,
        ));
    }

    // =========================================================================
    // ReplacementEffect struct tests
    // =========================================================================

    #[test]
    fn replacement_effect_new_starts_with_empty_applied_to() {
        let effect = ReplacementEffect::new(
            "regen-1",
            "creature-1",
            "p1",
            ReplacementEventFilter::DestroyPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::Regenerate,
            ReplacementDuration::NextOccurrence,
            1,
        );
        assert!(effect.applied_to.is_empty());
        assert_eq!(effect.effect_id, "regen-1");
        assert_eq!(effect.source_id, "creature-1");
        assert_eq!(effect.timestamp, 1);
    }

    #[test]
    fn destroy_reason_interceptable_for_lethal_damage_and_destroy_effect() {
        assert!(DestroyReason::LethalDamage.is_interceptable());
        assert!(DestroyReason::DestroyEffect.is_interceptable());
        assert!(!DestroyReason::ZeroToughness.is_interceptable());
        assert!(!DestroyReason::Sacrifice.is_interceptable());
    }

    // =========================================================================
    // Framework Registration tests
    // =========================================================================

    #[test]
    fn replacement_effect_can_be_registered_in_game() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let ts = game.next_timestamp();
        let effect = ReplacementEffect::new(
            "regen-1",
            "creature-1",
            &p1,
            ReplacementEventFilter::DestroyPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::Regenerate,
            ReplacementDuration::NextOccurrence,
            ts,
        );
        game.register_replacement_effect(effect);
        assert_eq!(game.replacement_effects.len(), 1);
        assert_eq!(game.replacement_effects[0].effect_id, "regen-1");
    }

    #[test]
    fn cleanup_removes_until_end_of_turn_effects() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let ts = game.next_timestamp();
        let effect = ReplacementEffect::new(
            "shield-1",
            "source-1",
            &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::PreventDamage { amount: 3 },
            ReplacementDuration::UntilEndOfTurn,
            ts,
        );
        game.register_replacement_effect(effect);
        assert_eq!(game.replacement_effects.len(), 1);

        game.cleanup_expired_replacement_effects();

        assert_eq!(game.replacement_effects.len(), 0, "UntilEndOfTurn effect should be removed");
    }

    #[test]
    fn while_source_on_battlefield_effect_removed_when_source_leaves() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-src", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts = game.next_timestamp();
        let effect = ReplacementEffect::new(
            "static-regen",
            "creature-src",
            &p1,
            ReplacementEventFilter::DestroyPermanent { permanent_id: "creature-src".into() },
            ReplacementOutcome::Regenerate,
            ReplacementDuration::WhileSourceOnBattlefield,
            ts,
        );
        game.register_replacement_effect(effect);
        assert_eq!(game.replacement_effects.len(), 1);

        // Sacrifice (bypasses regeneration, but still triggers source-cleanup).
        game.move_permanent_to_graveyard_with_reason("creature-src", DestroyReason::Sacrifice)
            .expect("sacrifice should succeed");

        assert_eq!(game.replacement_effects.len(), 0,
            "WhileSourceOnBattlefield effect removed when source sacrificed");
    }

    #[test]
    fn next_occurrence_effect_removed_after_applying_once() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_regen(&mut game, "creature-1", &p1);
        assert_eq!(game.replacement_effects.len(), 1);

        game.move_permanent_to_graveyard_with_reason("creature-1", DestroyReason::DestroyEffect)
            .expect("regenerate should succeed");

        assert_eq!(game.replacement_effects.len(), 0,
            "NextOccurrence effect consumed after firing");
    }

    // =========================================================================
    // Apply-Once Rule (CR 614.6)
    // =========================================================================

    #[test]
    fn prevention_shield_applied_once_per_event() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("big-creature", &p1, 10, 10);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "shield-1",
            "source-1",
            &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "big-creature".into() },
            ReplacementOutcome::PreventDamage { amount: 3 },
            ReplacementDuration::NextOccurrence,
            ts,
        ));

        let final_amount = game.apply_damage_with_replacement("source", "big-creature", 5, false, false, false);

        assert_eq!(final_amount, 2, "shield should prevent 3 from 5");
        assert_eq!(game.replacement_effects.len(), 0, "shield consumed");
    }

    #[test]
    fn two_different_effects_can_both_apply_to_same_event() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("big-creature", &p1, 10, 10);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts1 = game.next_timestamp();
        let ts2 = game.next_timestamp();

        game.register_replacement_effect(ReplacementEffect::new(
            "shield-1", "source-1", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "big-creature".into() },
            ReplacementOutcome::PreventDamage { amount: 2 },
            ReplacementDuration::NextOccurrence, ts1,
        ));
        game.register_replacement_effect(ReplacementEffect::new(
            "shield-2", "source-2", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "big-creature".into() },
            ReplacementOutcome::PreventDamage { amount: 2 },
            ReplacementDuration::NextOccurrence, ts2,
        ));

        // 7 damage - 2 - 2 = 3
        let final_amount = game.apply_damage_with_replacement("source", "big-creature", 7, false, false, false);
        assert_eq!(final_amount, 3, "both shields apply: 7 - 2 - 2 = 3");
        assert_eq!(game.replacement_effects.len(), 0, "both consumed");
    }

    // =========================================================================
    // Damage Replacement — Prevention Shield
    // =========================================================================

    #[test]
    fn prevention_shield_intercepts_damage_to_target_creature() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_prevention(&mut game, "creature-1", &p1, 3);

        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 3, false, false, false);
        assert_eq!(final_amount, 0, "3-point shield prevents all 3 damage");
        assert_eq!(game.replacement_effects.len(), 0, "shield depleted");
    }

    #[test]
    fn prevention_shield_3pts_intercepts_5_damage_leaves_2() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 6);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_prevention(&mut game, "creature-1", &p1, 3);

        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 5, false, false, false);
        assert_eq!(final_amount, 2, "3-point shield against 5 leaves 2");
        assert_eq!(game.replacement_effects.len(), 0, "shield consumed");
    }

    #[test]
    fn prevention_shield_3pts_intercepts_2_damage_decrements_to_1() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_prevention(&mut game, "creature-1", &p1, 3);

        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 2, false, false, false);
        assert_eq!(final_amount, 0, "3-point shield prevents all 2 damage");
        assert_eq!(game.replacement_effects.len(), 1, "shield still active with 1 remaining");
        match &game.replacement_effects[0].duration {
            ReplacementDuration::UntilDepleted { remaining } => {
                assert_eq!(*remaining, 1, "shield should have 1 remaining");
            }
            _ => panic!("expected UntilDepleted"),
        }
    }

    #[test]
    fn prevention_shield_3pts_intercepts_3_damage_fully_consumed() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_prevention(&mut game, "creature-1", &p1, 3);

        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 3, false, false, false);
        assert_eq!(final_amount, 0, "3-point shield prevents exactly 3 damage");
        assert_eq!(game.replacement_effects.len(), 0, "shield fully consumed");
    }

    #[test]
    fn prevention_shield_does_not_intercept_damage_to_other_creature() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let cx = make_creature_pt("creature-x", &p1, 2, 4);
        let cy = make_creature_pt("creature-y", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, cx);
        add_creature_to_battlefield(&mut game, &p1, cy);
        register_prevention(&mut game, "creature-x", &p1, 3);

        let final_amount = game.apply_damage_with_replacement("source", "creature-y", 3, false, false, false);
        assert_eq!(final_amount, 3, "shield on X should not intercept damage to Y");
        assert_eq!(game.replacement_effects.len(), 1, "shield still active");
    }

    #[test]
    fn creature_with_zero_damage_after_prevention_does_not_trigger_lethal_sba() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1); // 2/2
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_prevention(&mut game, "creature-1", &p1, 5);

        // Apply 2 damage through the framework — all prevented.
        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 2, false, false, false);
        assert_eq!(final_amount, 0);

        // Mark 0 damage (nothing to mark) and run SBA.
        game.perform_state_based_actions();

        let state = game.permanent_states.get("creature-1").expect("creature should exist");
        let cs = state.creature_state().expect("should be a creature");
        assert_eq!(cs.damage_marked_this_turn(), 0, "no damage marked");
        assert!(
            game.players.iter().any(|p| p.battlefield.iter().any(|c| c.instance_id() == "creature-1")),
            "creature should be alive"
        );
    }

    // =========================================================================
    // Destroy Replacement — Regeneration Shield
    // =========================================================================

    #[test]
    fn regeneration_shield_intercepts_destroy_event() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_regen(&mut game, "creature-1", &p1);

        game.move_permanent_to_graveyard_with_reason("creature-1", DestroyReason::DestroyEffect)
            .expect("regenerate");

        assert!(
            game.players.iter().any(|p| p.battlefield.iter().any(|c| c.instance_id() == "creature-1")),
            "creature should survive"
        );
        assert!(
            game.players.iter().all(|p| p.graveyard.is_empty()),
            "graveyard should be empty"
        );
    }

    #[test]
    fn regeneration_taps_creature_and_clears_damage() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);

        // Give it some damage first.
        {
            let state = game.permanent_states.get("creature-1").unwrap().clone();
            let damaged = state.with_damage(1).unwrap();
            game.permanent_states.insert("creature-1".to_owned(), damaged);
        }

        register_regen(&mut game, "creature-1", &p1);
        game.move_permanent_to_graveyard_with_reason("creature-1", DestroyReason::LethalDamage)
            .expect("regenerate");

        let state = game.permanent_states.get("creature-1").unwrap();
        assert!(state.is_tapped(), "regenerated creature should be tapped");
        let cs = state.creature_state().unwrap();
        assert_eq!(cs.damage_marked_this_turn(), 0, "damage cleared on regen");
    }

    #[test]
    fn regeneration_shield_consumed_after_firing() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_regen(&mut game, "creature-1", &p1);
        assert_eq!(game.replacement_effects.len(), 1);

        game.move_permanent_to_graveyard_with_reason("creature-1", DestroyReason::DestroyEffect)
            .expect("regenerate");

        assert_eq!(game.replacement_effects.len(), 0, "regen shield consumed");
    }

    #[test]
    fn regeneration_shield_does_not_intercept_other_creature() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let cx = make_creature("creature-x", &p1);
        let cy = make_creature("creature-y", &p1);
        add_creature_to_battlefield(&mut game, &p1, cx);
        add_creature_to_battlefield(&mut game, &p1, cy);
        register_regen(&mut game, "creature-x", &p1);

        game.move_permanent_to_graveyard_with_reason("creature-y", DestroyReason::DestroyEffect)
            .expect("destroy y");

        assert!(
            game.players.iter().any(|p| p.graveyard.iter().any(|c| c.instance_id() == "creature-y")),
            "creature-y in graveyard"
        );
        assert_eq!(game.replacement_effects.len(), 1, "shield on x untouched");
    }

    #[test]
    fn zero_toughness_sba_bypasses_regeneration_shield() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("zero-toughness", &p1, 2, 0);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_regen(&mut game, "zero-toughness", &p1);

        game.move_permanent_to_graveyard_with_reason("zero-toughness", DestroyReason::ZeroToughness)
            .expect("zero-toughness goes to graveyard");

        assert!(
            game.players.iter().any(|p| p.graveyard.iter().any(|c| c.instance_id() == "zero-toughness")),
            "zero-toughness creature in graveyard"
        );
        // The regen shield was NOT consumed by the zero-toughness SBA (CR 704.5f
        // bypasses destroy replacement). But the source left the battlefield, so
        // remove_replacement_effects_for_source cleans it up to prevent dangling.
        assert_eq!(game.replacement_effects.len(), 0,
            "regen shield cleaned up when source leaves battlefield");
    }

    #[test]
    fn exile_does_not_trigger_regeneration_shield() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_regen(&mut game, "creature-1", &p1);

        game.move_permanent_to_exile("creature-1").expect("exile");

        assert!(
            game.players.iter().any(|p| p.exile.iter().any(|c| c.instance_id() == "creature-1")),
            "creature in exile"
        );
        // The regen shield was NOT consumed by exile (exile bypasses the destroy
        // framework). However, the source left the battlefield and
        // remove_replacement_effects_for_source cleans up ALL effects for that
        // source — including NextOccurrence shields — to prevent dangling entries.
        assert_eq!(game.replacement_effects.len(), 0,
            "regen shield cleaned up when source leaves battlefield");
    }

    // =========================================================================
    // Multiple Replacement Effects (timestamp ordering)
    // =========================================================================

    #[test]
    fn two_shields_apply_in_timestamp_order() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts1 = game.next_timestamp();
        let ts2 = game.next_timestamp();

        game.register_replacement_effect(ReplacementEffect::new(
            "shield-1", "source-1", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::PreventDamage { amount: 2 },
            ReplacementDuration::NextOccurrence, ts1,
        ));
        game.register_replacement_effect(ReplacementEffect::new(
            "shield-2", "source-2", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::PreventDamage { amount: 2 },
            ReplacementDuration::NextOccurrence, ts2,
        ));

        // 3 damage - 2 (shield1, older) = 1, then 1 - 1 (shield2 prevents up to 2, only 1 left) = 0.
        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 3, false, false, false);
        assert_eq!(final_amount, 0, "both shields applied: 3 - 2 - 1 = 0");
    }

    #[test]
    fn damage_reduced_to_zero_second_shield_not_triggered() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts1 = game.next_timestamp();
        let ts2 = game.next_timestamp();

        game.register_replacement_effect(ReplacementEffect::new(
            "shield-1", "source-1", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::PreventDamage { amount: 5 },
            ReplacementDuration::NextOccurrence, ts1,
        ));
        game.register_replacement_effect(ReplacementEffect::new(
            "shield-2", "source-2", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::PreventDamage { amount: 5 },
            ReplacementDuration::NextOccurrence, ts2,
        ));

        // First shield prevents all 3 → second shield not needed.
        let final_amount = game.apply_damage_with_replacement("source", "creature-1", 3, false, false, false);
        assert_eq!(final_amount, 0);
        assert_eq!(game.replacement_effects.len(), 1, "second shield remains untouched");
        assert_eq!(game.replacement_effects[0].effect_id, "shield-2");
    }

    // =========================================================================
    // R12: AllCombatDamage filter
    // =========================================================================

    #[test]
    fn all_combat_damage_filter_matches_when_is_combat_true() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "fog-effect", "fog-spell", &p1,
            ReplacementEventFilter::AllCombatDamage,
            ReplacementOutcome::PreventDamage { amount: 0 },
            ReplacementDuration::UntilEndOfTurn,
            ts,
        ));

        // Combat damage should be intercepted (prevented all).
        let final_amount = game.apply_damage_with_replacement(
            "attacker", "creature-1", 3, false, false, true, // is_combat = true
        );
        assert_eq!(final_amount, 0, "AllCombatDamage filter should prevent combat damage");
        // UntilEndOfTurn: effect remains active
        assert_eq!(game.replacement_effects.len(), 1, "UntilEndOfTurn effect persists");
    }

    #[test]
    fn all_combat_damage_filter_does_not_match_when_is_combat_false() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 2, 4);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "fog-effect", "fog-spell", &p1,
            ReplacementEventFilter::AllCombatDamage,
            ReplacementOutcome::PreventDamage { amount: 0 },
            ReplacementDuration::UntilEndOfTurn,
            ts,
        ));

        // Spell damage should NOT be intercepted.
        let final_amount = game.apply_damage_with_replacement(
            "lightning-strike", "creature-1", 3, false, false, false, // is_combat = false
        );
        assert_eq!(final_amount, 3, "AllCombatDamage filter should NOT prevent spell damage");
        assert_eq!(game.replacement_effects.len(), 1, "effect unchanged");
    }

    #[test]
    fn damage_to_any_filter_matches_both_combat_and_non_combat() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 10, 10);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "global-shield", "some-source", &p1,
            ReplacementEventFilter::DamageToAny,
            ReplacementOutcome::PreventDamage { amount: 0 },
            ReplacementDuration::UntilEndOfTurn,
            ts,
        ));

        // DamageToAny should match combat damage.
        let combat_result = game.apply_damage_with_replacement(
            "attacker", "creature-1", 2, false, false, true,
        );
        assert_eq!(combat_result, 0, "DamageToAny should prevent combat damage");

        // DamageToAny should also match spell damage.
        let spell_result = game.apply_damage_with_replacement(
            "spell", "creature-1", 2, false, false, false,
        );
        assert_eq!(spell_result, 0, "DamageToAny should prevent spell damage too");
    }

    #[test]
    fn damage_to_permanent_filter_matches_regardless_of_is_combat() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature_pt("creature-1", &p1, 10, 10);
        add_creature_to_battlefield(&mut game, &p1, creature);

        let ts = game.next_timestamp();
        // Register a UntilEndOfTurn full-prevention shield (Guardian Shield style)
        game.register_replacement_effect(ReplacementEffect::new(
            "guardian-shield-effect", "guardian-shield-spell", &p1,
            ReplacementEventFilter::DamageToPermanent { permanent_id: "creature-1".into() },
            ReplacementOutcome::PreventDamage { amount: 0 },
            ReplacementDuration::UntilEndOfTurn,
            ts,
        ));

        // Combat damage to the target should be prevented.
        let combat = game.apply_damage_with_replacement(
            "attacker", "creature-1", 4, false, false, true,
        );
        assert_eq!(combat, 0, "DamageToPermanent should prevent combat damage");

        // Spell damage to the same target should also be prevented.
        let spell = game.apply_damage_with_replacement(
            "spell", "creature-1", 3, false, false, false,
        );
        assert_eq!(spell, 0, "DamageToPermanent should prevent spell damage");

        // Effect remains (UntilEndOfTurn, amount:0).
        assert_eq!(game.replacement_effects.len(), 1, "UntilEndOfTurn effect persists");
    }

    // =========================================================================
    // ETB Replacement
    // =========================================================================

    #[test]
    fn enters_tapped_static_ability_preserved() {
        use crate::domain::enums::StaticAbility;
        let (mut game, p1, _p2) = make_game_in_first_main();
        let def = CardDefinition::new("tapland", "Tapland", vec![CardType::Land])
            .with_static_ability(StaticAbility::EntersTapped);
        let card = CardInstance::new("tapland-1", def, &p1);
        game.enter_battlefield(card, &p1, ZoneName::Hand);
        let state = game.permanent_states.get("tapland-1").expect("should exist");
        assert!(state.is_tapped(), "EntersTapped permanent starts tapped");
    }

    #[test]
    fn etb_replacement_enters_with_counters() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "etb-counters-1", "some-source", &p1,
            ReplacementEventFilter::EntersBattlefield { permanent_id: "creature-1".into() },
            ReplacementOutcome::EnterWithCounters { counter_type: "PLUS_ONE_PLUS_ONE".into(), amount: 2 },
            ReplacementDuration::NextOccurrence,
            ts,
        ));

        game.enter_battlefield(creature, &p1, ZoneName::Hand);

        let state = game.permanent_states.get("creature-1").expect("should exist");
        assert_eq!(state.get_counters("PLUS_ONE_PLUS_ONE"), 2,
            "creature should have 2 +1/+1 counters from ETB replacement");
    }

    // =========================================================================
    // Integration with SBA Loop
    // =========================================================================

    #[test]
    fn creature_with_lethal_damage_and_regen_shield_survives_sba() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1); // 2/2
        add_creature_to_battlefield(&mut game, &p1, creature);
        register_regen(&mut game, "creature-1", &p1);

        // Mark lethal damage (2 damage = lethal for 2/2).
        {
            let state = game.permanent_states.get("creature-1").unwrap().clone();
            let damaged = state.with_damage(2).unwrap();
            game.permanent_states.insert("creature-1".to_owned(), damaged);
        }

        game.perform_state_based_actions();

        assert!(
            game.players.iter().any(|p| p.battlefield.iter().any(|c| c.instance_id() == "creature-1")),
            "creature survives via regen"
        );
        assert!(
            game.players.iter().all(|p| p.graveyard.is_empty()),
            "graveyard should be empty"
        );
        let state = game.permanent_states.get("creature-1").unwrap();
        assert!(state.is_tapped(), "regenerated creature is tapped");
        let cs = state.creature_state().unwrap();
        assert_eq!(cs.damage_marked_this_turn(), 0, "damage cleared");
    }

    #[test]
    fn creature_with_lethal_damage_no_shield_goes_to_graveyard() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let creature = make_creature("creature-1", &p1);
        add_creature_to_battlefield(&mut game, &p1, creature);

        {
            let state = game.permanent_states.get("creature-1").unwrap().clone();
            let damaged = state.with_damage(2).unwrap();
            game.permanent_states.insert("creature-1".to_owned(), damaged);
        }

        game.perform_state_based_actions();

        assert!(
            game.players.iter().any(|p| p.graveyard.iter().any(|c| c.instance_id() == "creature-1")),
            "creature in graveyard"
        );
    }

    // =========================================================================
    // Integration with Combat Damage (damage replacement)
    // =========================================================================

    #[test]
    fn prevention_shield_intercepts_combat_damage_to_creature() {
        use crate::domain::actions::Action;
        use crate::domain::enums::Step;
        use crate::domain::types::PlayerId;
        use crate::domain::value_objects::permanent_state::PermanentState;

        let mut game = Game::create("combat-test");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(42)).unwrap();

        // Advance to DeclareAttackers (5 steps from Untap).
        for _ in 0..5 {
            let current = game.turn_state.current_player_id().as_str().to_owned();
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&current) }).unwrap();
        }
        assert_eq!(game.turn_state.current_step(), Step::DeclareAttackers);

        // Add 3/3 attacker for p1 and 1/3 blocker for p2.
        let attacker_def = CardDefinition::new("attacker", "Attacker", vec![CardType::Creature])
            .with_power_toughness(3, 3);
        let blocker_def = CardDefinition::new("blocker", "Blocker", vec![CardType::Creature])
            .with_power_toughness(1, 3);
        let attacker = CardInstance::new("attacker-1", attacker_def, "p1");
        let blocker = CardInstance::new("blocker-1", blocker_def, "p2");

        // Add to battlefield.
        game.permanent_states.insert("attacker-1".to_owned(), {
            let s = PermanentState::for_creature(3, 3);
            s.with_summoning_sickness(false).unwrap()
        });
        game.players.iter_mut().find(|p| p.player_id.as_str() == "p1")
            .unwrap().battlefield.push(attacker);
        game.permanent_states.insert("blocker-1".to_owned(), PermanentState::for_creature(1, 3));
        game.players.iter_mut().find(|p| p.player_id.as_str() == "p2")
            .unwrap().battlefield.push(blocker);

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new("p1"),
            creature_id: CardInstanceId::new("attacker-1"),
        }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        assert_eq!(game.turn_state.current_step(), Step::DeclareBlockers);

        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new("p2"),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        }).unwrap();

        // Advance to FirstStrikeDamage.
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        assert_eq!(game.turn_state.current_step(), Step::FirstStrikeDamage);

        // Register prevention shield BEFORE advancing to CombatDamage.
        // Combat damage resolves when the CombatDamage step is entered.
        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "shield-1", "source-1", "p2",
            ReplacementEventFilter::DamageToPermanent { permanent_id: "blocker-1".into() },
            ReplacementOutcome::PreventDamage { amount: 3 },
            ReplacementDuration::UntilDepleted { remaining: 3 },
            ts,
        ));

        // Advance to CombatDamage — the step handler fires combat damage with shield active.
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        assert_eq!(game.turn_state.current_step(), Step::CombatDamage);

        // Blocker should have 0 damage marked.
        if let Some(state) = game.permanent_states.get("blocker-1") {
            let cs = state.creature_state().unwrap();
            assert_eq!(cs.damage_marked_this_turn(), 0,
                "prevention shield prevented all 3 combat damage to blocker");
        } else {
            panic!("blocker-1 should still exist on battlefield (0 damage marked, not destroyed)");
        }
    }

    #[test]
    fn prevention_shield_on_player_intercepts_unblocked_combat_damage() {
        use crate::domain::actions::Action;
        use crate::domain::enums::Step;
        use crate::domain::types::PlayerId;
        use crate::domain::value_objects::permanent_state::PermanentState;

        let mut game = Game::create("combat-test-player");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(42)).unwrap();

        let initial_life_p2 = game.players.iter().find(|p| p.player_id.as_str() == "p2").unwrap().life_total;

        // Advance to DeclareAttackers.
        for _ in 0..5 {
            let current = game.turn_state.current_player_id().as_str().to_owned();
            game.apply(Action::AdvanceStep { player_id: PlayerId::new(&current) }).unwrap();
        }
        assert_eq!(game.turn_state.current_step(), Step::DeclareAttackers);

        let attacker_def = CardDefinition::new("attacker", "Attacker", vec![CardType::Creature])
            .with_power_toughness(3, 3);
        let attacker = CardInstance::new("attacker-1", attacker_def, "p1");
        game.permanent_states.insert("attacker-1".to_owned(), {
            PermanentState::for_creature(3, 3).with_summoning_sickness(false).unwrap()
        });
        game.players.iter_mut().find(|p| p.player_id.as_str() == "p1")
            .unwrap().battlefield.push(attacker);

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new("p1"),
            creature_id: CardInstanceId::new("attacker-1"),
        }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        assert_eq!(game.turn_state.current_step(), Step::DeclareBlockers);
        // No blockers — advance to FirstStrikeDamage.
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        assert_eq!(game.turn_state.current_step(), Step::FirstStrikeDamage);

        // Register prevention shield BEFORE advancing to CombatDamage.
        // Combat damage resolves when the CombatDamage step is entered.
        let ts = game.next_timestamp();
        game.register_replacement_effect(ReplacementEffect::new(
            "shield-p2", "source-1", "p2",
            ReplacementEventFilter::DamageToPlayer { player_id: "p2".into() },
            ReplacementOutcome::PreventDamage { amount: 2 },
            ReplacementDuration::UntilDepleted { remaining: 2 },
            ts,
        ));

        // Advance to CombatDamage — the step handler fires combat damage with shield active.
        game.apply(Action::AdvanceStep { player_id: PlayerId::new("p1") }).unwrap();
        assert_eq!(game.turn_state.current_step(), Step::CombatDamage);

        let life_after = game.players.iter().find(|p| p.player_id.as_str() == "p2").unwrap().life_total;
        assert_eq!(life_after, initial_life_p2 - 1,
            "prevention shield reduces 3 combat damage to 1 (p2 loses only 1 life)");
    }
}
