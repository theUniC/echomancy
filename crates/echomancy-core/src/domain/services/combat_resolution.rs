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
    /// `true` if the source creature has Deathtouch.
    ///
    /// When `true`, any non-zero damage to a creature is lethal regardless of toughness.
    pub is_deathtouch: bool,
    /// `true` if the source creature has Lifelink.
    ///
    /// When `true`, the source creature's controller gains life equal to `amount`.
    pub has_lifelink: bool,
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
    /// Layer-evaluated effective power (overrides `state.current_power()`).
    ///
    /// When `Some`, this value is used for all damage calculations. When `None`,
    /// the code falls back to `state.current_power()` for backward compatibility.
    pub effective_power: Option<i32>,
    /// Layer-evaluated effective toughness (overrides `state.current_toughness()`).
    ///
    /// When `Some`, this value is used for lethal-damage calculations. When `None`,
    /// the code falls back to `state.current_toughness()` for backward compatibility.
    pub effective_toughness: Option<i32>,
    /// `true` if this creature has Trample.
    pub has_trample: bool,
    /// `true` if this creature has Deathtouch.
    pub has_deathtouch: bool,
    /// `true` if this creature has Lifelink.
    pub has_lifelink: bool,
    /// `true` if this creature has Menace (CR 702.110).
    pub has_menace: bool,
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

        let attacker_power = match entry.effective_power.or_else(|| entry.state.current_power().ok()) {
            Some(p) => p,
            None => continue,
        };

        let blocker_ids = cs.blocked_by();

        // CR 702.110: Menace — attacker with fewer than 2 blockers is treated
        // as unblocked (the blocking assignment was illegal).
        let effectively_unblocked =
            blocker_ids.is_empty() || (entry.has_menace && blocker_ids.len() < 2);

        if effectively_unblocked {
            // Unblocked attacker damages the defending player.
            if attacker_power > 0 {
                assignments.push(DamageAssignment {
                    source_id: entry.instance_id.to_owned(),
                    source_controller_id: entry.controller_id.to_owned(),
                    target_id: defending_player_id.to_owned(),
                    amount: attacker_power,
                    is_player: true,
                    is_deathtouch: entry.has_deathtouch,
                    has_lifelink: entry.has_lifelink,
                });
            }
        } else {
            // Resolve attacker damage against all present blockers using auto-assignment.
            // Collect present blockers, sorted by toughness ascending (kill smallest first).
            let mut present_blockers: Vec<&CreatureCombatEntry<'_>> = blocker_ids
                .iter()
                .filter_map(|bid| all_creatures.iter().find(|e| e.instance_id == bid.as_str()))
                .collect();

            if present_blockers.is_empty() {
                // All blockers disappeared (killed by other effects).
                // Trample: full power goes to player. No trample: no damage (CR 510.1c).
                if entry.has_trample && attacker_power > 0 {
                    assignments.push(DamageAssignment {
                        source_id: entry.instance_id.to_owned(),
                        source_controller_id: entry.controller_id.to_owned(),
                        target_id: defending_player_id.to_owned(),
                        amount: attacker_power,
                        is_player: true,
                        is_deathtouch: entry.has_deathtouch,
                        has_lifelink: entry.has_lifelink,
                    });
                }
            } else {
                // Sort blockers by effective toughness ascending so smallest die first.
                present_blockers.sort_by_key(|b| {
                    b.effective_toughness
                        .or_else(|| b.state.current_toughness().ok())
                        .unwrap_or(0)
                });

                // Assign attacker damage to blockers in order.
                //
                // CR 510.1c: assign at least lethal damage to each blocker before assigning
                // to the next. The last blocker absorbs all remaining damage (excess power
                // stays on the blocker, not the player — no trample).
                //
                // With Trample: after all blockers receive lethal, excess hits the player.
                let n = present_blockers.len();
                let mut remaining_power = attacker_power;

                for (i, blocker) in present_blockers.iter().enumerate() {
                    if remaining_power <= 0 {
                        break;
                    }

                    let is_last = i == n - 1;

                    let blocker_toughness = match blocker
                        .effective_toughness
                        .or_else(|| blocker.state.current_toughness().ok())
                    {
                        Some(t) => t,
                        None => continue,
                    };
                    let blocker_damage_already = blocker
                        .state
                        .creature_state()
                        .map(|bcs| bcs.damage_marked_this_turn())
                        .unwrap_or(0);

                    // With Deathtouch: 1 damage is lethal (unless blocker already has dt damage).
                    let lethal_for_blocker = if entry.has_deathtouch {
                        let already_has_dt_damage = blocker
                            .state
                            .creature_state()
                            .map(|bcs| bcs.has_deathtouch_damage())
                            .unwrap_or(false);
                        if already_has_dt_damage { 0 } else { 1 }
                    } else {
                        (blocker_toughness - blocker_damage_already).max(0)
                    };

                    // Without Trample: the last blocker absorbs ALL remaining power (including
                    // excess beyond lethal — CR 510.1c, damage assignment rule).
                    // With Trample: assign exactly lethal to every blocker; excess hits player.
                    let damage_to_blocker = if is_last && !entry.has_trample {
                        remaining_power
                    } else {
                        remaining_power.min(lethal_for_blocker)
                    };
                    remaining_power -= damage_to_blocker;

                    if damage_to_blocker > 0 {
                        assignments.push(DamageAssignment {
                            source_id: entry.instance_id.to_owned(),
                            source_controller_id: entry.controller_id.to_owned(),
                            target_id: blocker.instance_id.to_owned(),
                            amount: damage_to_blocker,
                            is_player: false,
                            is_deathtouch: entry.has_deathtouch,
                            has_lifelink: entry.has_lifelink,
                        });
                    }
                }

                // Trample: excess damage after assigning lethal to all blockers hits player.
                if entry.has_trample && remaining_power > 0 {
                    assignments.push(DamageAssignment {
                        source_id: entry.instance_id.to_owned(),
                        source_controller_id: entry.controller_id.to_owned(),
                        target_id: defending_player_id.to_owned(),
                        amount: remaining_power,
                        is_player: true,
                        is_deathtouch: entry.has_deathtouch,
                        has_lifelink: entry.has_lifelink,
                    });
                }

                // Each blocker deals its power as damage back to the attacker.
                for blocker in &present_blockers {
                    let blocker_power = match blocker
                        .effective_power
                        .or_else(|| blocker.state.current_power().ok())
                    {
                        Some(p) => p,
                        None => continue,
                    };
                    if blocker_power > 0 {
                        assignments.push(DamageAssignment {
                            source_id: blocker.instance_id.to_owned(),
                            source_controller_id: blocker.controller_id.to_owned(),
                            target_id: entry.instance_id.to_owned(),
                            amount: blocker_power,
                            is_player: false,
                            is_deathtouch: blocker.has_deathtouch,
                            has_lifelink: blocker.has_lifelink,
                        });
                    }
                }
            }
        }
    }

    assignments
}

/// Calculates combat damage for the `FirstStrikeDamage` step.
///
/// `first_strikers` — only the creatures with First Strike (both attackers and blockers).
/// `all_creatures`  — the full set of creatures in combat (used for blocker/attacker lookups).
/// `defending_player_id` — the ID of the player being attacked.
///
/// Damage is produced only from creatures in `first_strikers`.
/// A First Strike blocker damages the attacker it is blocking.
/// A First Strike attacker damages its blocker (or the defending player if unblocked).
pub(crate) fn calculate_first_strike_combat_damage(
    first_strikers: &[CreatureCombatEntry<'_>],
    all_creatures: &[CreatureCombatEntry<'_>],
    defending_player_id: &str,
) -> Vec<DamageAssignment> {
    let mut assignments = Vec::new();

    for entry in first_strikers {
        let cs = match entry.state.creature_state() {
            Some(cs) => cs,
            None => continue,
        };

        let power = match entry
            .effective_power
            .or_else(|| entry.state.current_power().ok())
        {
            Some(p) if p > 0 => p,
            _ => continue,
        };

        if cs.is_attacking() {
            // First Strike attacker: damages blocker(s) or defending player.
            let blocker_ids = cs.blocked_by();

            if blocker_ids.is_empty() {
                // Unblocked — deal full power to defending player.
                assignments.push(DamageAssignment {
                    source_id: entry.instance_id.to_owned(),
                    source_controller_id: entry.controller_id.to_owned(),
                    target_id: defending_player_id.to_owned(),
                    amount: power,
                    is_player: true,
                    is_deathtouch: entry.has_deathtouch,
                    has_lifelink: entry.has_lifelink,
                });
            } else {
                // Blocked — assign damage to present blockers sorted by toughness ascending.
                let mut present_blockers: Vec<&CreatureCombatEntry<'_>> = blocker_ids
                    .iter()
                    .filter_map(|bid| all_creatures.iter().find(|e| e.instance_id == bid.as_str()))
                    .collect();

                if present_blockers.is_empty() {
                    // All blockers gone — Trample deals full power to player.
                    if entry.has_trample {
                        assignments.push(DamageAssignment {
                            source_id: entry.instance_id.to_owned(),
                            source_controller_id: entry.controller_id.to_owned(),
                            target_id: defending_player_id.to_owned(),
                            amount: power,
                            is_player: true,
                            is_deathtouch: entry.has_deathtouch,
                            has_lifelink: entry.has_lifelink,
                        });
                    }
                } else {
                    present_blockers.sort_by_key(|b| {
                        b.effective_toughness
                            .or_else(|| b.state.current_toughness().ok())
                            .unwrap_or(0)
                    });

                    let n = present_blockers.len();
                    let mut remaining = power;
                    for (i, blocker) in present_blockers.iter().enumerate() {
                        if remaining <= 0 {
                            break;
                        }
                        let is_last = i == n - 1;
                        let blocker_toughness = match blocker
                            .effective_toughness
                            .or_else(|| blocker.state.current_toughness().ok())
                        {
                            Some(t) => t,
                            None => continue,
                        };
                        let blocker_damage_already = blocker
                            .state
                            .creature_state()
                            .map(|bcs| bcs.damage_marked_this_turn())
                            .unwrap_or(0);
                        let lethal_for_blocker = if entry.has_deathtouch {
                            let already_has_dt_damage = blocker
                                .state
                                .creature_state()
                                .map(|bcs| bcs.has_deathtouch_damage())
                                .unwrap_or(false);
                            if already_has_dt_damage { 0 } else { 1 }
                        } else {
                            (blocker_toughness - blocker_damage_already).max(0)
                        };
                        let damage_to_blocker = if is_last && !entry.has_trample {
                            remaining
                        } else {
                            remaining.min(lethal_for_blocker)
                        };
                        remaining -= damage_to_blocker;

                        if damage_to_blocker > 0 {
                            assignments.push(DamageAssignment {
                                source_id: entry.instance_id.to_owned(),
                                source_controller_id: entry.controller_id.to_owned(),
                                target_id: blocker.instance_id.to_owned(),
                                amount: damage_to_blocker,
                                is_player: false,
                                is_deathtouch: entry.has_deathtouch,
                                has_lifelink: entry.has_lifelink,
                            });
                        }
                    }
                    // Trample excess.
                    if entry.has_trample && remaining > 0 {
                        assignments.push(DamageAssignment {
                            source_id: entry.instance_id.to_owned(),
                            source_controller_id: entry.controller_id.to_owned(),
                            target_id: defending_player_id.to_owned(),
                            amount: remaining,
                            is_player: true,
                            is_deathtouch: entry.has_deathtouch,
                            has_lifelink: entry.has_lifelink,
                        });
                    }
                }
            }
        } else if cs.blocking_creature_id().is_some() {
            // First Strike blocker: damages the attacker it is blocking.
            let attacker_id = cs.blocking_creature_id().unwrap();
            if all_creatures.iter().any(|e| e.instance_id == attacker_id) {
                assignments.push(DamageAssignment {
                    source_id: entry.instance_id.to_owned(),
                    source_controller_id: entry.controller_id.to_owned(),
                    target_id: attacker_id.to_owned(),
                    amount: power,
                    is_player: false,
                    is_deathtouch: entry.has_deathtouch,
                    has_lifelink: entry.has_lifelink,
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

    fn attacker_blocked_by_two(
        power: i32,
        toughness: i32,
        blocker_id1: &str,
        blocker_id2: &str,
    ) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_attacking(true)
            .unwrap()
            .with_summoning_sickness(false)
            .unwrap()
            .with_blocked_by(Some(CardInstanceId::new(blocker_id1)))
            .unwrap()
            .with_blocked_by(Some(CardInstanceId::new(blocker_id2)))
            .unwrap()
    }

    fn blocker(power: i32, toughness: i32, blocking_id: &str) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_summoning_sickness(false)
            .unwrap()
            .with_blocking_creature_id(Some(CardInstanceId::new(blocking_id)))
            .unwrap()
    }

    fn plain_entry<'a>(instance_id: &'a str, controller_id: &'a str, state: &'a PermanentState) -> CreatureCombatEntry<'a> {
        CreatureCombatEntry {
            instance_id,
            controller_id,
            state,
            effective_power: None,
            effective_toughness: None,
            has_trample: false,
            has_deathtouch: false,
            has_lifelink: false,
            has_menace: false,
        }
    }

    fn trample_entry<'a>(instance_id: &'a str, controller_id: &'a str, state: &'a PermanentState) -> CreatureCombatEntry<'a> {
        CreatureCombatEntry {
            instance_id,
            controller_id,
            state,
            effective_power: None,
            effective_toughness: None,
            has_trample: true,
            has_deathtouch: false,
            has_lifelink: false,
            has_menace: false,
        }
    }

    fn deathtouch_entry<'a>(instance_id: &'a str, controller_id: &'a str, state: &'a PermanentState) -> CreatureCombatEntry<'a> {
        CreatureCombatEntry {
            instance_id,
            controller_id,
            state,
            effective_power: None,
            effective_toughness: None,
            has_trample: false,
            has_deathtouch: true,
            has_lifelink: false,
            has_menace: false,
        }
    }

    fn trample_deathtouch_entry<'a>(instance_id: &'a str, controller_id: &'a str, state: &'a PermanentState) -> CreatureCombatEntry<'a> {
        CreatureCombatEntry {
            instance_id,
            controller_id,
            state,
            effective_power: None,
            effective_toughness: None,
            has_trample: true,
            has_deathtouch: true,
            has_lifelink: false,
            has_menace: false,
        }
    }

    fn menace_entry<'a>(instance_id: &'a str, controller_id: &'a str, state: &'a PermanentState) -> CreatureCombatEntry<'a> {
        CreatureCombatEntry {
            instance_id,
            controller_id,
            state,
            effective_power: None,
            effective_toughness: None,
            has_trample: false,
            has_deathtouch: false,
            has_lifelink: false,
            has_menace: true,
        }
    }

    fn lifelink_entry<'a>(instance_id: &'a str, controller_id: &'a str, state: &'a PermanentState) -> CreatureCombatEntry<'a> {
        CreatureCombatEntry {
            instance_id,
            controller_id,
            state,
            effective_power: None,
            effective_toughness: None,
            has_trample: false,
            has_deathtouch: false,
            has_lifelink: true,
            has_menace: false,
        }
    }

    // ---- calculate_all_combat_damage ----------------------------------------

    #[test]
    fn unblocked_attacker_damages_player() {
        let state = attacker(3, 3);
        let entries = [plain_entry("a1", "p1", &state)];
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
            plain_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
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
        // Blocker disappears — attacker deals no damage (no trample)
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        // Only the attacker is supplied; the blocker is absent.
        let entries = [plain_entry("a1", "p1", &attacker_state)];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert!(result.is_empty(), "No damage when blocker is removed (no trample)");
    }

    #[test]
    fn multiple_unblocked_attackers_all_damage_player() {
        let s1 = attacker(2, 2);
        let s2 = attacker(1, 1);

        let entries = [
            plain_entry("a1", "p1", &s1),
            plain_entry("a2", "p1", &s2),
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
        let entries = [plain_entry("c1", "p2", &state)];

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

        let entries = [plain_entry("a1", "p1", &state)];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].amount, 4); // 2 base + 2 counters
    }

    #[test]
    fn blocker_deals_damage_back_to_attacker_bidirectional() {
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        let blocker_state = blocker(2, 5, "a1");

        let entries = [
            plain_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
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
        let attacker_state = attacker_blocked_by(4, 4, "b1");
        let blocker_state = blocker(1, 3, "a1");

        let entries = [
            plain_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
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
        let entries = [plain_entry("a1", "p1", &state)];

        let result = calculate_all_combat_damage(&entries, "p2");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].source_id, "a1");
        assert_eq!(result[0].source_controller_id, "p1");
    }

    // ---- Trample tests -------------------------------------------------------

    #[test]
    fn trample_6_6_blocked_by_1_1_assigns_1_to_blocker_5_to_player() {
        let attacker_state = attacker_blocked_by(6, 6, "b1");
        let blocker_state = blocker(1, 1, "a1");

        let entries = [
            trample_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        let to_blocker = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(to_blocker.amount, 1);
        assert!(!to_blocker.is_player);

        let to_player = result.iter().find(|d| d.target_id == "p2").unwrap();
        assert_eq!(to_player.amount, 5);
        assert!(to_player.is_player);
    }

    #[test]
    fn trample_3_3_blocked_by_3_3_assigns_3_to_blocker_0_to_player() {
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        let blocker_state = blocker(3, 3, "a1");

        let entries = [
            trample_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        // No trample through — all 3 damage goes to blocker
        let to_blocker = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(to_blocker.amount, 3);

        // No damage reaches the player
        assert!(result.iter().all(|d| !d.is_player));
    }

    #[test]
    fn trample_deathtouch_6_6_blocked_by_5_5_assigns_1_to_blocker_5_to_player() {
        // With Deathtouch+Trample: only 1 damage needed for lethal, rest tramples through
        let attacker_state = attacker_blocked_by(6, 6, "b1");
        let blocker_state = blocker(1, 5, "a1");

        let entries = [
            trample_deathtouch_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        let to_blocker = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(to_blocker.amount, 1);
        assert!(to_blocker.is_deathtouch);

        let to_player = result.iter().find(|d| d.target_id == "p2").unwrap();
        assert_eq!(to_player.amount, 5);
    }

    #[test]
    fn trample_with_missing_blocker_full_power_to_player() {
        // Blocker died (e.g., removed), Trample means full power hits player
        let attacker_state = attacker_blocked_by(4, 4, "b1");
        // b1 is absent from entries
        let entries = [trample_entry("a1", "p1", &attacker_state)];

        let result = calculate_all_combat_damage(&entries, "p2");

        assert_eq!(result.len(), 1);
        let to_player = &result[0];
        assert_eq!(to_player.target_id, "p2");
        assert_eq!(to_player.amount, 4);
        assert!(to_player.is_player);
    }

    #[test]
    fn no_trample_blocked_attacker_no_damage_to_player_even_with_excess_power() {
        // 6/6 blocked by 1/1, no trample: all 6 damage goes to blocker, 0 to player
        let attacker_state = attacker_blocked_by(6, 6, "b1");
        let blocker_state = blocker(1, 1, "a1");

        let entries = [
            plain_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        let to_blocker = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(to_blocker.amount, 6);

        assert!(result.iter().all(|d| !d.is_player));
    }

    // ---- Deathtouch flag tests -----------------------------------------------

    #[test]
    fn deathtouch_assignment_sets_is_deathtouch_flag() {
        let attacker_state = attacker_blocked_by(1, 1, "b1");
        let blocker_state = blocker(5, 5, "a1");

        let entries = [
            deathtouch_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        // Attacker → blocker: should have is_deathtouch = true
        let a_to_b = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert!(a_to_b.is_deathtouch, "deathtouch attacker should set is_deathtouch flag");

        // Blocker → attacker: should NOT have is_deathtouch (blocker is plain)
        let b_to_a = result.iter().find(|d| d.target_id == "a1").unwrap();
        assert!(!b_to_a.is_deathtouch);
    }

    // ---- Lifelink flag tests -------------------------------------------------

    #[test]
    fn lifelink_unblocked_sets_has_lifelink_flag() {
        let state = attacker(3, 3);
        let entries = [lifelink_entry("a1", "p1", &state)];

        let result = calculate_all_combat_damage(&entries, "p2");

        assert_eq!(result.len(), 1);
        assert!(result[0].has_lifelink, "lifelink attacker should set has_lifelink flag");
    }

    // ---- Multiple blockers tests ---------------------------------------------

    #[test]
    fn multiple_blockers_damage_auto_assigned_kills_smallest_first() {
        // 5/5 attacker blocked by 1/2 and 2/3 — sorted by toughness ascending: [1/2, 2/3]
        // Attacker has 5 power: assign 2 (lethal for 1/2), then 3 (lethal for 2/3), 0 trample.
        let attacker_state = attacker_blocked_by_two(5, 5, "b1", "b2");
        let blocker1_state = blocker(1, 2, "a1"); // toughness 2
        let blocker2_state = blocker(2, 3, "a1"); // toughness 3

        let entries = [
            plain_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker1_state),
            plain_entry("b2", "p2", &blocker2_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        // b1 should receive lethal (2 damage)
        let to_b1 = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(to_b1.amount, 2, "b1 should receive lethal damage (2)");
        assert!(!to_b1.is_player);

        // b2 should receive remaining 3 damage
        let to_b2 = result.iter().find(|d| d.target_id == "b2").unwrap();
        assert_eq!(to_b2.amount, 3, "b2 should receive remaining damage (3)");
        assert!(!to_b2.is_player);

        // No damage to player (no trample)
        assert!(result.iter().all(|d| !d.is_player));
    }

    #[test]
    fn multiple_blockers_all_deal_damage_to_attacker() {
        // Each blocker deals its power to the attacker
        let attacker_state = attacker_blocked_by_two(3, 5, "b1", "b2");
        let blocker1_state = blocker(1, 1, "a1"); // 1 power
        let blocker2_state = blocker(2, 2, "a1"); // 2 power

        let entries = [
            plain_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker1_state),
            plain_entry("b2", "p2", &blocker2_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        // b1 (1 power) damages attacker
        let b1_to_a = result.iter().find(|d| d.source_id == "b1" && d.target_id == "a1").unwrap();
        assert_eq!(b1_to_a.amount, 1);

        // b2 (2 power) damages attacker
        let b2_to_a = result.iter().find(|d| d.source_id == "b2" && d.target_id == "a1").unwrap();
        assert_eq!(b2_to_a.amount, 2);
    }

    #[test]
    fn trample_with_multiple_blockers_excess_to_player() {
        // 10/10 trampler blocked by 1/1 and 2/2 — sorted by toughness: [1/1, 2/2]
        // assign 1 to b1 (lethal), 2 to b2 (lethal), 7 trample to player
        let attacker_state = attacker_blocked_by_two(10, 10, "b1", "b2");
        let blocker1_state = blocker(1, 1, "a1"); // toughness 1
        let blocker2_state = blocker(2, 2, "a1"); // toughness 2

        let entries = [
            trample_entry("a1", "p1", &attacker_state),
            plain_entry("b1", "p2", &blocker1_state),
            plain_entry("b2", "p2", &blocker2_state),
        ];

        let result = calculate_all_combat_damage(&entries, "p2");

        let to_b1 = result.iter().find(|d| d.target_id == "b1").unwrap();
        assert_eq!(to_b1.amount, 1);

        let to_b2 = result.iter().find(|d| d.target_id == "b2").unwrap();
        assert_eq!(to_b2.amount, 2);

        let to_player = result.iter().find(|d| d.is_player).unwrap();
        assert_eq!(to_player.amount, 7, "7 damage should trample to player");
        assert_eq!(to_player.target_id, "p2");
    }

    // ---- Menace (CR 702.110) ------------------------------------------------

    #[test]
    fn menace_attacker_with_one_blocker_hits_player() {
        // Menace 3/3 blocked by a single 2/2 — treated as unblocked.
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        let blocker_state = blocker(2, 2, "a1");

        let result = calculate_all_combat_damage(
            &[menace_entry("a1", "p1", &attacker_state), plain_entry("b1", "p2", &blocker_state)],
            "p2",
        );

        // Menace with 1 blocker → unblocked → damage goes to player, not blocker.
        let player_damage: Vec<_> = result.iter().filter(|d| d.is_player).collect();
        assert_eq!(player_damage.len(), 1);
        assert_eq!(player_damage[0].amount, 3);
        assert_eq!(player_damage[0].target_id, "p2");
    }

    #[test]
    fn menace_attacker_with_two_blockers_fights_blockers() {
        // Menace 4/4 blocked by two 1/1s — legal block, fights normally.
        let attacker_state = PermanentState::for_creature(4, 4)
            .with_summoning_sickness(false).unwrap()
            .with_attacking(true).unwrap()
            .with_has_attacked_this_turn(true).unwrap()
            .with_blocked_by(Some(CardInstanceId::new("b1"))).unwrap()
            .with_blocked_by(Some(CardInstanceId::new("b2"))).unwrap();
        let blocker1 = blocker(1, 1, "a1");
        let blocker2 = blocker(1, 1, "a1");

        let result = calculate_all_combat_damage(
            &[
                menace_entry("a1", "p1", &attacker_state),
                plain_entry("b1", "p2", &blocker1),
                plain_entry("b2", "p2", &blocker2),
            ],
            "p2",
        );

        // With 2 blockers, menace is satisfied — damage to blockers, not player.
        let player_damage: Vec<_> = result.iter().filter(|d| d.is_player).collect();
        assert!(player_damage.is_empty(), "2 blockers satisfy menace — no player damage");

        let blocker_damage: Vec<_> = result.iter().filter(|d| !d.is_player && d.source_id == "a1").collect();
        assert!(!blocker_damage.is_empty(), "attacker should damage blockers");
    }

    #[test]
    fn non_menace_attacker_with_one_blocker_fights_blocker() {
        // Regular 3/3 blocked by 2/2 — normal combat.
        let attacker_state = attacker_blocked_by(3, 3, "b1");
        let blocker_state = blocker(2, 2, "a1");

        let result = calculate_all_combat_damage(
            &[plain_entry("a1", "p1", &attacker_state), plain_entry("b1", "p2", &blocker_state)],
            "p2",
        );

        let player_damage: Vec<_> = result.iter().filter(|d| d.is_player).collect();
        assert!(player_damage.is_empty(), "blocked non-menace should not hit player");
    }
}
