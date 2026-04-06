use serde::{Deserialize, Serialize};

use crate::domain::enums::Step;
use crate::domain::types::PlayerId;

// ============================================================================
// TurnStateSnapshot
// ============================================================================

/// Serialisable snapshot of turn state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnStateSnapshot {
    pub current_player_id: PlayerId,
    pub current_step: Step,
    pub turn_number: u32,
    pub played_lands: u32,
}

// ============================================================================
// TurnState
// ============================================================================

/// Immutable value object that tracks turn-related game state.
///
/// All mutating operations return **new** instances; the original is unchanged.
///
/// # Examples
///
/// ```
/// use echomancy_core::prelude::{TurnState, Step, PlayerId};
///
/// let state = TurnState::initial(PlayerId::new("player-1"));
/// assert_eq!(state.turn_number(), 1);
/// assert_eq!(state.current_step(), Step::Untap);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnState {
    current_player_id: PlayerId,
    current_step: Step,
    turn_number: u32,
    played_lands: u32,
}

impl TurnState {
    // ---- constructors -------------------------------------------------------

    /// Creates the initial `TurnState` for the first turn of the game.
    pub fn initial(starting_player_id: PlayerId) -> Self {
        Self {
            current_player_id: starting_player_id,
            current_step: Step::Untap,
            turn_number: 1,
            played_lands: 0,
        }
    }

    /// Reconstructs a `TurnState` from a previously captured snapshot.
    pub fn from_snapshot(snapshot: TurnStateSnapshot) -> Self {
        Self {
            current_player_id: snapshot.current_player_id,
            current_step: snapshot.current_step,
            turn_number: snapshot.turn_number,
            played_lands: snapshot.played_lands,
        }
    }

    // ---- accessors ----------------------------------------------------------

    pub fn current_player_id(&self) -> &PlayerId {
        &self.current_player_id
    }

    pub fn current_step(&self) -> Step {
        self.current_step
    }

    pub fn turn_number(&self) -> u32 {
        self.turn_number
    }

    pub fn played_lands(&self) -> u32 {
        self.played_lands
    }

    // ---- derived predicates -------------------------------------------------

    /// Returns `true` if the current step is a main phase
    /// (`FirstMain` or `SecondMain`).
    pub fn is_main_phase(&self) -> bool {
        matches!(self.current_step, Step::FirstMain | Step::SecondMain)
    }

    /// Returns `true` if at least one land has been played this turn.
    pub fn has_played_land(&self) -> bool {
        self.played_lands > 0
    }

    // ---- builders -----------------------------------------------------------

    /// Returns a new `TurnState` with the given step.
    pub fn with_step(&self, step: Step) -> Self {
        Self {
            current_step: step,
            ..self.clone()
        }
    }

    /// Returns a new `TurnState` with a different active player.
    pub fn with_current_player(&self, player_id: PlayerId) -> Self {
        Self {
            current_player_id: player_id,
            ..self.clone()
        }
    }

    /// Returns a new `TurnState` with `turn_number` incremented by 1.
    pub fn with_incremented_turn_number(&self) -> Self {
        Self {
            turn_number: self.turn_number + 1,
            ..self.clone()
        }
    }

    /// Returns a new `TurnState` with `played_lands` incremented by 1.
    pub fn with_land_played(&self) -> Self {
        Self {
            played_lands: self.played_lands + 1,
            ..self.clone()
        }
    }

    /// Returns a new `TurnState` with `played_lands` reset to 0.
    pub fn with_reset_lands(&self) -> Self {
        Self {
            played_lands: 0,
            ..self.clone()
        }
    }

    /// Returns a new `TurnState` representing the start of a fresh turn for
    /// `next_player_id`. The turn number is **not** incremented here — callers
    /// should call `with_incremented_turn_number` separately if needed.
    pub fn for_new_turn(&self, next_player_id: PlayerId) -> Self {
        Self {
            current_player_id: next_player_id,
            current_step: Step::Untap,
            turn_number: self.turn_number,
            played_lands: 0,
        }
    }

    // ---- snapshot ----------------------------------------------------------

    /// Exports the state as a serialisable snapshot.
    pub fn to_snapshot(&self) -> TurnStateSnapshot {
        TurnStateSnapshot {
            current_player_id: self.current_player_id.clone(),
            current_step: self.current_step,
            turn_number: self.turn_number,
            played_lands: self.played_lands,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::Step;
    use crate::domain::types::PlayerId;

    fn player(id: &str) -> PlayerId {
        PlayerId::new(id)
    }

    // ---- initial -----------------------------------------------------------

    #[test]
    fn initial_sets_untap_step_turn_1_no_lands() {
        let state = TurnState::initial(player("p1"));
        assert_eq!(state.current_step(), Step::Untap);
        assert_eq!(state.turn_number(), 1);
        assert_eq!(state.played_lands(), 0);
        assert_eq!(state.current_player_id(), &player("p1"));
    }

    // ---- with_step ---------------------------------------------------------

    #[test]
    fn with_step_returns_new_instance_original_unchanged() {
        let state = TurnState::initial(player("p1"));
        let next = state.with_step(Step::FirstMain);

        assert_eq!(state.current_step(), Step::Untap); // original unchanged
        assert_eq!(next.current_step(), Step::FirstMain);
    }

    // ---- with_current_player -----------------------------------------------

    #[test]
    fn with_current_player_changes_player() {
        let state = TurnState::initial(player("p1"));
        let next = state.with_current_player(player("p2"));

        assert_eq!(state.current_player_id(), &player("p1")); // original
        assert_eq!(next.current_player_id(), &player("p2"));
    }

    // ---- with_incremented_turn_number --------------------------------------

    #[test]
    fn incremented_turn_number_adds_one() {
        let state = TurnState::initial(player("p1"));
        let next = state.with_incremented_turn_number();
        assert_eq!(next.turn_number(), 2);
        assert_eq!(state.turn_number(), 1); // original unchanged
    }

    // ---- with_land_played --------------------------------------------------

    #[test]
    fn with_land_played_increments_count() {
        let state = TurnState::initial(player("p1")).with_land_played();
        assert_eq!(state.played_lands(), 1);
    }

    #[test]
    fn multiple_land_played_calls_accumulate() {
        let state = TurnState::initial(player("p1"))
            .with_land_played()
            .with_land_played();
        assert_eq!(state.played_lands(), 2);
    }

    // ---- with_reset_lands --------------------------------------------------

    #[test]
    fn with_reset_lands_zeroes_count() {
        let state = TurnState::initial(player("p1"))
            .with_land_played()
            .with_reset_lands();
        assert_eq!(state.played_lands(), 0);
    }

    // ---- for_new_turn ------------------------------------------------------

    #[test]
    fn for_new_turn_resets_step_and_lands() {
        let state = TurnState::initial(player("p1"))
            .with_step(Step::SecondMain)
            .with_land_played()
            .with_incremented_turn_number();
        let next = state.for_new_turn(player("p2"));

        assert_eq!(next.current_step(), Step::Untap);
        assert_eq!(next.played_lands(), 0);
        assert_eq!(next.current_player_id(), &player("p2"));
        // turn_number carries over (matches TS behaviour)
        assert_eq!(next.turn_number(), state.turn_number());
    }

    // ---- is_main_phase -----------------------------------------------------

    #[test]
    fn untap_is_not_main_phase() {
        let state = TurnState::initial(player("p1"));
        assert!(!state.is_main_phase());
    }

    #[test]
    fn first_main_is_main_phase() {
        let state = TurnState::initial(player("p1")).with_step(Step::FirstMain);
        assert!(state.is_main_phase());
    }

    #[test]
    fn second_main_is_main_phase() {
        let state = TurnState::initial(player("p1")).with_step(Step::SecondMain);
        assert!(state.is_main_phase());
    }

    #[test]
    fn declare_attackers_is_not_main_phase() {
        let state = TurnState::initial(player("p1")).with_step(Step::DeclareAttackers);
        assert!(!state.is_main_phase());
    }

    // ---- has_played_land ---------------------------------------------------

    #[test]
    fn has_played_land_false_initially() {
        let state = TurnState::initial(player("p1"));
        assert!(!state.has_played_land());
    }

    #[test]
    fn has_played_land_true_after_land_played() {
        let state = TurnState::initial(player("p1")).with_land_played();
        assert!(state.has_played_land());
    }

    // ---- snapshot round-trip -----------------------------------------------

    #[test]
    fn snapshot_roundtrip_preserves_all_fields() {
        let original = TurnState::initial(player("p1"))
            .with_step(Step::CombatDamage)
            .with_incremented_turn_number()
            .with_land_played();
        let snap = original.to_snapshot();
        let restored = TurnState::from_snapshot(snap);
        assert_eq!(original, restored);
    }
}
