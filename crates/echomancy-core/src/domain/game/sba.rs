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
                .filter(|c| c.definition().has_static_ability(StaticAbility::Indestructible))
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
                .map(|(id, s)| CreatureSbaEntry {
                    instance_id: id.as_str(),
                    state: s,
                    is_indestructible: indestructible_ids.contains(id),
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
            .collect();

        if !all_losers.is_empty() {
            let reason = if !losers_by_life.is_empty() && !losers_by_library.is_empty() {
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
}
