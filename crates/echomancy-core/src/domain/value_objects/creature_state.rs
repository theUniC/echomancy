use serde::{Deserialize, Serialize};

use crate::domain::types::CardInstanceId;

// ============================================================================
// CreatureSubState
// ============================================================================

/// Creature-specific state: combat flags, base stats, and damage.
///
/// Only present on permanents that are creatures.
/// Mirrors the TypeScript `CreatureSubState` type from `PermanentState.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatureSubState {
    pub(crate) base_power: i32,
    pub(crate) base_toughness: i32,
    pub(crate) has_summoning_sickness: bool,
    pub(crate) is_attacking: bool,
    pub(crate) has_attacked_this_turn: bool,
    pub(crate) damage_marked_this_turn: i32,
    pub(crate) blocking_creature_id: Option<CardInstanceId>,
    /// Instance IDs of creatures currently blocking this creature.
    /// Empty when unblocked; may contain multiple IDs when multiple blockers are declared.
    pub(crate) blocked_by: Vec<CardInstanceId>,
    /// Set to `true` when this creature has dealt damage in the `FirstStrikeDamage` step.
    ///
    /// Used to prevent the creature from dealing damage again in the regular
    /// `CombatDamage` step (CR 702.7c). Cleared at end of combat.
    pub(crate) dealt_first_strike_damage: bool,
    /// Set to `true` when any damage from a Deathtouch source has been marked on this
    /// creature. Any non-zero damage from a Deathtouch source is lethal (CR 702.2).
    /// Cleared alongside `damage_marked_this_turn` during Cleanup.
    pub(crate) has_deathtouch_damage: bool,
}

impl CreatureSubState {
    /// Returns a new `CreatureSubState` for a creature entering the battlefield.
    ///
    /// The creature has summoning sickness and all combat flags cleared.
    pub(crate) fn new(base_power: i32, base_toughness: i32) -> Self {
        Self {
            base_power,
            base_toughness,
            has_summoning_sickness: true,
            is_attacking: false,
            has_attacked_this_turn: false,
            damage_marked_this_turn: 0,
            blocking_creature_id: None,
            blocked_by: Vec::new(),
            dealt_first_strike_damage: false,
            has_deathtouch_damage: false,
        }
    }

    // ---- public accessors --------------------------------------------------

    /// Returns the base power of the creature (before counters or continuous effects).
    pub fn base_power(&self) -> i32 {
        self.base_power
    }

    /// Returns the base toughness of the creature (before counters or continuous effects).
    pub fn base_toughness(&self) -> i32 {
        self.base_toughness
    }

    /// Returns `true` if the creature is currently attacking.
    pub fn is_attacking(&self) -> bool {
        self.is_attacking
    }

    /// Returns `true` if the creature has already attacked this turn.
    pub fn has_attacked_this_turn(&self) -> bool {
        self.has_attacked_this_turn
    }

    /// Returns `true` if the creature has summoning sickness.
    pub fn has_summoning_sickness(&self) -> bool {
        self.has_summoning_sickness
    }

    /// Returns the amount of damage marked on the creature this turn.
    pub fn damage_marked_this_turn(&self) -> i32 {
        self.damage_marked_this_turn
    }

    /// Returns the instance ID of the creature this creature is blocking, if any.
    pub fn blocking_creature_id(&self) -> Option<&str> {
        self.blocking_creature_id.as_ref().map(|id| id.as_str())
    }

    /// Returns the instance IDs of all creatures currently blocking this creature.
    pub fn blocked_by(&self) -> &[CardInstanceId] {
        &self.blocked_by
    }

    /// Returns `true` if at least one creature is blocking this creature.
    pub fn is_blocked(&self) -> bool {
        !self.blocked_by.is_empty()
    }

    /// Returns `true` if this creature already dealt damage in the `FirstStrikeDamage` step.
    pub fn dealt_first_strike_damage(&self) -> bool {
        self.dealt_first_strike_damage
    }

    /// Returns `true` if any damage from a Deathtouch source has been marked on this creature.
    pub fn has_deathtouch_damage(&self) -> bool {
        self.has_deathtouch_damage
    }

    // ---- builder methods ---------------------------------------------------

    /// Returns a new `CreatureSubState` with `is_attacking` set to `attacking`.
    pub(crate) fn with_attacking(&self, attacking: bool) -> Self {
        Self {
            is_attacking: attacking,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `has_attacked_this_turn` updated.
    pub(crate) fn with_has_attacked_this_turn(&self, value: bool) -> Self {
        Self {
            has_attacked_this_turn: value,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `has_summoning_sickness` updated.
    pub(crate) fn with_summoning_sickness(&self, value: bool) -> Self {
        Self {
            has_summoning_sickness: value,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `damage_marked_this_turn` set to `damage`.
    pub(crate) fn with_damage(&self, damage: i32) -> Self {
        Self {
            damage_marked_this_turn: damage,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `blocking_creature_id` updated.
    pub(crate) fn with_blocking_creature_id(&self, id: Option<CardInstanceId>) -> Self {
        Self {
            blocking_creature_id: id,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with the given blocker ID added to `blocked_by`.
    ///
    /// Passing `None` clears all blockers (used for reset).
    /// Passing `Some(id)` appends the ID to the list (multiple blockers supported).
    pub(crate) fn with_blocked_by(&self, id: Option<CardInstanceId>) -> Self {
        match id {
            None => Self {
                blocked_by: Vec::new(),
                ..self.clone()
            },
            Some(blocker_id) => {
                let mut new_blocked_by = self.blocked_by.clone();
                new_blocked_by.push(blocker_id);
                Self {
                    blocked_by: new_blocked_by,
                    ..self.clone()
                }
            }
        }
    }

    /// Returns a new `CreatureSubState` with `dealt_first_strike_damage` updated.
    pub(crate) fn with_dealt_first_strike_damage(&self, val: bool) -> Self {
        Self {
            dealt_first_strike_damage: val,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `has_deathtouch_damage` set to `true`.
    pub(crate) fn with_deathtouch_damage(&self) -> Self {
        Self {
            has_deathtouch_damage: true,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with all combat flags reset for a new
    /// turn and summoning sickness cleared.
    pub(crate) fn reset_for_new_turn(&self) -> Self {
        Self {
            is_attacking: false,
            has_attacked_this_turn: false,
            damage_marked_this_turn: 0,
            blocking_creature_id: None,
            blocked_by: Vec::new(),
            has_summoning_sickness: false,
            dealt_first_strike_damage: false,
            has_deathtouch_damage: false,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `damage_marked_this_turn` and
    /// `has_deathtouch_damage` cleared.
    pub(crate) fn clear_damage(&self) -> Self {
        Self {
            damage_marked_this_turn: 0,
            has_deathtouch_damage: false,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with combat state cleared (end of combat).
    pub(crate) fn clear_combat_state(&self) -> Self {
        Self {
            is_attacking: false,
            blocking_creature_id: None,
            blocked_by: Vec::new(),
            dealt_first_strike_damage: false,
            ..self.clone()
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creature_state_has_summoning_sickness() {
        let cs = CreatureSubState::new(2, 3);
        assert!(cs.has_summoning_sickness());
        assert!(!cs.is_attacking());
        assert_eq!(cs.base_power(), 2);
        assert_eq!(cs.base_toughness(), 3);
        assert_eq!(cs.damage_marked_this_turn(), 0);
        assert!(cs.blocking_creature_id().is_none());
        assert!(cs.blocked_by().is_empty());
    }

    #[test]
    fn with_attacking_returns_new_original_unchanged() {
        let cs = CreatureSubState::new(2, 2);
        let attacking = cs.with_attacking(true);
        assert!(attacking.is_attacking());
        assert!(!cs.is_attacking());
    }

    #[test]
    fn with_has_attacked_this_turn_updates_flag() {
        let cs = CreatureSubState::new(1, 1);
        let attacked = cs.with_has_attacked_this_turn(true);
        assert!(attacked.has_attacked_this_turn());
        assert!(!cs.has_attacked_this_turn());
    }

    #[test]
    fn with_summoning_sickness_updates_flag() {
        let cs = CreatureSubState::new(1, 1);
        let no_sick = cs.with_summoning_sickness(false);
        assert!(!no_sick.has_summoning_sickness());
    }

    #[test]
    fn with_damage_updates_damage() {
        let cs = CreatureSubState::new(3, 3);
        let damaged = cs.with_damage(2);
        assert_eq!(damaged.damage_marked_this_turn(), 2);
    }

    #[test]
    fn with_blocking_creature_id_updates_field() {
        let id = CardInstanceId::new("blocker-1");
        let cs = CreatureSubState::new(2, 2);
        let blocking = cs.with_blocking_creature_id(Some(id.clone()));
        assert_eq!(blocking.blocking_creature_id(), Some("blocker-1"));
        assert!(cs.blocking_creature_id().is_none());
    }

    #[test]
    fn with_blocked_by_adds_to_vec() {
        let id = CardInstanceId::new("attacker-1");
        let cs = CreatureSubState::new(2, 2);
        let blocked = cs.with_blocked_by(Some(id.clone()));
        assert_eq!(blocked.blocked_by().len(), 1);
        assert_eq!(blocked.blocked_by()[0].as_str(), "attacker-1");
        assert!(cs.blocked_by().is_empty());
    }

    #[test]
    fn with_blocked_by_multiple_adds_all() {
        let id1 = CardInstanceId::new("blocker-1");
        let id2 = CardInstanceId::new("blocker-2");
        let cs = CreatureSubState::new(2, 2);
        let once = cs.with_blocked_by(Some(id1.clone()));
        let twice = once.with_blocked_by(Some(id2.clone()));
        assert_eq!(twice.blocked_by().len(), 2);
        assert!(twice.is_blocked());
    }

    #[test]
    fn with_blocked_by_none_clears_vec() {
        let id = CardInstanceId::new("attacker-1");
        let cs = CreatureSubState::new(2, 2).with_blocked_by(Some(id));
        let cleared = cs.with_blocked_by(None);
        assert!(cleared.blocked_by().is_empty());
        assert!(!cleared.is_blocked());
    }

    #[test]
    fn reset_for_new_turn_clears_combat_and_sickness() {
        let cs = CreatureSubState::new(2, 2)
            .with_attacking(true)
            .with_damage(1)
            .with_has_attacked_this_turn(true);
        let reset = cs.reset_for_new_turn();
        assert!(!reset.is_attacking());
        assert!(!reset.has_attacked_this_turn());
        assert_eq!(reset.damage_marked_this_turn(), 0);
        assert!(!reset.has_summoning_sickness());
        assert!(reset.blocking_creature_id().is_none());
        assert!(reset.blocked_by().is_empty());
    }

    #[test]
    fn clear_damage_zeroes_damage() {
        let cs = CreatureSubState::new(2, 2).with_damage(5);
        let cleared = cs.clear_damage();
        assert_eq!(cleared.damage_marked_this_turn(), 0);
        assert_eq!(cs.damage_marked_this_turn(), 5);
    }

    #[test]
    fn clear_combat_state_clears_attacking_and_blocking() {
        let id = CardInstanceId::new("attacker-1");
        let cs = CreatureSubState::new(2, 2)
            .with_attacking(true)
            .with_blocking_creature_id(Some(id));
        let cleared = cs.clear_combat_state();
        assert!(!cleared.is_attacking());
        assert!(cleared.blocking_creature_id().is_none());
    }

    #[test]
    fn clear_combat_state_preserves_damage() {
        let cs = CreatureSubState::new(2, 4)
            .with_damage(3)
            .with_attacking(true);
        let cleared = cs.clear_combat_state();
        assert_eq!(cleared.damage_marked_this_turn(), 3);
    }
}
