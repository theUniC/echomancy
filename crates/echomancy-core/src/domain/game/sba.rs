//! State-based actions (SBA) for the `Game` aggregate.
//!
//! Implements `perform_state_based_actions` which checks game state and
//! applies automatic game rules (creature death, player loss, etc.).

use std::collections::HashSet;

use crate::domain::enums::GraveyardReason;
use crate::domain::events::GameEvent;
use crate::domain::services::state_based_actions::{
    CreatureSbaEntry, PlayerSbaEntry, find_creatures_to_destroy,
    find_players_who_attempted_empty_library_draw, find_players_with_zero_or_less_life,
};
use crate::domain::value_objects::permanent_state::PermanentState;

use super::{Game, GameEndReason, GameOutcome};

impl Game {
    /// Perform state-based actions (SBA).
    ///
    /// Per CR 704.3, SBAs are checked repeatedly in a loop until no more apply.
    /// Destroys creatures with lethal damage or zero toughness, and ends the
    /// game if a player has lost their win condition.
    pub(crate) fn perform_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // CR 704.3: loop until no state-based actions are performed.
        // CR 104.4b: if the loop never stabilizes, the game is a draw.
        const MAX_SBA_ITERATIONS: usize = 20;
        let mut stabilized = false;
        for _ in 0..MAX_SBA_ITERATIONS {
            let mut any_action = false;

            // 1. Destroy creatures with lethal damage or zero toughness
            //    Build a lookup of which permanents are indestructible.
            use crate::domain::enums::StaticAbility;
            let indestructible_ids: std::collections::HashSet<String> = self
                .players
                .iter()
                .flat_map(|p| p.battlefield.iter())
                .filter(|c| {
                    self.effective_abilities(c.instance_id())
                        .map(|a| a.contains(&StaticAbility::Indestructible))
                        .unwrap_or(false)
                })
                .map(|c| c.instance_id().to_owned())
                .collect();

            let creature_entries: Vec<(String, PermanentState)> = self
                .permanent_states
                .iter()
                .filter(|(_, s)| s.creature_state().is_some())
                .map(|(id, s)| (id.clone(), s.clone()))
                .collect();

            let sba_entries: Vec<CreatureSbaEntry<'_>> = creature_entries
                .iter()
                .map(|(id, s)| {
                    // Use layer-aware effective toughness so Layer 7 effects (e.g.
                    // "becomes 1/1") are respected by SBA checks (LS1).
                    let effective_toughness = self.effective_toughness(id.as_str());
                    CreatureSbaEntry {
                        instance_id: id.as_str(),
                        state: s,
                        is_indestructible: indestructible_ids.contains(id),
                        effective_toughness,
                    }
                })
                .collect();

            let to_destroy = find_creatures_to_destroy(&sba_entries);
            for creature_id in &to_destroy {
                if let Ok(evts) =
                    self.move_permanent_to_graveyard(creature_id, GraveyardReason::StateBased)
                {
                    events.extend(evts);
                    any_action = true;
                }
            }

            // 2. Legend rule (CR 704.5j): if a player controls two or more
            //    legendary permanents with the same name, keep one and sacrifice the rest.
            let legend_victims = self.find_legend_rule_victims();
            for victim_id in &legend_victims {
                if let Ok(evts) =
                    self.move_permanent_to_graveyard(victim_id, GraveyardReason::StateBased)
                {
                    events.extend(evts);
                    any_action = true;
                }
            }

            // 3. CR 704.5q: Counter annihilation — if a permanent has both
            //    +1/+1 and -1/-1 counters, remove one of each for each pair.
            let counter_annihilation_ids: Vec<String> = self
                .permanent_states
                .iter()
                .filter(|(_, s)| {
                    s.get_counters("PLUS_ONE_PLUS_ONE") > 0
                        && s.get_counters("MINUS_ONE_MINUS_ONE") > 0
                })
                .map(|(id, _)| id.clone())
                .collect();

            for perm_id in &counter_annihilation_ids {
                if let Some(state) = self.permanent_states.get(perm_id).cloned() {
                    let plus = state.get_counters("PLUS_ONE_PLUS_ONE");
                    let minus = state.get_counters("MINUS_ONE_MINUS_ONE");
                    let pairs = plus.min(minus);
                    let new_state = state
                        .remove_counters("PLUS_ONE_PLUS_ONE", pairs)
                        .remove_counters("MINUS_ONE_MINUS_ONE", pairs);
                    self.permanent_states.insert(perm_id.clone(), new_state);
                    any_action = true;
                }
            }

            if !any_action {
                stabilized = true;
                break;
            }
        }

        // CR 104.4b: SBA loop never stabilized — declare draw.
        if !stabilized {
            self.outcome = Some(GameOutcome::Draw {
                reason: GameEndReason::InfiniteLoop,
            });
            self.lifecycle = crate::domain::enums::GameLifecycleState::Finished;
            return events;
        }

        // 2. Check player loss conditions (outside the loop — game-ending)
        // CR 704.5c: player with 10 or more poison counters loses.
        let losers_by_poison: Vec<String> = self
            .players
            .iter()
            .filter(|p| p.poison_counters >= 10)
            .map(|p| p.player_id.as_str().to_owned())
            .collect();

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
            .chain(losers_by_poison.iter().map(String::as_str))
            .collect();

        if !all_losers.is_empty() {
            let reason = if !losers_by_poison.is_empty() {
                GameEndReason::PoisonCounters
            } else if !losers_by_life.is_empty() && !losers_by_library.is_empty() {
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

    /// CR 704.5j: Find legendary permanents that violate the legend rule.
    ///
    /// For each player, if they control two or more legendary permanents with
    /// the same name, all but the first (oldest) are returned for removal.
    fn find_legend_rule_victims(&self) -> Vec<String> {
        use std::collections::HashMap;

        let mut victims = Vec::new();

        for player in &self.players {
            // Group legendary permanents by name
            let mut seen: HashMap<&str, &str> = HashMap::new(); // name → first instance_id
            for card in &player.battlefield {
                if card.definition().is_legendary() {
                    let name = card.definition().name();
                    if seen.contains_key(name) {
                        // Duplicate — this one goes to graveyard
                        victims.push(card.instance_id().to_owned());
                    } else {
                        seen.insert(name, card.instance_id());
                    }
                }
            }
        }

        victims
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::CardType;
    use crate::domain::game::test_helpers::{add_permanent_to_battlefield, make_game_in_first_main};

    // ---- Poison counters (CR 704.5c) ---------------------------------------

    #[test]
    fn player_with_ten_poison_counters_loses() {
        let (mut game, p1, p2) = make_game_in_first_main();

        game.add_poison_counters(&p1, 10).expect("should add poison counters");
        game.perform_state_based_actions();

        let outcome = game.outcome();
        assert!(outcome.is_some(), "game should be over");
        use crate::domain::game::{GameEndReason, GameOutcome};
        match outcome.unwrap() {
            GameOutcome::Win { winner_id, reason } => {
                assert_eq!(winner_id.as_str(), p2, "opponent should win");
                assert_eq!(*reason, GameEndReason::PoisonCounters);
            }
            _ => panic!("expected Win outcome"),
        }
    }

    #[test]
    fn player_with_nine_poison_counters_survives() {
        let (mut game, p1, _p2) = make_game_in_first_main();

        game.add_poison_counters(&p1, 9).expect("should add 9 poison counters");
        game.perform_state_based_actions();

        assert!(game.outcome().is_none(), "9 poison counters should not end the game");
    }

    // ---- Counter annihilation (CR 704.5q) ----------------------------------

    #[test]
    fn counter_annihilation_removes_pairs_of_plus_and_minus() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let card = CardInstance::new("bear-counters", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        // Add 3 +1/+1 counters and 2 -1/-1 counters
        {
            let state = game.permanent_state("bear-counters").unwrap().clone();
            let new_state = state
                .add_counters("PLUS_ONE_PLUS_ONE", 3)
                .add_counters("MINUS_ONE_MINUS_ONE", 2);
            game.set_permanent_state("bear-counters", new_state);
        }

        game.perform_state_based_actions();

        // Should remove 2 pairs — 1 +1/+1 counter remains, 0 -1/-1 counters
        let state = game.permanent_state("bear-counters").unwrap();
        assert_eq!(state.get_counters("PLUS_ONE_PLUS_ONE"), 1);
        assert_eq!(state.get_counters("MINUS_ONE_MINUS_ONE"), 0);
    }

    #[test]
    fn counter_annihilation_removes_all_when_equal_counts() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let card = CardInstance::new("bear-eq", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        // Add 2 +1/+1 counters and 2 -1/-1 counters
        {
            let state = game.permanent_state("bear-eq").unwrap().clone();
            let new_state = state
                .add_counters("PLUS_ONE_PLUS_ONE", 2)
                .add_counters("MINUS_ONE_MINUS_ONE", 2);
            game.set_permanent_state("bear-eq", new_state);
        }

        game.perform_state_based_actions();

        // All pairs removed — both counters at 0
        let state = game.permanent_state("bear-eq").unwrap();
        assert_eq!(state.get_counters("PLUS_ONE_PLUS_ONE"), 0);
        assert_eq!(state.get_counters("MINUS_ONE_MINUS_ONE"), 0);
    }

    #[test]
    fn legend_rule_removes_duplicate_legendary_permanent() {
        let (mut game, p1, _p2) = make_game_in_first_main();

        let legend_def = CardDefinition::new("thalia", "Thalia", vec![CardType::Creature])
            .with_legendary()
            .with_subtype("Human")
            .with_power_toughness(2, 1);

        let first = CardInstance::new("thalia-1", legend_def.clone(), &p1);
        let second = CardInstance::new("thalia-2", legend_def, &p1);

        add_permanent_to_battlefield(&mut game, &p1, first);
        add_permanent_to_battlefield(&mut game, &p1, second);
        assert_eq!(game.battlefield(&p1).unwrap().len(), 2);

        game.perform_state_based_actions();

        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            1,
            "legend rule should remove duplicate"
        );
        assert_eq!(
            game.battlefield(&p1).unwrap()[0].instance_id(),
            "thalia-1",
            "should keep the first (oldest) legendary permanent"
        );
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    #[test]
    fn non_legendary_duplicates_are_fine() {
        let (mut game, p1, _p2) = make_game_in_first_main();

        let bear_def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);

        let bear1 = CardInstance::new("bear-1", bear_def.clone(), &p1);
        let bear2 = CardInstance::new("bear-2", bear_def, &p1);

        add_permanent_to_battlefield(&mut game, &p1, bear1);
        add_permanent_to_battlefield(&mut game, &p1, bear2);

        game.perform_state_based_actions();

        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            2,
            "non-legendary duplicates should both stay"
        );
    }

    #[test]
    fn different_legendary_names_are_fine() {
        let (mut game, p1, _p2) = make_game_in_first_main();

        let thalia = CardDefinition::new("thalia", "Thalia", vec![CardType::Creature])
            .with_legendary()
            .with_power_toughness(2, 1);
        let jace = CardDefinition::new("jace", "Jace", vec![CardType::Creature])
            .with_legendary()
            .with_power_toughness(1, 3);

        add_permanent_to_battlefield(&mut game, &p1, CardInstance::new("thalia-1", thalia, &p1));
        add_permanent_to_battlefield(&mut game, &p1, CardInstance::new("jace-1", jace, &p1));

        game.perform_state_based_actions();

        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            2,
            "different legendary names should both stay"
        );
    }

    #[test]
    fn legend_rule_per_player() {
        let (mut game, p1, p2) = make_game_in_first_main();

        let legend_def = CardDefinition::new("thalia", "Thalia", vec![CardType::Creature])
            .with_legendary()
            .with_power_toughness(2, 1);

        // Each player has one Thalia — this is fine
        add_permanent_to_battlefield(
            &mut game,
            &p1,
            CardInstance::new("thalia-p1", legend_def.clone(), &p1),
        );
        add_permanent_to_battlefield(
            &mut game,
            &p2,
            CardInstance::new("thalia-p2", legend_def, &p2),
        );

        game.perform_state_based_actions();

        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
        assert_eq!(game.battlefield(&p2).unwrap().len(), 1);
    }
}
