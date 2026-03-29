//! Priority assignment and passing helpers for the `Game` aggregate.

use crate::domain::enums::Step;
use crate::domain::events::GameEvent;
use crate::domain::types::PlayerId;

use super::Game;

impl Game {
    /// Assign priority to a player, triggering auto-pass logic if applicable.
    pub(crate) fn assign_priority_to(&mut self, player_id: &str) -> Vec<GameEvent> {
        self.priority_player_id = Some(PlayerId::new(player_id));

        // Auto-pass: if player is in auto-pass mode
        if self.auto_pass_players.contains(player_id) {
            if self.stack_has_items() {
                // Auto-pass priority when stack is non-empty
                return self.perform_internal_pass(player_id);
            } else if player_id == self.turn_state.current_player_id().as_str() {
                // Auto-advance steps when stack is empty and they're the active player
                return self.process_auto_pass();
            }
        }
        Vec::new()
    }

    /// Give priority to the opponent of the given player.
    pub(crate) fn give_priority_to_opponent_of(
        &mut self,
        player_id: &str,
    ) -> Vec<GameEvent> {
        self.players_who_passed_priority.clear();
        if let Ok(opponent_id) = self.opponent_of(player_id).map(str::to_owned) {
            self.assign_priority_to(&opponent_id)
        } else {
            Vec::new()
        }
    }

    /// Record that a player has passed priority.
    pub(crate) fn record_passed_priority(&mut self, player_id: &str) {
        self.players_who_passed_priority.insert(player_id.to_owned());
    }

    pub(crate) fn perform_internal_pass(&mut self, player_id: &str) -> Vec<GameEvent> {
        self.players_who_passed_priority.insert(player_id.to_owned());

        if self.both_players_have_passed() {
            self.resolve_top_of_stack()
        } else {
            let opponent_id = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() != player_id)
                .map(|p| p.player_id.as_str().to_owned())
                .unwrap_or_default();
            self.assign_priority_to(&opponent_id)
        }
    }

    pub(crate) fn process_auto_pass(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let max_iterations = 100;
        let mut iterations = 0;
        let mut passed_cleanup = false;

        while iterations < max_iterations {
            iterations += 1;

            let current_player = self.turn_state.current_player_id().as_str().to_owned();

            // After passing Cleanup (turn changed), keep advancing through
            // the new player's non-interactive/empty steps until FirstMain.
            if passed_cleanup {
                if self.turn_state.current_step() == Step::FirstMain {
                    break; // Reached FirstMain of new turn — stop.
                }
                // Force-advance through Untap/Upkeep/Draw of new turn.
                events.extend(self.perform_step_advance());
                continue;
            }

            // Stop if active player is not in auto-pass
            if !self.auto_pass_players.contains(&current_player) {
                break;
            }

            // Stop if there's something on the stack
            if self.stack_has_items() {
                break;
            }

            // Advance through cleanup to next turn
            if self.turn_state.current_step() == Step::Cleanup {
                events.extend(self.perform_step_advance());
                passed_cleanup = true;
                continue; // Don't break — advance to new player's FirstMain.
            }

            events.extend(self.perform_step_advance());
        }

        events
    }
}
