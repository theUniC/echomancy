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
    pub base_power: i32,
    pub base_toughness: i32,
    pub has_summoning_sickness: bool,
    pub is_attacking: bool,
    pub has_attacked_this_turn: bool,
    pub damage_marked_this_turn: i32,
    pub blocking_creature_id: Option<CardInstanceId>,
    pub blocked_by: Option<CardInstanceId>,
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
            blocked_by: None,
        }
    }

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

    /// Returns a new `CreatureSubState` with `blocked_by` updated.
    pub(crate) fn with_blocked_by(&self, id: Option<CardInstanceId>) -> Self {
        Self {
            blocked_by: id,
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
            blocked_by: None,
            has_summoning_sickness: false,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with `damage_marked_this_turn` cleared.
    pub(crate) fn clear_damage(&self) -> Self {
        Self {
            damage_marked_this_turn: 0,
            ..self.clone()
        }
    }

    /// Returns a new `CreatureSubState` with combat state cleared (end of combat).
    pub(crate) fn clear_combat_state(&self) -> Self {
        Self {
            is_attacking: false,
            blocking_creature_id: None,
            blocked_by: None,
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
        assert!(cs.has_summoning_sickness);
        assert!(!cs.is_attacking);
        assert_eq!(cs.base_power, 2);
        assert_eq!(cs.base_toughness, 3);
        assert_eq!(cs.damage_marked_this_turn, 0);
        assert!(cs.blocking_creature_id.is_none());
        assert!(cs.blocked_by.is_none());
    }

    #[test]
    fn with_attacking_returns_new_original_unchanged() {
        let cs = CreatureSubState::new(2, 2);
        let attacking = cs.with_attacking(true);
        assert!(attacking.is_attacking);
        assert!(!cs.is_attacking);
    }

    #[test]
    fn with_has_attacked_this_turn_updates_flag() {
        let cs = CreatureSubState::new(1, 1);
        let attacked = cs.with_has_attacked_this_turn(true);
        assert!(attacked.has_attacked_this_turn);
        assert!(!cs.has_attacked_this_turn);
    }

    #[test]
    fn with_summoning_sickness_updates_flag() {
        let cs = CreatureSubState::new(1, 1);
        let no_sick = cs.with_summoning_sickness(false);
        assert!(!no_sick.has_summoning_sickness);
    }

    #[test]
    fn with_damage_updates_damage() {
        let cs = CreatureSubState::new(3, 3);
        let damaged = cs.with_damage(2);
        assert_eq!(damaged.damage_marked_this_turn, 2);
    }

    #[test]
    fn with_blocking_creature_id_updates_field() {
        let id = CardInstanceId::new("blocker-1");
        let cs = CreatureSubState::new(2, 2);
        let blocking = cs.with_blocking_creature_id(Some(id.clone()));
        assert_eq!(blocking.blocking_creature_id, Some(id));
        assert!(cs.blocking_creature_id.is_none());
    }

    #[test]
    fn with_blocked_by_updates_field() {
        let id = CardInstanceId::new("attacker-1");
        let cs = CreatureSubState::new(2, 2);
        let blocked = cs.with_blocked_by(Some(id.clone()));
        assert_eq!(blocked.blocked_by, Some(id));
        assert!(cs.blocked_by.is_none());
    }

    #[test]
    fn reset_for_new_turn_clears_combat_and_sickness() {
        let cs = CreatureSubState::new(2, 2)
            .with_attacking(true)
            .with_damage(1)
            .with_has_attacked_this_turn(true);
        let reset = cs.reset_for_new_turn();
        assert!(!reset.is_attacking);
        assert!(!reset.has_attacked_this_turn);
        assert_eq!(reset.damage_marked_this_turn, 0);
        assert!(!reset.has_summoning_sickness);
        assert!(reset.blocking_creature_id.is_none());
        assert!(reset.blocked_by.is_none());
    }

    #[test]
    fn clear_damage_zeroes_damage() {
        let cs = CreatureSubState::new(2, 2).with_damage(5);
        let cleared = cs.clear_damage();
        assert_eq!(cleared.damage_marked_this_turn, 0);
        assert_eq!(cs.damage_marked_this_turn, 5);
    }

    #[test]
    fn clear_combat_state_clears_attacking_and_blocking() {
        let id = CardInstanceId::new("attacker-1");
        let cs = CreatureSubState::new(2, 2)
            .with_attacking(true)
            .with_blocking_creature_id(Some(id));
        let cleared = cs.clear_combat_state();
        assert!(!cleared.is_attacking);
        assert!(cleared.blocking_creature_id.is_none());
    }

    #[test]
    fn clear_combat_state_preserves_damage() {
        let cs = CreatureSubState::new(2, 4)
            .with_damage(3)
            .with_attacking(true);
        let cleared = cs.clear_combat_state();
        assert_eq!(cleared.damage_marked_this_turn, 3);
    }
}
