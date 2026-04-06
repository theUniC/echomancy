use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::types::CardInstanceId;
use crate::domain::value_objects::creature_state::CreatureSubState;

// ============================================================================
// EffectDuration
// ============================================================================

/// How long a continuous effect lasts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDuration {
    /// The effect ends at the Cleanup step of the current turn.
    UntilEndOfTurn,
    /// The effect lasts as long as the source permanent (identified by its ID)
    /// remains on the battlefield (CR 613.7a).
    ///
    /// Used for:
    /// - A permanent's own static ability effects (source_id = the permanent itself).
    /// - Aura/equipment effects where the enchanting/equipped relationship persists.
    WhileSourceOnBattlefield(String),
}

// ============================================================================
// ContinuousEffect
// ============================================================================

/// A temporary modification to a permanent's characteristics.
///
/// Created by instants or triggered abilities that say "until end of turn".
/// Stored on [`PermanentState`] and removed when their duration expires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousEffect {
    /// Power modifier (may be negative).
    pub power_modifier: i32,
    /// Toughness modifier (may be negative).
    pub toughness_modifier: i32,
    /// When this effect expires.
    pub duration: EffectDuration,
    /// Instance ID of the spell or ability that created this effect.
    pub source_id: String,
}

// ============================================================================
// PermanentStateError
// ============================================================================

/// Error type for operations on [`PermanentState`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PermanentStateError {
    #[error("Cannot use creature-specific operation on non-creature permanent")]
    NotACreature,
}

// ============================================================================
// PermanentStateSnapshot
// ============================================================================

/// Serialisable snapshot of a permanent's complete state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermanentStateSnapshot {
    pub is_tapped: bool,
    /// Counter type name → count.
    pub counters: HashMap<String, u32>,
    pub creature_state: Option<CreatureSubState>,
}

// ============================================================================
// PermanentState
// ============================================================================

/// Immutable snapshot of a permanent's state on the battlefield.
///
/// Supports all permanent types (creatures, artifacts, enchantments, lands,
/// planeswalkers). All mutating operations return **new** instances.
///
/// Mirrors the TypeScript `PermanentState` class from `PermanentState.ts`.
///
/// # Examples
///
/// ```
/// use echomancy_core::prelude::PermanentState;
///
/// let state = PermanentState::for_non_creature();
/// let tapped = state.with_tapped(true);
/// assert!(tapped.is_tapped());
/// assert!(!state.is_tapped()); // original unchanged
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermanentState {
    is_tapped: bool,
    counters: HashMap<String, u32>,
    creature_state: Option<CreatureSubState>,
    continuous_effects: Vec<ContinuousEffect>,
    /// The timestamp assigned when this permanent entered the battlefield (CR 613.7d).
    ///
    /// Used by the layer system to determine effect ordering: static-ability
    /// effects from this permanent inherit this timestamp (CR 613.7a).
    etb_timestamp: u64,
}

impl PermanentState {
    // ---- constructors -------------------------------------------------------

    /// Creates a `PermanentState` for a creature entering the battlefield.
    ///
    /// The creature has summoning sickness and starts untapped with no counters.
    /// `etb_timestamp` defaults to 0; the `Game` aggregate sets the correct
    /// monotonically increasing timestamp when the permanent actually ETBs.
    pub fn for_creature(base_power: i32, base_toughness: i32) -> Self {
        Self {
            is_tapped: false,
            counters: HashMap::new(),
            creature_state: Some(CreatureSubState::new(base_power, base_toughness)),
            continuous_effects: Vec::new(),
            etb_timestamp: 0,
        }
    }

    /// Creates a `PermanentState` for a non-creature permanent (land, artifact,
    /// enchantment, planeswalker) entering the battlefield.
    pub fn for_non_creature() -> Self {
        Self {
            is_tapped: false,
            counters: HashMap::new(),
            creature_state: None,
            continuous_effects: Vec::new(),
            etb_timestamp: 0,
        }
    }

    /// Reconstructs a `PermanentState` from a previously captured snapshot.
    pub fn from_snapshot(snapshot: PermanentStateSnapshot) -> Self {
        Self {
            is_tapped: snapshot.is_tapped,
            counters: snapshot.counters,
            creature_state: snapshot.creature_state,
            continuous_effects: Vec::new(),
            etb_timestamp: 0,
        }
    }

    // ---- accessors ----------------------------------------------------------

    pub fn is_tapped(&self) -> bool {
        self.is_tapped
    }

    /// Returns `Some(&CreatureSubState)` if this permanent is a creature,
    /// `None` otherwise.
    pub fn creature_state(&self) -> Option<&CreatureSubState> {
        self.creature_state.as_ref()
    }

    /// Returns the count of counters of the given type (0 if none).
    pub fn get_counters(&self, counter_type: &str) -> u32 {
        self.counters.get(counter_type).copied().unwrap_or(0)
    }

    /// Returns the ETB timestamp for this permanent (CR 613.7d).
    ///
    /// The timestamp is assigned by `Game::enter_battlefield` and is used by
    /// the layer system to determine effect ordering.
    #[allow(dead_code)]
    pub(crate) fn etb_timestamp(&self) -> u64 {
        self.etb_timestamp
    }

    /// Returns a new `PermanentState` with the given ETB timestamp.
    ///
    /// Called by `Game::enter_battlefield` immediately after creating the state.
    pub(crate) fn with_etb_timestamp(mut self, ts: u64) -> Self {
        self.etb_timestamp = ts;
        self
    }

    // ---- continuous effects accessors ---------------------------------------

    /// Returns the list of active continuous effects on this permanent.
    pub fn continuous_effects(&self) -> &[ContinuousEffect] {
        &self.continuous_effects
    }

    // ---- continuous effects builders ----------------------------------------

    /// Returns a new `PermanentState` with the given continuous effect added.
    pub fn with_continuous_effect(self, effect: ContinuousEffect) -> Self {
        let mut next = self;
        next.continuous_effects.push(effect);
        next
    }

    /// Returns a new `PermanentState` with all continuous effects matching
    /// the given duration removed.
    pub fn without_expired_effects(self, duration: EffectDuration) -> Self {
        let mut next = self;
        next.continuous_effects.retain(|e| e.duration != duration);
        next
    }

    // ---- common builders (all permanents) -----------------------------------

    /// Returns a new `PermanentState` with `is_tapped` set to the given value.
    pub fn with_tapped(&self, is_tapped: bool) -> Self {
        let mut next = self.clone();
        next.is_tapped = is_tapped;
        next
    }

    /// Returns a new `PermanentState` with `amount` counters of `counter_type`
    /// added to the existing count.
    pub fn add_counters(&self, counter_type: &str, amount: u32) -> Self {
        let mut next = self.clone();
        let entry = next.counters.entry(counter_type.to_owned()).or_insert(0);
        *entry += amount;
        next
    }

    /// Returns a new `PermanentState` with up to `amount` counters of
    /// `counter_type` removed (clamped to 0; removes the key when 0).
    pub fn remove_counters(&self, counter_type: &str, amount: u32) -> Self {
        let mut next = self.clone();
        let current = next.counters.get(counter_type).copied().unwrap_or(0);
        let new_value = current.saturating_sub(amount);
        if new_value == 0 {
            next.counters.remove(counter_type);
        } else {
            next.counters.insert(counter_type.to_owned(), new_value);
        }
        next
    }

    // ---- creature builders --------------------------------------------------

    /// Returns `Err` if called on a non-creature permanent.
    fn require_creature_state(&self) -> Result<&CreatureSubState, PermanentStateError> {
        self.creature_state
            .as_ref()
            .ok_or(PermanentStateError::NotACreature)
    }

    /// Returns a new `PermanentState` with `is_attacking` set to `attacking`.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub fn with_attacking(&self, attacking: bool) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_attacking(attacking));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `has_attacked_this_turn` updated.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub fn with_has_attacked_this_turn(&self, value: bool) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_has_attacked_this_turn(value));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `has_summoning_sickness` updated.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub fn with_summoning_sickness(&self, value: bool) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_summoning_sickness(value));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `damage_marked_this_turn` set to
    /// `damage`.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub fn with_damage(&self, damage: i32) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_damage(damage));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `blocking_creature_id` updated.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub fn with_blocking_creature_id(
        &self,
        id: Option<CardInstanceId>,
    ) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_blocking_creature_id(id));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `blocked_by` updated.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub fn with_blocked_by(&self, id: Option<CardInstanceId>) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_blocked_by(id));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `dealt_first_strike_damage` updated.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this permanent has no
    /// creature sub-state.
    pub(crate) fn with_dealt_first_strike_damage(
        &self,
        val: bool,
    ) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.with_dealt_first_strike_damage(val));
        Ok(next)
    }

    /// Returns a new `PermanentState` with `has_deathtouch_damage` set to `true`.
    ///
    /// Used when damage from a Deathtouch source has been marked on this creature.
    /// Returns `self` unchanged if this is not a creature (no-op rather than error,
    /// since this is always called from code that has already confirmed creature-ness).
    pub(crate) fn with_deathtouch_damage(&self) -> Self {
        match &self.creature_state {
            Some(cs) => {
                let mut next = self.clone();
                next.creature_state = Some(cs.with_deathtouch_damage());
                next
            }
            None => self.clone(),
        }
    }

    // ---- derived creature stats --------------------------------------------

    /// Returns the current power, including +1/+1 and -1/-1 counter effects and
    /// continuous effect modifiers.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this is not a creature.
    pub fn current_power(&self) -> Result<i32, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let plus_counters = self.get_counters("PLUS_ONE_PLUS_ONE") as i32;
        let minus_counters = self.get_counters("MINUS_ONE_MINUS_ONE") as i32;
        let effects: i32 = self.continuous_effects.iter().map(|e| e.power_modifier).sum();
        Ok(cs.base_power() + plus_counters - minus_counters + effects)
    }

    /// Returns the current toughness, including +1/+1 and -1/-1 counter effects and
    /// continuous effect modifiers.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this is not a creature.
    pub fn current_toughness(&self) -> Result<i32, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let plus_counters = self.get_counters("PLUS_ONE_PLUS_ONE") as i32;
        let minus_counters = self.get_counters("MINUS_ONE_MINUS_ONE") as i32;
        let effects: i32 = self.continuous_effects.iter().map(|e| e.toughness_modifier).sum();
        Ok(cs.base_toughness() + plus_counters - minus_counters + effects)
    }

    /// Returns `true` if the creature has taken lethal damage this turn.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this is not a creature.
    pub fn has_lethal_damage(&self) -> Result<bool, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let toughness = self.current_toughness()?;
        Ok(cs.damage_marked_this_turn() >= toughness)
    }

    // ---- turn reset operations ---------------------------------------------

    /// Returns a new `PermanentState` with all combat flags reset for a new
    /// turn and summoning sickness cleared.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this is not a creature.
    pub fn reset_for_new_turn(&self) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.reset_for_new_turn());
        Ok(next)
    }

    /// Returns a new `PermanentState` with `damage_marked_this_turn` cleared.
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this is not a creature.
    pub fn clear_damage(&self) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.clear_damage());
        Ok(next)
    }

    /// Returns a new `PermanentState` with combat state cleared (end of
    /// combat).
    ///
    /// # Errors
    ///
    /// Returns [`PermanentStateError::NotACreature`] if this is not a creature.
    pub fn clear_combat_state(&self) -> Result<Self, PermanentStateError> {
        let cs = self.require_creature_state()?;
        let mut next = self.clone();
        next.creature_state = Some(cs.clear_combat_state());
        Ok(next)
    }

    // ---- snapshot ----------------------------------------------------------

    /// Exports the state as a serialisable snapshot.
    pub fn to_snapshot(&self) -> PermanentStateSnapshot {
        PermanentStateSnapshot {
            is_tapped: self.is_tapped,
            counters: self.counters.clone(),
            creature_state: self.creature_state.clone(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- for_creature -------------------------------------------------------

    #[test]
    fn for_creature_starts_untapped_with_sickness() {
        let state = PermanentState::for_creature(2, 3);
        assert!(!state.is_tapped());
        let cs = state.creature_state().expect("should have creature state");
        assert!(cs.has_summoning_sickness());
        assert!(!cs.is_attacking());
        assert_eq!(cs.base_power(), 2);
        assert_eq!(cs.base_toughness(), 3);
        assert_eq!(cs.damage_marked_this_turn(), 0);
        assert!(cs.blocking_creature_id().is_none());
        assert!(cs.blocked_by().is_empty());
    }

    // ---- for_non_creature --------------------------------------------------

    #[test]
    fn for_non_creature_has_no_creature_state() {
        let state = PermanentState::for_non_creature();
        assert!(!state.is_tapped());
        assert!(state.creature_state().is_none());
    }

    // ---- with_tapped -------------------------------------------------------

    #[test]
    fn with_tapped_returns_new_instance_original_unchanged() {
        let state = PermanentState::for_non_creature();
        let tapped = state.with_tapped(true);

        assert!(!state.is_tapped()); // original unchanged
        assert!(tapped.is_tapped());
    }

    #[test]
    fn with_tapped_false_untaps() {
        let state = PermanentState::for_non_creature().with_tapped(true);
        let untapped = state.with_tapped(false);
        assert!(!untapped.is_tapped());
    }

    // ---- counters ----------------------------------------------------------

    #[test]
    fn add_counters_accumulates() {
        let state = PermanentState::for_non_creature()
            .add_counters("CHARGE", 2)
            .add_counters("CHARGE", 1);
        assert_eq!(state.get_counters("CHARGE"), 3);
    }

    #[test]
    fn add_counters_original_unchanged() {
        let state = PermanentState::for_non_creature();
        let _ = state.add_counters("CHARGE", 3);
        assert_eq!(state.get_counters("CHARGE"), 0);
    }

    #[test]
    fn remove_counters_clamps_to_zero() {
        let state = PermanentState::for_non_creature()
            .add_counters("CHARGE", 2)
            .remove_counters("CHARGE", 5);
        assert_eq!(state.get_counters("CHARGE"), 0);
    }

    #[test]
    fn remove_counters_removes_key_at_zero() {
        let state = PermanentState::for_non_creature()
            .add_counters("CHARGE", 2)
            .remove_counters("CHARGE", 2);
        // Counter key should be gone; get_counters returns 0
        assert_eq!(state.get_counters("CHARGE"), 0);
    }

    #[test]
    fn get_counters_returns_zero_for_unknown_type() {
        let state = PermanentState::for_non_creature();
        assert_eq!(state.get_counters("UNKNOWN"), 0);
    }

    // ---- creature-specific builders ----------------------------------------

    #[test]
    fn with_attacking_sets_flag() {
        let state = PermanentState::for_creature(2, 2);
        let attacking = state.with_attacking(true).unwrap();
        assert!(attacking.creature_state().unwrap().is_attacking());
        assert!(!state.creature_state().unwrap().is_attacking()); // original unchanged
    }

    #[test]
    fn with_attacking_on_non_creature_returns_err() {
        let state = PermanentState::for_non_creature();
        assert!(state.with_attacking(true).is_err());
    }

    #[test]
    fn with_summoning_sickness_updates_flag() {
        let state = PermanentState::for_creature(1, 1);
        let no_sick = state.with_summoning_sickness(false).unwrap();
        assert!(!no_sick.creature_state().unwrap().has_summoning_sickness());
    }

    #[test]
    fn with_damage_updates_damage() {
        let state = PermanentState::for_creature(3, 3);
        let damaged = state.with_damage(2).unwrap();
        assert_eq!(damaged.creature_state().unwrap().damage_marked_this_turn(), 2);
    }

    // ---- derived stats -----------------------------------------------------

    #[test]
    fn current_power_includes_plus_counters() {
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 2);
        assert_eq!(state.current_power().unwrap(), 4);
    }

    #[test]
    fn current_toughness_includes_plus_counters() {
        let state = PermanentState::for_creature(2, 3)
            .add_counters("PLUS_ONE_PLUS_ONE", 1);
        assert_eq!(state.current_toughness().unwrap(), 4);
    }

    #[test]
    fn has_lethal_damage_true_when_damage_gte_toughness() {
        let state = PermanentState::for_creature(2, 2)
            .with_damage(2)
            .unwrap();
        assert!(state.has_lethal_damage().unwrap());
    }

    #[test]
    fn has_lethal_damage_false_when_damage_less_than_toughness() {
        let state = PermanentState::for_creature(2, 3)
            .with_damage(2)
            .unwrap();
        assert!(!state.has_lethal_damage().unwrap());
    }

    #[test]
    fn current_power_on_non_creature_returns_err() {
        let state = PermanentState::for_non_creature();
        assert!(state.current_power().is_err());
    }

    // ---- reset operations --------------------------------------------------

    #[test]
    fn reset_for_new_turn_clears_combat_and_sickness() {
        let state = PermanentState::for_creature(2, 2)
            .with_attacking(true)
            .unwrap()
            .with_damage(1)
            .unwrap();
        let reset = state.reset_for_new_turn().unwrap();
        let cs = reset.creature_state().unwrap();
        assert!(!cs.is_attacking());
        assert!(!cs.has_attacked_this_turn());
        assert_eq!(cs.damage_marked_this_turn(), 0);
        assert!(!cs.has_summoning_sickness());
    }

    #[test]
    fn clear_damage_zeroes_damage() {
        let state = PermanentState::for_creature(2, 2).with_damage(3).unwrap();
        let cleared = state.clear_damage().unwrap();
        assert_eq!(cleared.creature_state().unwrap().damage_marked_this_turn(), 0);
    }

    #[test]
    fn clear_combat_state_clears_attacking_and_blocking() {
        let id = CardInstanceId::new("attacker-1");
        let state = PermanentState::for_creature(2, 2)
            .with_attacking(true)
            .unwrap()
            .with_blocking_creature_id(Some(id.clone()))
            .unwrap();
        let cleared = state.clear_combat_state().unwrap();
        let cs = cleared.creature_state().unwrap();
        assert!(!cs.is_attacking());
        assert!(cs.blocking_creature_id().is_none());
    }

    // ---- clear_combat_state preserves damage --------------------------------

    #[test]
    fn clear_combat_state_preserves_damage() {
        let state = PermanentState::for_creature(2, 4)
            .with_damage(3)
            .unwrap()
            .with_attacking(true)
            .unwrap();
        let cleared = state.clear_combat_state().unwrap();
        assert_eq!(cleared.creature_state().unwrap().damage_marked_this_turn(), 3);
    }

    // ---- has_lethal_damage with +1/+1 counters ------------------------------

    // ---- -1/-1 counters (P10.2) -------------------------------------------

    #[test]
    fn minus_one_minus_one_counters_reduce_power_and_toughness() {
        let state = PermanentState::for_creature(3, 3)
            .add_counters("MINUS_ONE_MINUS_ONE", 2);
        assert_eq!(state.current_power().unwrap(), 1, "3 - 2 = 1 power");
        assert_eq!(state.current_toughness().unwrap(), 1, "3 - 2 = 1 toughness");
    }

    #[test]
    fn minus_counters_reduce_effective_toughness_to_zero_triggers_sba() {
        // A 1/1 with 1 -1/-1 counter has effective 0/0 — SBA should kill it.
        let state = PermanentState::for_creature(1, 1)
            .add_counters("MINUS_ONE_MINUS_ONE", 1);
        assert_eq!(state.current_toughness().unwrap(), 0);
        // has_lethal_damage uses current_toughness; 0 toughness kills via SBA check.
        // (The SBA itself is in find_creatures_to_destroy which checks current_toughness <= 0)
    }

    #[test]
    fn plus_and_minus_counters_partially_cancel() {
        // 3 +1/+1 and 1 -1/-1 = net +2/+2 on base 2/2 → 4/4
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 3)
            .add_counters("MINUS_ONE_MINUS_ONE", 1);
        assert_eq!(state.current_power().unwrap(), 4);
        assert_eq!(state.current_toughness().unwrap(), 4);
    }

    #[test]
    fn has_lethal_damage_false_with_plus_counters_boosting_toughness() {
        // 3-toughness creature + 2 +1/+1 counters = 5 effective toughness
        // 4 damage should NOT be lethal
        let state = PermanentState::for_creature(2, 3)
            .add_counters("PLUS_ONE_PLUS_ONE", 2)
            .with_damage(4)
            .unwrap();
        assert!(!state.has_lethal_damage().unwrap());
    }

    // ---- with_has_attacked_this_turn on non-creature ------------------------

    #[test]
    fn with_has_attacked_this_turn_on_non_creature_returns_err() {
        let state = PermanentState::for_non_creature();
        assert!(state.with_has_attacked_this_turn(true).is_err());
    }

    // ---- snapshot round-trip -----------------------------------------------

    #[test]
    fn snapshot_roundtrip_non_creature() {
        let original = PermanentState::for_non_creature()
            .with_tapped(true)
            .add_counters("LORE", 2);
        let snap = original.to_snapshot();
        let restored = PermanentState::from_snapshot(snap);
        assert_eq!(original, restored);
    }

    #[test]
    fn snapshot_roundtrip_creature() {
        let original = PermanentState::for_creature(3, 4)
            .with_damage(1)
            .unwrap()
            .add_counters("PLUS_ONE_PLUS_ONE", 1);
        let snap = original.to_snapshot();
        let restored = PermanentState::from_snapshot(snap);
        assert_eq!(original, restored);
    }

    // ---- continuous effects -------------------------------------------------

    #[test]
    fn continuous_effect_modifies_current_power() {
        let effect = ContinuousEffect {
            power_modifier: 3,
            toughness_modifier: 3,
            duration: EffectDuration::UntilEndOfTurn,
            source_id: "giant-growth-1".to_owned(),
        };
        let state = PermanentState::for_creature(2, 2).with_continuous_effect(effect);
        assert_eq!(state.current_power().unwrap(), 5);
        assert_eq!(state.current_toughness().unwrap(), 5);
    }

    #[test]
    fn continuous_effect_stacks_with_plus_counters() {
        let effect = ContinuousEffect {
            power_modifier: 3,
            toughness_modifier: 3,
            duration: EffectDuration::UntilEndOfTurn,
            source_id: "giant-growth-1".to_owned(),
        };
        // 2/2 base + 1 counter + 3 effect = 6/6
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 1)
            .with_continuous_effect(effect);
        assert_eq!(state.current_power().unwrap(), 6);
        assert_eq!(state.current_toughness().unwrap(), 6);
    }

    #[test]
    fn continuous_effects_returns_slice() {
        let effect = ContinuousEffect {
            power_modifier: 1,
            toughness_modifier: 2,
            duration: EffectDuration::UntilEndOfTurn,
            source_id: "source-1".to_owned(),
        };
        let state = PermanentState::for_creature(2, 2).with_continuous_effect(effect.clone());
        assert_eq!(state.continuous_effects().len(), 1);
        assert_eq!(state.continuous_effects()[0], effect);
    }

    #[test]
    fn without_expired_effects_removes_matching_duration() {
        let effect = ContinuousEffect {
            power_modifier: 3,
            toughness_modifier: 3,
            duration: EffectDuration::UntilEndOfTurn,
            source_id: "giant-growth-1".to_owned(),
        };
        let state = PermanentState::for_creature(2, 2).with_continuous_effect(effect);
        assert_eq!(state.current_power().unwrap(), 5);

        let expired = state.without_expired_effects(EffectDuration::UntilEndOfTurn);
        assert_eq!(expired.continuous_effects().len(), 0);
        assert_eq!(expired.current_power().unwrap(), 2);
        assert_eq!(expired.current_toughness().unwrap(), 2);
    }

    #[test]
    fn with_continuous_effect_original_unchanged() {
        let state = PermanentState::for_creature(2, 2);
        let effect = ContinuousEffect {
            power_modifier: 3,
            toughness_modifier: 3,
            duration: EffectDuration::UntilEndOfTurn,
            source_id: "source-1".to_owned(),
        };
        let _ = state.clone().with_continuous_effect(effect);
        // Original is unchanged (no continuous effects)
        assert_eq!(state.continuous_effects().len(), 0);
        assert_eq!(state.current_power().unwrap(), 2);
    }
}
