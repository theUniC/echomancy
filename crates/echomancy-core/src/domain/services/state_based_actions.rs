//! StateBasedActions — identify game-state violations that require automatic
//! correction per MTG rules.
//!
//! Stateless service: all functions take slices / references and return lists
//! of IDs. The caller (Game aggregate) applies the actual mutations.
//!
//! Mirrors `StateBasedActions.ts`.
//!
//! MVP coverage:
//! - Creatures with lethal damage (damage >= toughness).
//! - Creatures with 0 or less toughness.
//! - Players who attempted to draw from an empty library.
//! - Players with 0 or less life.

use crate::domain::value_objects::permanent_state::PermanentState;

/// A minimal description of a creature on the battlefield for SBA checks.
pub(crate) struct CreatureSbaEntry<'a> {
    /// The unique instance ID of this creature.
    pub instance_id: &'a str,
    /// The permanent state (must have creature sub-state).
    pub state: &'a PermanentState,
    /// Whether this creature has Indestructible (CR 702.12).
    pub is_indestructible: bool,
}

/// A minimal description of a player for SBA checks.
pub(crate) struct PlayerSbaEntry<'a> {
    /// The unique player ID.
    pub player_id: &'a str,
    /// The player's current life total.
    pub life_total: i32,
    /// Whether this player has attempted to draw from an empty library.
    pub attempted_empty_library_draw: bool,
}

/// Returns the instance IDs of all creatures that should be destroyed due to
/// state-based actions.
///
/// A creature is destroyed if:
/// - Its `damage_marked_this_turn` >= its current toughness (lethal damage), OR
/// - Its current toughness <= 0, OR
/// - It has `has_deathtouch_damage == true` AND `damage_marked_this_turn > 0`
///   (any non-zero damage from a Deathtouch source is lethal — CR 702.2).
///
/// The returned `Vec` may contain duplicates if both conditions are true, but
/// in practice a creature will only appear once (the first matching condition
/// triggers `continue`).
pub(crate) fn find_creatures_to_destroy(
    creatures: &[CreatureSbaEntry<'_>],
) -> Vec<String> {
    let mut to_destroy = Vec::new();

    for entry in creatures {
        let cs = match entry.state.creature_state() {
            Some(cs) => cs,
            None => continue, // Not a creature — skip.
        };

        let current_toughness = match entry.state.current_toughness() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Check 0 or less toughness — this is NOT "destroy", so Indestructible
        // does NOT prevent it (CR 704.5f).
        if current_toughness <= 0 {
            to_destroy.push(entry.instance_id.to_owned());
            continue;
        }

        // CR 702.12: Indestructible permanents can't be destroyed by lethal
        // damage or deathtouch. Skip the destroy checks.
        if entry.is_indestructible {
            continue;
        }

        // Check lethal damage.
        if cs.damage_marked_this_turn() >= current_toughness {
            to_destroy.push(entry.instance_id.to_owned());
            continue;
        }

        // Check Deathtouch: any non-zero damage from a Deathtouch source is lethal (CR 702.2).
        if cs.has_deathtouch_damage() && cs.damage_marked_this_turn() > 0 {
            to_destroy.push(entry.instance_id.to_owned());
        }
    }

    to_destroy
}

/// Returns the IDs of players who should lose because they attempted to draw
/// from an empty library.
///
/// Per MTG rules 121.4 and 704.5b.
pub(crate) fn find_players_who_attempted_empty_library_draw(
    players: &[PlayerSbaEntry<'_>],
) -> Vec<String> {
    players
        .iter()
        .filter(|p| p.attempted_empty_library_draw)
        .map(|p| p.player_id.to_owned())
        .collect()
}

/// Returns the IDs of players who should lose because they have 0 or less
/// life.
///
/// Per MTG rules 704.5a.
pub(crate) fn find_players_with_zero_or_less_life(
    players: &[PlayerSbaEntry<'_>],
) -> Vec<String> {
    players
        .iter()
        .filter(|p| p.life_total <= 0)
        .map(|p| p.player_id.to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::permanent_state::PermanentState;

    // ---- helpers ------------------------------------------------------------

    fn creature_with_damage(power: i32, toughness: i32, damage: i32) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_summoning_sickness(false)
            .unwrap()
            .with_damage(damage)
            .unwrap()
    }

    fn creature_clean(power: i32, toughness: i32) -> PermanentState {
        PermanentState::for_creature(power, toughness)
            .with_summoning_sickness(false)
            .unwrap()
    }

    fn player_entry(id: &str, life: i32, empty_draw: bool) -> PlayerSbaEntry<'_> {
        PlayerSbaEntry {
            player_id: id,
            life_total: life,
            attempted_empty_library_draw: empty_draw,
        }
    }

    // ---- find_creatures_to_destroy: lethal damage --------------------------

    #[test]
    fn creature_with_lethal_damage_is_destroyed() {
        let state = creature_with_damage(2, 2, 2); // 2 damage >= 2 toughness
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"]);
    }

    #[test]
    fn creature_with_more_than_lethal_damage_is_destroyed() {
        let state = creature_with_damage(2, 2, 5);
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"]);
    }

    #[test]
    fn creature_with_one_less_than_lethal_is_not_destroyed() {
        let state = creature_with_damage(2, 3, 2); // 2 damage < 3 toughness
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert!(result.is_empty());
    }

    // ---- find_creatures_to_destroy: zero toughness -------------------------

    #[test]
    fn creature_with_zero_toughness_is_destroyed() {
        // Use a negative toughness base to trigger the 0-toughness check.
        let state = PermanentState::for_creature(1, 0); // 0 toughness
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"]);
    }

    #[test]
    fn creature_with_negative_toughness_is_destroyed() {
        let state = PermanentState::for_creature(1, -1);
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"]);
    }

    #[test]
    fn creature_with_positive_toughness_and_no_damage_is_not_destroyed() {
        let state = creature_clean(2, 3);
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert!(result.is_empty());
    }

    // ---- find_creatures_to_destroy: deathtouch --------------------------------

    #[test]
    fn creature_with_deathtouch_damage_is_destroyed_regardless_of_toughness() {
        // 1 damage on a 5/5, but source had Deathtouch — should still die
        let state = PermanentState::for_creature(5, 5)
            .with_summoning_sickness(false)
            .unwrap()
            .with_damage(1)
            .unwrap()
            .with_deathtouch_damage();
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"]);
    }

    #[test]
    fn creature_with_deathtouch_flag_but_zero_damage_survives() {
        // has_deathtouch_damage true but no actual damage — should NOT die
        let state = PermanentState::for_creature(5, 5)
            .with_summoning_sickness(false)
            .unwrap()
            .with_deathtouch_damage();
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert!(result.is_empty());
    }

    // ---- find_creatures_to_destroy: plus counters boost --------------------

    #[test]
    fn plus_counters_increase_effective_toughness() {
        // 2 toughness + 2 counters = 4 effective; 3 damage is not lethal
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 2)
            .with_damage(3)
            .unwrap();
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert!(result.is_empty());
    }

    #[test]
    fn damage_exactly_equals_boosted_toughness_is_lethal() {
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 1)
            .with_damage(3)
            .unwrap(); // 3 toughness = 3 damage — lethal
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: false,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"]);
    }

    // ---- find_creatures_to_destroy: multiple creatures ---------------------

    #[test]
    fn only_affected_creatures_are_returned() {
        let dying = creature_with_damage(2, 2, 2);
        let healthy = creature_clean(3, 3);

        let entries = [
            CreatureSbaEntry {
                instance_id: "dead",
                state: &dying,
                is_indestructible: false,
            },
            CreatureSbaEntry {
                instance_id: "alive",
                state: &healthy,
                is_indestructible: false,
            },
        ];

        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["dead"]);
    }

    // ---- find_creatures_to_destroy: indestructible (CR 702.12) ------------

    #[test]
    fn indestructible_creature_survives_lethal_damage() {
        let state = creature_with_damage(2, 2, 5); // 5 damage >= 2 toughness
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: true,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert!(result.is_empty(), "Indestructible should survive lethal damage");
    }

    #[test]
    fn indestructible_creature_survives_deathtouch() {
        let state = PermanentState::for_creature(5, 5)
            .with_summoning_sickness(false)
            .unwrap()
            .with_damage(1)
            .unwrap()
            .with_deathtouch_damage();
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: true,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert!(result.is_empty(), "Indestructible should survive deathtouch");
    }

    #[test]
    fn indestructible_creature_still_dies_to_zero_toughness() {
        // CR 704.5f: toughness <= 0 is NOT destroy — Indestructible doesn't help.
        let state = PermanentState::for_creature(1, 0);
        let entries = [CreatureSbaEntry {
            instance_id: "c1",
            state: &state,
            is_indestructible: true,
        }];
        let result = find_creatures_to_destroy(&entries);
        assert_eq!(result, vec!["c1"], "zero toughness kills even Indestructible");
    }

    // ---- find_players_who_attempted_empty_library_draw ---------------------

    #[test]
    fn player_with_empty_draw_flag_returned() {
        let players = [player_entry("p1", 20, true)];
        let result = find_players_who_attempted_empty_library_draw(&players);
        assert_eq!(result, vec!["p1"]);
    }

    #[test]
    fn player_without_empty_draw_flag_not_returned() {
        let players = [player_entry("p1", 20, false)];
        let result = find_players_who_attempted_empty_library_draw(&players);
        assert!(result.is_empty());
    }

    #[test]
    fn only_affected_players_returned_for_empty_draw() {
        let players = [
            player_entry("p1", 20, true),
            player_entry("p2", 15, false),
        ];
        let result = find_players_who_attempted_empty_library_draw(&players);
        assert_eq!(result, vec!["p1"]);
    }

    // ---- find_players_with_zero_or_less_life ------------------------------

    #[test]
    fn player_with_zero_life_returned() {
        let players = [player_entry("p1", 0, false)];
        let result = find_players_with_zero_or_less_life(&players);
        assert_eq!(result, vec!["p1"]);
    }

    #[test]
    fn player_with_negative_life_returned() {
        let players = [player_entry("p1", -3, false)];
        let result = find_players_with_zero_or_less_life(&players);
        assert_eq!(result, vec!["p1"]);
    }

    #[test]
    fn player_with_positive_life_not_returned() {
        let players = [player_entry("p1", 1, false)];
        let result = find_players_with_zero_or_less_life(&players);
        assert!(result.is_empty());
    }

    #[test]
    fn both_life_loss_conditions_in_same_call() {
        let players = [
            player_entry("p1", 0, false),
            player_entry("p2", 20, false),
            player_entry("p3", -5, false),
        ];
        let result = find_players_with_zero_or_less_life(&players);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"p1".to_owned()));
        assert!(result.contains(&"p3".to_owned()));
    }
}
