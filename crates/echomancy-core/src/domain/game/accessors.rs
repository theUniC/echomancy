//! Read-only accessor methods for the `Game` aggregate.
//!
//! All methods in this file take `&self` and return read-only views of game
//! state. They are kept separate from mutation helpers to make the public API
//! surface easy to survey.

use crate::domain::entities::the_stack::StackItem;
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::enums::{GameLifecycleState, Step};
use crate::infrastructure::game_state_export::GameStateExport;
use crate::infrastructure::game_state_export::export_game_state;
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::types::PlayerId;
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;

use super::{Game, GameOutcome, GamePlayerState};

impl Game {
    // =========================================================================
    // Public accessors
    // =========================================================================

    /// The game's unique ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Current lifecycle state.
    pub fn lifecycle(&self) -> GameLifecycleState {
        self.lifecycle
    }

    /// The ID of the player whose turn it currently is.
    pub fn current_player_id(&self) -> &str {
        self.turn_state.current_player_id().as_str()
    }

    /// The current step in the turn.
    pub fn current_step(&self) -> Step {
        self.turn_state.current_step()
    }

    /// The current turn number (1-indexed).
    pub fn turn_number(&self) -> u32 {
        self.turn_state.turn_number()
    }

    /// The ID of the player who currently holds priority, if any.
    pub fn priority_player_id(&self) -> Option<&str> {
        self.priority_player_id.as_ref().map(PlayerId::as_str)
    }

    /// Number of lands the active player has played this turn.
    pub fn played_lands_this_turn(&self) -> u32 {
        self.turn_state.played_lands()
    }

    /// Player IDs in turn order.
    pub fn turn_order(&self) -> Vec<&str> {
        self.players.iter().map(|p| p.player_id.as_str()).collect()
    }

    /// Returns `true` if a player with the given ID is in the game.
    pub fn has_player(&self, player_id: &str) -> bool {
        self.players.iter().any(|p| p.player_id.as_str() == player_id)
    }

    /// Life total for a player.
    ///
    /// # Errors
    ///
    /// Returns `GameError::PlayerNotFound` if the player ID is unknown.
    pub fn player_life_total(&self, player_id: &str) -> Result<i32, GameError> {
        self.player_state(player_id).map(|p| p.life_total)
    }

    /// All cards in a player's hand (read-only slice).
    pub fn hand(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.hand.as_slice())
    }

    /// All permanents on a player's battlefield (read-only slice).
    pub fn battlefield(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.battlefield.as_slice())
    }

    /// All cards in a player's graveyard (read-only slice).
    pub fn graveyard(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.graveyard.as_slice())
    }

    /// All cards in a player's exile zone (read-only slice).
    pub fn exile(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.exile.as_slice())
    }

    /// Number of cards in a player's library.
    pub fn library_count(&self, player_id: &str) -> Result<usize, GameError> {
        self.player_state(player_id).map(|p| p.library.len())
    }

    /// The mana pool for a player.
    pub fn mana_pool(&self, player_id: &str) -> Result<&ManaPool, GameError> {
        self.player_state(player_id).map(|p| &p.mana_pool)
    }

    /// The `PermanentState` for a permanent, if it exists.
    pub fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
        self.permanent_states.get(instance_id)
    }

    /// The stack items from bottom (index 0) to top (last).
    pub fn stack(&self) -> &[StackItem] {
        &self.stack
    }

    /// All accumulated events since the game started.
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// The game outcome, if the game has finished.
    pub fn outcome(&self) -> Option<&GameOutcome> {
        self.outcome.as_ref()
    }

    /// Returns `true` if the stack has at least one item.
    pub fn stack_has_items(&self) -> bool {
        !self.stack.is_empty()
    }

    /// The opponent of a player in a 2-player game.
    ///
    /// # Errors
    ///
    /// Returns `GameError::PlayerNotFound` if `player_id` is not in the game.
    pub fn opponent_of(&self, player_id: &str) -> Result<&str, GameError> {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() != player_id)
            .map(|p| p.player_id.as_str())
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: PlayerId::new(player_id),
            })
    }

    /// Export the complete game state as a plain data structure.
    pub fn export_state(&self) -> GameStateExport {
        export_game_state(self)
    }

    // =========================================================================
    // Internal accessors (used by handlers and services)
    // =========================================================================

    /// Read-only reference to a player's state.
    pub(crate) fn player_state(&self, player_id: &str) -> Result<&GamePlayerState, GameError> {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: PlayerId::new(player_id),
            })
    }

    /// Mutable reference to a player's state.
    pub(crate) fn player_state_mut(
        &mut self,
        player_id: &str,
    ) -> Result<&mut GamePlayerState, GameError> {
        self.players
            .iter_mut()
            .find(|p| p.player_id.as_str() == player_id)
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: PlayerId::new(player_id),
            })
    }

    /// Whether a player has passed priority in the current window.
    #[allow(dead_code)]
    pub(crate) fn has_passed_priority(&self, player_id: &str) -> bool {
        self.players_who_passed_priority.contains(player_id)
    }

    /// Whether all players have passed priority.
    pub(crate) fn both_players_have_passed(&self) -> bool {
        self.players_who_passed_priority.len() >= self.players.len()
    }

    /// Whether a player is in auto-pass mode.
    #[allow(dead_code)]
    pub(crate) fn is_auto_pass(&self, player_id: &str) -> bool {
        self.auto_pass_players.contains(player_id)
    }

    /// Whether the current player holds priority.
    pub(crate) fn has_priority(&self, player_id: &str) -> bool {
        self.priority_player_id
            .as_ref()
            .map(|id| id.as_str() == player_id)
            .unwrap_or(false)
    }
}
