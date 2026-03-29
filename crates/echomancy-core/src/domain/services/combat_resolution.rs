//! CombatResolution — calculate damage assignments from attackers/blockers.
//!
//! Stateless service that computes what damage is dealt during the
//! COMBAT_DAMAGE step. It does NOT mutate state; the caller applies the
//! resulting assignments.
//!
//! MVP limitations (mirrors `CombatResolution.ts`):
//! - First strike / Double strike not implemented.
//! - Trample not implemented (blocked attacker with removed blocker deals
//!   no damage — no trample through).
//! - Deathtouch not implemented.
//! - Multiple blockers per attacker not implemented.
//! - Damage prevention not implemented.

use crate::domain::value_objects::permanent_state::PermanentState;

/// A single damage assignment produced during the COMBAT_DAMAGE step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DamageAssignment {
    /// The instance ID of the creature dealing the damage.
    pub source_id: String,
    /// The player ID of the creature's controller.
    pub source_controller_id: String,
    /// The target receiving the damage (player ID or card instance ID).
    pub target_id: String,
    /// Amount of damage to deal.
    pub amount: i32,
    /// `true` if the target is a player; `false` if it is a creature.
    pub is_player: bool,
}

/// A creature entry on the battlefield used by `calculate_all_combat_damage`.
///
/// Carries only the data needed for damage calculation so that the service
/// does not depend on a full Game struct.
pub(crate) struct CreatureCombatEntry<'a> {
    /// The instance ID of this creature.
    pub instance_id: &'a str,
    /// The player ID of the controller of this creature.
    pub controller_id: &'a str,
    /// The permanent state for this creature.
    pub state: &'a PermanentState,
}

/// Calculates combat damage for both attacking and blocking creatures.
///
/// Receives _all_ creatures from both battlefields that are involved in combat.
/// It correctly assigns damage in both directions (attacker → blocker/player,
/// blocker → attacker) and populates `source_id` and `source_controller_id`
/// on every assignment.
///
/// For each attacker:
/// - If unblocked → damage the defending player.
/// - If blocked and the blocker is still present → attacker damages the blocker
///   AND the blocker damages the attacker.
/// - If blocked but the blocker is absent → no damage (no trample MVP).
///
/// # Parameters
///
/// - `all_creatures` — all permanents from both battlefields. The function
///   filters internally for those involved in combat.
/// - `defending_player_id` — the ID of the player being attacked.
pub(crate) fn calculate_all_combat_damage(
    all_creatures: &[CreatureCombatEntry<'_>],
    defending_player_id: &str,
) -> Vec<DamageAssignment> {
    let mut assignments = Vec::new();

    for entry in all_creatures {
        let cs = match entry.state.creature_state() {
            Some(cs) if cs.is_attacking() => cs,
            _ => continue,
        };

        let attacker_power = match entry.state.current_power() {
            Ok(p) => p,
            Err(_) => continue,
        };

        match cs.blocked_by() {
            None => {
                // Unblocked attacker damages the defending player.
                assignments.push(DamageAssignment {
                    source_id: entry.instance_id.to_owned(),
                    source_controller_id: entry.controller_id.to_owned(),
                    target_id: defending_player_id.to_owned(),
                    amount: attacker_power,
                    is_player: true,
                });
            }
            Some(blocker_id) => {
                // Find the blocker in the slice to get its power and controller.
                let blocker_entry = all_creatures
                    .iter()
                    .find(|e| e.instance_id == blocker_id);

                let (blocker_power, blocker_controller_id) = match blocker_entry {
                    Some(blocker) => match blocker.state.current_power() {
                        Ok(p) => (p, blocker.controller_id),
                        Err(_) => continue,
                    },
                    // Blocker disappeared (instant/ability removed it) — no damage (MVP: no trample).
                    None => continue,
                };

                // Attacker damages blocker.
                assignments.push(DamageAssignment {
                    source_id: entry.instance_id.to_owned(),
                    source_controller_id: entry.controller_id.to_owned(),
                    target_id: blocker_id.to_owned(),
                    amount: attacker_power,
                    is_player: false,
                });

                // Blocker damages attacker.
                assignments.push(DamageAssignment {
                    source_id: blocker_id.to_owned(),
                    source_controller_id: blocker_controller_id.to_owned(),
                    target_id: entry.instance_id.to_owned(),
                    amount: blocker_power,
                    is_player: false,
                });
            }
        }
    }

    assignments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::CardInstanceId;
    use crate::domain::value_objects::permanent_state::PermanentState;

    fn attacker(power: i32, toughness: i32) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_attacking(true)
            .unwrap()
            .with_summoning_sickness(false)
            .unwrap()
    }

    fn attacker_blocked_by(
        power: i32,
        toughness: i32,
        blocker_id: &str,
    ) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_attacking(true)
            .unwrap()
            .with_summoning_sickness(false)
            .unwrap()
            .with_blocked_by(Some(CardInstanceId::new(blocker_id)))
            .unwrap()
    }

    fn blocker(power: i32, toughness: i32, blocking_id: &str) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_summoning_sickness(false)
            .unwrap()
            .with_blocking_creature_id(Some(CardInstanceId::new(blocking_id)))
            .unwrap()
    }

    // ---- calculate_all_combat_damage ----------------------------------------

    #[test]
    fn unblocked_attacker_damages_player() {
        let state = attacker(3, 3);
        let entries = [CreatureCombatEntry {
            instance_id: "a1",
            controller_id: "p1",
            state: &state,
        }];
        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].target_id, "p2");
        assert_eq!(result[0].amount, 3);
        assert!(result[0].is_player);
    }

    #[test]
    fn blocked_attacker_and_blocker_damage_each_other() {
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        let blocker_state = blocker(2, 2, "a1");

        let entries = [
            CreatureCombatEntry {
                instance_id: "a1",
                controller_id: "p1",
                state: &attacker_state,
            },
            CreatureCombatEntry {
                instance_id: "b1",
                controller_id: "p2",
                state: &blocker_state,
            },
        ];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 2);

        // Attacker (3 power) damages blocker.
        let attacker_dmg = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(attacker_dmg.amount, 3);
        assert!(!attacker_dmg.is_player);

        // Blocker (2 power) damages attacker.
        let blocker_dmg = result.iter().find(|d| d.target_id == "a1").unwrap();
        assert_eq!(blocker_dmg.amount, 2);
        assert!(!blocker_dmg.is_player);
    }

    #[test]
    fn blocker_removed_deals_no_damage_no_trample() {
        // Blocker disappears — attacker deals no damage (MVP: no trample)
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        // Only the attacker is supplied; the blocker is absent.
        let entries = [CreatureCombatEntry {
            instance_id: "a1",
            controller_id: "p1",
            state: &attacker_state,
        }];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert!(result.is_empty(), "No damage when blocker is removed (no trample)");
    }

    #[test]
    fn multiple_unblocked_attackers_all_damage_player() {
        let s1 = attacker(2, 2);
        let s2 = attacker(1, 1);

        let entries = [
            CreatureCombatEntry {
                instance_id: "a1",
                controller_id: "p1",
                state: &s1,
            },
            CreatureCombatEntry {
                instance_id: "a2",
                controller_id: "p1",
                state: &s2,
            },
        ];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 2);
        let total: i32 = result.iter().map(|d| d.amount).sum();
        assert_eq!(total, 3);
        for d in &result {
            assert!(d.is_player);
            assert_eq!(d.target_id, "p2");
        }
    }

    #[test]
    fn non_attacking_creature_skipped() {
        // Creature on battlefield but not attacking.
        let state = PermanentState::for_creature(5, 5)
            .with_summoning_sickness(false)
            .unwrap();
        let entries = [CreatureCombatEntry {
            instance_id: "c1",
            controller_id: "p2",
            state: &state,
        }];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert!(result.is_empty());
    }

    #[test]
    fn attacker_with_plus_counters_deals_boosted_damage() {
        let state = PermanentState::for_creature(2, 2)
            .with_attacking(true)
            .unwrap()
            .with_summoning_sickness(false)
            .unwrap()
            .add_counters("PLUS_ONE_PLUS_ONE", 2);

        let entries = [CreatureCombatEntry {
            instance_id: "a1",
            controller_id: "p1",
            state: &state,
        }];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount, 4); // 2 base + 2 counters
    }

    #[test]
    fn blocker_deals_damage_back_to_attacker_bidirectional() {
        // RED test: verify blocker damages attacker (bidirectional)
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        let blocker_state = blocker(2, 5, "a1");

        let entries = [
            CreatureCombatEntry {
                instance_id: "a1",
                controller_id: "p1",
                state: &attacker_state,
            },
            CreatureCombatEntry {
                instance_id: "b1",
                controller_id: "p2",
                state: &blocker_state,
            },
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        // Blocker (2 power) should damage the attacker
        let blocker_dmg = result
            .iter()
            .find(|d| d.target_id == "a1" && !d.is_player)
            .expect("blocker must deal damage to attacker");
        assert_eq!(blocker_dmg.amount, 2);
        assert_eq!(blocker_dmg.source_id, "b1");
        assert_eq!(blocker_dmg.source_controller_id, "p2");
    }

    #[test]
    fn source_id_and_source_controller_id_populated_correctly() {
        // RED test: verify source fields are set on every assignment
        let attacker_state = attacker_blocked_by(4, 4, "b1");
        let blocker_state = blocker(1, 3, "a1");

        let entries = [
            CreatureCombatEntry {
                instance_id: "a1",
                controller_id: "p1",
                state: &attacker_state,
            },
            CreatureCombatEntry {
                instance_id: "b1",
                controller_id: "p2",
                state: &blocker_state,
            },
        ];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 2);

        // Attacker → blocker assignment
        let a_to_b = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(a_to_b.source_id, "a1");
        assert_eq!(a_to_b.source_controller_id, "p1");

        // Blocker → attacker assignment
        let b_to_a = result.iter().find(|d| d.target_id == "a1").unwrap();
        assert_eq!(b_to_a.source_id, "b1");
        assert_eq!(b_to_a.source_controller_id, "p2");
    }

    #[test]
    fn unblocked_attacker_source_fields_set() {
        let state = attacker(2, 3);
        let entries = [CreatureCombatEntry {
            instance_id: "a1",
            controller_id: "p1",
            state: &state,
        }];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source_id, "a1");
        assert_eq!(result[0].source_controller_id, "p1");
    }
}
