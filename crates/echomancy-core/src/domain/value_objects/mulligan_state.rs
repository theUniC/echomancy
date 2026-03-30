//! MulliganState — per-player mulligan progress tracking.
//!
//! Tracks which players have kept their hand, how many mulligans each player has
//! taken, and how many cards each player still needs to put on the bottom of their
//! library (the Vancouver Mulligan "put-back" step).
//!
//! This state is created at game start and discarded when all players have
//! completed their mulligan decisions.

/// Per-player mulligan tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlayerMulliganStatus {
    /// Whether this player has made their keep decision.
    pub(crate) has_kept: bool,
    /// How many times this player has taken a mulligan.
    pub(crate) mulligan_count: u32,
    /// How many cards this player still needs to put on the bottom.
    ///
    /// Set to `mulligan_count` when the player keeps after one or more mulligans.
    /// Decrements with each `PutCardOnBottom` action.
    pub(crate) cards_to_put_back: u32,
}

impl PlayerMulliganStatus {
    fn new() -> Self {
        Self {
            has_kept: false,
            mulligan_count: 0,
            cards_to_put_back: 0,
        }
    }

    /// Mark this player as having kept their hand.
    ///
    /// If they took mulligans, sets `cards_to_put_back` to the number of mulligans.
    pub(crate) fn record_keep(&mut self) {
        self.has_kept = true;
        self.cards_to_put_back = self.mulligan_count;
    }

    /// Increment the mulligan count (player shuffled and redrew).
    pub(crate) fn record_mulligan(&mut self) {
        self.mulligan_count += 1;
    }

    /// Decrement the cards-to-put-back counter after one card was placed on the bottom.
    ///
    /// Saturates at zero (no underflow).
    pub(crate) fn record_put_back(&mut self) {
        self.cards_to_put_back = self.cards_to_put_back.saturating_sub(1);
    }

    /// Whether this player's mulligan phase is complete:
    /// they have kept AND have no remaining cards to put back.
    pub(crate) fn is_complete(&self) -> bool {
        self.has_kept && self.cards_to_put_back == 0
    }
}

/// Tracks the mulligan phase for all players in a game.
///
/// `MulliganState` exists only while the game is in the mulligan phase.
/// Once all players have completed their decisions it can be discarded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MulliganState {
    /// Player IDs in turn order, kept to enforce ordering.
    pub(crate) player_ids: Vec<String>,
    /// Per-player status, keyed by player ID.
    pub(crate) statuses: std::collections::HashMap<String, PlayerMulliganStatus>,
}

// Accessor methods are used in tests and may be used by future callers.
#[allow(dead_code)]
impl MulliganState {
    /// Create a new `MulliganState` for the given player IDs.
    pub(crate) fn new(player_ids: Vec<String>) -> Self {
        let mut statuses = std::collections::HashMap::new();
        for id in &player_ids {
            statuses.insert(id.clone(), PlayerMulliganStatus::new());
        }
        Self { player_ids, statuses }
    }

    /// Return the status for the given player, if they are in the mulligan.
    pub(crate) fn status(&self, player_id: &str) -> Option<&PlayerMulliganStatus> {
        self.statuses.get(player_id)
    }

    /// Return a mutable status for the given player.
    pub(crate) fn status_mut(&mut self, player_id: &str) -> Option<&mut PlayerMulliganStatus> {
        self.statuses.get_mut(player_id)
    }

    /// Return `true` when all players have completed their mulligan decisions.
    pub(crate) fn all_complete(&self) -> bool {
        self.statuses.values().all(|s| s.is_complete())
    }

    /// Mulligan count for the given player (0 if player not found).
    pub(crate) fn mulligan_count(&self, player_id: &str) -> u32 {
        self.statuses
            .get(player_id)
            .map(|s| s.mulligan_count)
            .unwrap_or(0)
    }

    /// Cards to put back for the given player (0 if player not found).
    pub(crate) fn cards_to_put_back(&self, player_id: &str) -> u32 {
        self.statuses
            .get(player_id)
            .map(|s| s.cards_to_put_back)
            .unwrap_or(0)
    }

    /// Whether the given player has kept (false if player not found).
    pub(crate) fn has_kept(&self, player_id: &str) -> bool {
        self.statuses
            .get(player_id)
            .map(|s| s.has_kept)
            .unwrap_or(false)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn two_player_state() -> MulliganState {
        MulliganState::new(vec!["p1".to_owned(), "p2".to_owned()])
    }

    #[test]
    fn new_state_has_no_keeps_and_zero_counts() {
        let state = two_player_state();
        assert!(!state.has_kept("p1"));
        assert!(!state.has_kept("p2"));
        assert_eq!(state.mulligan_count("p1"), 0);
        assert_eq!(state.cards_to_put_back("p1"), 0);
    }

    #[test]
    fn all_complete_false_when_nobody_kept() {
        let state = two_player_state();
        assert!(!state.all_complete());
    }

    #[test]
    fn record_keep_with_zero_mulligans_completes_player() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_keep();
        // cards_to_put_back should be 0 (no mulligans taken)
        assert_eq!(state.cards_to_put_back("p1"), 0);
        assert!(state.status("p1").unwrap().is_complete());
    }

    #[test]
    fn record_mulligan_increments_count() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_mulligan();
        state.status_mut("p1").unwrap().record_mulligan();
        assert_eq!(state.mulligan_count("p1"), 2);
    }

    #[test]
    fn record_keep_after_mulligan_sets_cards_to_put_back() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_mulligan();
        state.status_mut("p1").unwrap().record_mulligan();
        state.status_mut("p1").unwrap().record_keep();
        assert_eq!(state.cards_to_put_back("p1"), 2);
        // Player has kept but still has cards to put back → not complete yet
        assert!(!state.status("p1").unwrap().is_complete());
    }

    #[test]
    fn record_put_back_decrements_counter() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_mulligan();
        state.status_mut("p1").unwrap().record_keep();
        assert_eq!(state.cards_to_put_back("p1"), 1);
        state.status_mut("p1").unwrap().record_put_back();
        assert_eq!(state.cards_to_put_back("p1"), 0);
        assert!(state.status("p1").unwrap().is_complete());
    }

    #[test]
    fn record_put_back_saturates_at_zero() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_mulligan();
        state.status_mut("p1").unwrap().record_keep();
        state.status_mut("p1").unwrap().record_put_back();
        // Calling put_back again should not underflow
        state.status_mut("p1").unwrap().record_put_back();
        assert_eq!(state.cards_to_put_back("p1"), 0);
    }

    #[test]
    fn all_complete_true_when_all_players_are_done() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_keep();
        state.status_mut("p2").unwrap().record_keep();
        assert!(state.all_complete());
    }

    #[test]
    fn all_complete_false_when_one_player_has_put_back_remaining() {
        let mut state = two_player_state();
        state.status_mut("p1").unwrap().record_keep();
        state.status_mut("p2").unwrap().record_mulligan();
        state.status_mut("p2").unwrap().record_keep();
        // p2 has cards_to_put_back = 1
        assert!(!state.all_complete());
        state.status_mut("p2").unwrap().record_put_back();
        assert!(state.all_complete());
    }
}
