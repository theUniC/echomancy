//! Combat damage resolution for the `Game` aggregate.
//!
//! This module contains:
//! - `CombatCreatureData` — intermediate struct for collecting combat participants.
//! - `collect_combat_creatures` — gather all attacking/blocking creatures with their
//!   layer-evaluated characteristics.
//! - `resolve_first_strike_damage` — FirstStrikeDamage step logic (CR 510.1).
//! - `resolve_regular_combat_damage` — CombatDamage step logic (CR 510.2).
//! - Private helpers `creature_has_first_strike` and `creature_has_double_strike`.

use crate::domain::enums::StaticAbility;
use crate::domain::services::combat_resolution::{
    calculate_all_combat_damage, calculate_first_strike_combat_damage, CreatureCombatEntry,
};
use crate::domain::value_objects::permanent_state::PermanentState;

use super::Game;

/// Data for a single creature in combat, collected by `collect_combat_creatures`.
pub(super) struct CombatCreatureData {
    pub(super) instance_id: String,
    pub(super) controller_id: String,
    pub(super) state: PermanentState,
    /// Layer-evaluated power, or `None` if not a creature.
    pub(super) effective_power: Option<i32>,
    /// Layer-evaluated toughness, or `None` if not a creature.
    pub(super) effective_toughness: Option<i32>,
    pub(super) has_trample: bool,
    pub(super) has_deathtouch: bool,
    pub(super) has_lifelink: bool,
    pub(super) has_menace: bool,
}

impl Game {
    /// Collect all creatures currently in combat (attacking or blocking).
    ///
    /// `effective_power` and `effective_toughness` are computed via the layer pipeline so
    /// that Layer 7b (set P/T) and Layer 7c (modify P/T) effects are applied to combat.
    pub(super) fn collect_combat_creatures(&self) -> Vec<CombatCreatureData> {
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
                // Compute layer-aware effective characteristics for this combat creature.
                let (effective_power, effective_toughness) = self
                    .effective_characteristics(&id)
                    .map(|ch| (ch.power, ch.toughness))
                    .unwrap_or((None, None));

                // Read keyword abilities from the layer pipeline to account for
                // Layer 6 effects (e.g. Trample granted by a spell).
                let effective_abilities = self
                    .effective_abilities(&id)
                    .unwrap_or_default();

                result.push(CombatCreatureData {
                    instance_id: id,
                    controller_id: controller.clone(),
                    state,
                    effective_power,
                    effective_toughness,
                    has_trample: effective_abilities.contains(&StaticAbility::Trample),
                    has_deathtouch: effective_abilities.contains(&StaticAbility::Deathtouch),
                    has_lifelink: effective_abilities.contains(&StaticAbility::Lifelink),
                    has_menace: effective_abilities.contains(&StaticAbility::Menace),
                });
            }
        }

        result
    }

    /// Returns `true` if the creature with the given instance_id has `FirstStrike`.
    ///
    /// Consults the layer pipeline first (CR 613.1f — Layer 6 can add/remove abilities)
    /// so that effects like "Turn to Frog" (RemoveAllAbilities) are respected.
    pub(super) fn creature_has_first_strike(&self, instance_id: &str) -> bool {
        self.effective_abilities(instance_id)
            .map(|a| {
                a.contains(&StaticAbility::FirstStrike) || a.contains(&StaticAbility::DoubleStrike)
            })
            .unwrap_or(false)
    }

    pub(super) fn creature_has_double_strike(&self, instance_id: &str) -> bool {
        self.effective_abilities(instance_id)
            .map(|a| a.contains(&StaticAbility::DoubleStrike))
            .unwrap_or(false)
    }

    /// Resolve damage for the `FirstStrikeDamage` step.
    ///
    /// Only creatures with `FirstStrike` deal damage in this step.
    /// Each creature that deals damage gets `dealt_first_strike_damage = true`.
    pub(super) fn resolve_first_strike_damage(&mut self) {
        let active_player = self.turn_state.current_player_id().as_str().to_owned();
        let defending_player_id = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() != active_player)
            .map(|p| p.player_id.as_str().to_owned())
            .unwrap_or_default();

        let all_combat = self.collect_combat_creatures();

        // Determine which combat creatures have FirstStrike.
        let first_strikers: Vec<&CombatCreatureData> = all_combat
            .iter()
            .filter(|c| self.creature_has_first_strike(&c.instance_id))
            .collect();

        // If no first strikers, nothing to do.
        if first_strikers.is_empty() {
            return;
        }

        // Collect IDs that have first strike (both attackers and blockers).
        let first_striker_ids: Vec<String> = first_strikers
            .iter()
            .map(|c| c.instance_id.clone())
            .collect();

        // Build snapshots for the full combat pool (needed for blocker lookups) and for
        // first strikers only (the damage sources in this step).
        let all_entries: Vec<CreatureCombatEntry<'_>> = all_combat
            .iter()
            .map(|c| CreatureCombatEntry {
                instance_id: c.instance_id.as_str(),
                controller_id: c.controller_id.as_str(),
                state: &c.state,
                effective_power: c.effective_power,
                effective_toughness: c.effective_toughness,
                has_trample: c.has_trample,
                has_deathtouch: c.has_deathtouch,
                has_lifelink: c.has_lifelink,
                has_menace: c.has_menace,
            })
            .collect();

        let fs_entries: Vec<CreatureCombatEntry<'_>> = first_strikers
            .iter()
            .map(|c| CreatureCombatEntry {
                instance_id: c.instance_id.as_str(),
                controller_id: c.controller_id.as_str(),
                state: &c.state,
                effective_power: c.effective_power,
                effective_toughness: c.effective_toughness,
                has_trample: c.has_trample,
                has_deathtouch: c.has_deathtouch,
                has_lifelink: c.has_lifelink,
                has_menace: c.has_menace,
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

        // Apply damage; also handle Lifelink life gain and Toxic poison counters.
        for assignment in &assignments {
            if assignment.is_player {
                self.deal_damage_to_player(&assignment.target_id, assignment.amount);
                // CR 702.164: Toxic — if source deals combat damage to a player,
                // that player gets N poison counters (where N = toxic value on the card).
                if assignment.amount > 0 {
                    let toxic_n = self.players.iter()
                        .flat_map(|p| p.battlefield.iter())
                        .find(|c| c.instance_id() == assignment.source_id)
                        .map(|c| c.definition().toxic())
                        .unwrap_or(0);
                    if toxic_n > 0 {
                        let _ = self.add_poison_counters(&assignment.target_id, toxic_n);
                    }
                }
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
    }

    /// Resolve damage for the regular `CombatDamage` step.
    ///
    /// Only creatures that have NOT already dealt first strike damage participate.
    pub(super) fn resolve_regular_combat_damage(&mut self) {
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
        let regular_combat: Vec<CombatCreatureData> = all_combat
            .into_iter()
            .filter(|c| {
                let dealt_fs = c.state
                    .creature_state()
                    .map(|cs| cs.dealt_first_strike_damage())
                    .unwrap_or(false);
                if !dealt_fs {
                    return true; // Didn't deal first strike damage → participates
                }
                // Dealt first strike damage but has Double Strike → still participates
                self.creature_has_double_strike(&c.instance_id)
            })
            .collect();

        let combat_entries: Vec<CreatureCombatEntry<'_>> = regular_combat
            .iter()
            .map(|c| CreatureCombatEntry {
                instance_id: c.instance_id.as_str(),
                controller_id: c.controller_id.as_str(),
                state: &c.state,
                effective_power: c.effective_power,
                effective_toughness: c.effective_toughness,
                has_trample: c.has_trample,
                has_deathtouch: c.has_deathtouch,
                has_lifelink: c.has_lifelink,
                has_menace: c.has_menace,
            })
            .collect();

        let assignments = calculate_all_combat_damage(&combat_entries, &defending_player_id);

        // Apply all damage; also handle Lifelink life gain and Toxic poison counters.
        for assignment in &assignments {
            if assignment.is_player {
                self.deal_damage_to_player(&assignment.target_id, assignment.amount);
                // CR 702.164: Toxic — if source deals combat damage to a player,
                // that player gets N poison counters (where N = toxic value on the card).
                if assignment.amount > 0 {
                    let toxic_n = self.players.iter()
                        .flat_map(|p| p.battlefield.iter())
                        .find(|c| c.instance_id() == assignment.source_id)
                        .map(|c| c.definition().toxic())
                        .unwrap_or(0);
                    if toxic_n > 0 {
                        let _ = self.add_poison_counters(&assignment.target_id, toxic_n);
                    }
                }
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

        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        // Advance to DeclareBlockers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        // Advance to FirstStrikeDamage
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        // Advance to CombatDamage
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

    // =========================================================================
    // Toxic tests (K11.14 — CR 702.164)
    // =========================================================================

    #[test]
    fn toxic_creature_gives_poison_counters_on_combat_damage_to_player() {
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

        // 1/1 creature with Toxic 2
        let def = CardDefinition::new("phyrexian", "Phyrexian", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_toxic(2);
        let attacker_card = CardInstance::new("toxic-attacker", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "toxic-attacker");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("toxic-attacker"),
        })
        .unwrap();

        // Advance through DeclareBlockers → FirstStrikeDamage → CombatDamage
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        // Defending player should have 1 damage + 2 poison counters
        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            19,
            "Defending player should take 1 damage"
        );
        assert_eq!(
            game.player_poison_counters(&p2).unwrap(),
            2,
            "Toxic 2 should give 2 poison counters"
        );
    }

    #[test]
    fn non_toxic_creature_does_not_give_poison_counters() {
        let (mut game, p1, p2) = make_started_game();

        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        let attacker_card = make_creature_card("normal-attacker", &p1, 3, 3);
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "normal-attacker");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("normal-attacker"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        assert_eq!(
            game.player_poison_counters(&p2).unwrap(),
            0,
            "Non-toxic creature should not give poison counters"
        );
    }

    #[test]
    fn toxic_blocked_creature_does_not_give_poison_counters() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, p2) = make_started_game();

        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        // Toxic 2 attacker
        let def = CardDefinition::new("phyrexian", "Phyrexian", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_toxic(2);
        let attacker_card = CardInstance::new("toxic-attacker", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, attacker_card);
        clear_summoning_sickness(&mut game, "toxic-attacker");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("toxic-attacker"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();

        // Block the attacker
        let blocker_card = make_creature_card("blocker-1", &p2, 2, 5);
        add_permanent_to_battlefield(&mut game, &p2, blocker_card);
        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("toxic-attacker"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::CombatDamage);

        // Blocked — no player damage, no poison counters
        assert_eq!(
            game.player_life_total(&p2).unwrap(),
            20,
            "Blocked attacker should not deal damage to defending player"
        );
        assert_eq!(
            game.player_poison_counters(&p2).unwrap(),
            0,
            "Blocked Toxic creature should not give poison counters"
        );
    }

    // =========================================================================
    // Layer-system bypass tests (LS1 fixes)
    // =========================================================================

    /// A creature with FirstStrike that has all abilities removed via the layer
    /// system BEFORE the FirstStrikeDamage step should NOT deal damage in that step.
    #[test]
    fn first_strike_removed_by_layer_system_does_not_deal_first_strike_damage() {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::game::test_helpers::make_creature_with_ability;
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, p2) = make_started_game();

        // Advance to DeclareAttackers (5 steps from Untap).
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        // 2/2 attacker with FirstStrike on card definition.
        let attacker = make_creature_with_ability("attacker-1", &p1, 2, 2, StaticAbility::FirstStrike);
        add_permanent_to_battlefield(&mut game, &p1, attacker);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance to DeclareBlockers.
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        // Add a 2/3 blocker (survives 2 damage if first strike doesn't fire).
        let blocker = make_creature_card("blocker-1", &p2, 2, 3);
        add_permanent_to_battlefield(&mut game, &p2, blocker);
        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // BEFORE advancing to FirstStrikeDamage: apply RemoveAllAbilities to the attacker.
        let remove_abilities = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::RemoveAllAbilities,
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 200,
            source_id: "turn-to-frog".to_owned(),
            controller_id: p2.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["attacker-1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(remove_abilities);

        // Verify the layer system no longer reports FirstStrike.
        let abilities = game.effective_abilities("attacker-1").expect("should find creature");
        assert!(
            !abilities.contains(&StaticAbility::FirstStrike),
            "effective_abilities must not include FirstStrike after RemoveAllAbilities"
        );

        // Advance into FirstStrikeDamage — this triggers resolve_first_strike_damage().
        game.apply(Action::AdvanceStep { player_id: PlayerId::new(&p1) }).unwrap();
        assert_eq!(game.current_step(), Step::FirstStrikeDamage);

        // The attacker no longer has FirstStrike, so it must NOT have dealt first-strike damage.
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        assert!(
            !attacker_state
                .creature_state()
                .unwrap()
                .dealt_first_strike_damage(),
            "Attacker with FirstStrike removed by layer system must not have dealt first-strike damage"
        );

        // The blocker should have 0 damage (first-strike step did not fire).
        let blocker_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(
            blocker_state.creature_state().unwrap().damage_marked_this_turn(),
            0,
            "Blocker should take no first-strike damage when attacker's FirstStrike was removed"
        );
    }
}
