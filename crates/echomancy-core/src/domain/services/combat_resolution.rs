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
    /// The target receiving the damage (player ID or card instance ID).
    pub target_id: String,
    /// Amount of damage to deal.
    pub amount: i32,
    /// `true` if the target is a player; `false` if it is a creature.
    pub is_player: bool,
}

/// A creature entry on the battlefield used by `calculate_damage_assignments`.
///
/// Carries only the data needed for damage calculation so that the service
/// does not depend on a full Game struct.
pub(crate) struct CreatureCombatEntry<'a> {
    /// The instance ID of this creature.
    pub instance_id: &'a str,
    /// The permanent state for this creature.
    pub state: &'a PermanentState,
}

/// Calculates all combat damage assignments for the current combat.
///
/// # Parameters
///
/// - `attackers` — slice of all attacking creatures (from the active player's
///   battlefield). Only entries whose `PermanentState` has `is_attacking == true`
///   are considered.
/// - `defending_player_id` — the ID of the player being attacked.
///
/// # Returns
///
/// A `Vec<DamageAssignment>` ready to be applied by the caller.
/// Assignments are produced in the order attackers are supplied.
pub(crate) fn calculate_damage_assignments(
    attackers: &[CreatureCombatEntry<'_>],
    defending_player_id: &str,
) -> Vec<DamageAssignment> {
    let mut assignments = Vec::new();

    for entry in attackers {
        let cs = match entry.state.creature_state() {
            Some(cs) if cs.is_attacking() => cs,
            _ => continue, // Not a creature or not attacking — skip.
        };

        // Power is the damage this creature deals.
        let power = match entry.state.current_power() {
            Ok(p) => p,
            Err(_) => continue, // Not a creature — skip (should not happen).
        };

        match cs.blocked_by() {
            None => {
                // Unblocked: damage goes to the defending player.
                assignments.push(DamageAssignment {
                    target_id: defending_player_id.to_owned(),
                    amount: power,
                    is_player: true,
                });
            }
            Some(blocker_id) => {
                // Blocked: attacker and blocker deal damage to each other
                // simultaneously. The blocker's entry must also be in the
                // caller's data; we look it up from the provided attackers
                // slice (blocker is identified by its instance ID only —
                // the Game aggregate will supply blocker entries too if needed).
                //
                // Since this service receives only the attacker side, we emit
                // the attacker-→blocker damage here. The blocker-→attacker
                // direction is emitted separately when the blocker entry is
                // processed.
                //
                // MVP: blockers are supplied in the same slice as attackers
                // when the caller flattens all battlefield creatures.
                assignments.push(DamageAssignment {
                    target_id: blocker_id.to_owned(),
                    amount: power,
                    is_player: false,
                });
            }
        }
    }

    assignments
}

/// Calculates combat damage for both attacking and blocking creatures.
///
/// This is the preferred entry-point: it receives _all_ creatures from both
/// battlefields that are involved in combat (attacker side first, blocker side
/// may overlap). It correctly assigns damage in both directions.
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
#[allow(dead_code)]
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
                    target_id: defending_player_id.to_owned(),
                    amount: attacker_power,
                    is_player: true,
                });
            }
            Some(blocker_id) => {
                // Find the blocker in the slice to get its power.
                let blocker_entry = all_creatures
                    .iter()
                    .find(|e| e.instance_id == blocker_id);

                let blocker_power = match blocker_entry {
                    Some(blocker) => match blocker.state.current_power() {
                        Ok(p) => p,
                        Err(_) => continue,
                    },
                    // Blocker disappeared (instant/ability removed it) — no damage (MVP: no trample).
                    None => continue,
                };

                // Attacker damages blocker.
                assignments.push(DamageAssignment {
                    target_id: blocker_id.to_owned(),
                    amount: attacker_power,
                    is_player: false,
                });

                // Blocker damages attacker.
                assignments.push(DamageAssignment {
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

    fn attacker(_instance_id: &str, power: i32, toughness: i32) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_attacking(true)
            .unwrap()
            .with_summoning_sickness(false)
            .unwrap()
    }

    fn attacker_blocked_by(
        _instance_id: &str,
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
        let state = attacker("a1", 3, 3);
        let entries = [CreatureCombatEntry {
            instance_id: "a1",
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
        let attacker_state = attacker_blocked_by("a1", 3, 3, "b1");
        let blocker_state = blocker(2, 2, "a1");

        let entries = [
            CreatureCombatEntry {
                instance_id: "a1",
                state: &attacker_state,
            },
            CreatureCombatEntry {
                instance_id: "b1",
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
        let attacker_state = attacker_blocked_by("a1", 3, 3, "b1");
        // Only the attacker is supplied; the blocker is absent.
        let entries = [CreatureCombatEntry {
            instance_id: "a1",
            state: &attacker_state,
        }];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert!(result.is_empty(), "No damage when blocker is removed (no trample)");
    }

    #[test]
    fn multiple_unblocked_attackers_all_damage_player() {
        let s1 = attacker("a1", 2, 2);
        let s2 = attacker("a2", 1, 1);

        let entries = [
            CreatureCombatEntry {
                instance_id: "a1",
                state: &s1,
            },
            CreatureCombatEntry {
                instance_id: "a2",
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
            state: &state,
        }];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount, 4); // 2 base + 2 counters
    }
}
