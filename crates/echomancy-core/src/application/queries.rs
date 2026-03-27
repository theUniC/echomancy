//! Application queries — GetGameState and GetAllowedActions.
//!
//! Each query is a plain struct carrying its parameters. The associated
//! `execute` method takes `&dyn GameRepository` (read-only) and returns
//! `Result<T, ApplicationError>`.

use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::repository::GameRepository;
use crate::domain::enums::{CardType, Step};
use crate::domain::services::game_state_export::GameStateExport;

// ============================================================================
// GetGameState
// ============================================================================

/// Query that exports the full, unfiltered game state.
pub struct GetGameState {
    /// The UUID of the game to retrieve.
    pub game_id: String,
}

impl GetGameState {
    pub fn new(game_id: impl Into<String>) -> Self {
        Self {
            game_id: game_id.into(),
        }
    }

    /// Execute the query against the repository.
    ///
    /// # Errors
    ///
    /// - `ApplicationError::InvalidGameId` — `game_id` is not a valid UUID.
    /// - `ApplicationError::GameNotFound` — no game with the given ID exists.
    pub fn execute(self, repo: &dyn GameRepository) -> Result<GameStateExport, ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;

        let game = repo
            .find_by_id(&self.game_id)
            .ok_or_else(|| ApplicationError::GameNotFound {
                id: self.game_id.clone(),
            })?;

        Ok(game.export_state())
    }
}

// ============================================================================
// GetAllowedActions
// ============================================================================

/// The result of `GetAllowedActions`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedActionsResult {
    /// Instance IDs of land cards in the player's hand that can be played now.
    pub playable_lands: Vec<String>,
}

/// Query that returns which actions a player can take right now.
pub struct GetAllowedActions {
    /// The UUID of the game.
    pub game_id: String,
    /// The UUID of the querying player.
    pub player_id: String,
}

impl GetAllowedActions {
    pub fn new(game_id: impl Into<String>, player_id: impl Into<String>) -> Self {
        Self {
            game_id: game_id.into(),
            player_id: player_id.into(),
        }
    }

    /// Execute the query against the repository.
    ///
    /// # Errors
    ///
    /// - `ApplicationError::InvalidGameId` — `game_id` is not a valid UUID.
    /// - `ApplicationError::GameNotFound` — no game with the given ID exists.
    pub fn execute(
        self,
        repo: &dyn GameRepository,
    ) -> Result<AllowedActionsResult, ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;

        let game = repo
            .find_by_id(&self.game_id)
            .ok_or_else(|| ApplicationError::GameNotFound {
                id: self.game_id.clone(),
            })?;

        // Check whether PLAY_LAND is currently allowed using the same logic as
        // the domain specification (CanPlayLand). If not allowed, return empty.
        let can_play_land = can_player_play_land(game, &self.player_id);

        if !can_play_land {
            return Ok(AllowedActionsResult {
                playable_lands: Vec::new(),
            });
        }

        // Collect land cards from the player's hand.
        let playable_lands = game
            .hand(&self.player_id)
            .unwrap_or(&[])
            .iter()
            .filter(|c| c.definition().types().contains(&CardType::Land))
            .map(|c| c.instance_id().to_owned())
            .collect();

        Ok(AllowedActionsResult { playable_lands })
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Returns `true` if `player_id` can currently play a land.
///
/// Checks:
/// 1. Player is the active player (their turn).
/// 2. No land has been played this turn yet.
/// 3. Current step is a main phase.
/// 4. Stack is empty.
fn can_player_play_land(game: &crate::domain::game::Game, player_id: &str) -> bool {
    // 1. Must be active player.
    if game.current_player_id() != player_id {
        return false;
    }

    // 2. No land played yet.
    if game.played_lands_this_turn() > 0 {
        return false;
    }

    // 3. Must be a main phase.
    let step = game.current_step();
    let in_main_phase = matches!(step, Step::FirstMain | Step::SecondMain);
    if !in_main_phase {
        return false;
    }

    // 4. Stack must be empty.
    if game.stack_has_items() {
        return false;
    }

    true
}

/// Returns `Ok(())` if `id` parses as a valid UUID.
fn validate_uuid<E>(id: &str, make_err: impl Fn(&str) -> E) -> Result<(), E> {
    if Uuid::parse_str(id).is_ok() {
        Ok(())
    } else {
        Err(make_err(id))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::commands::{CreateGame, JoinGame, StartGame};
    use crate::domain::actions::Action;
    use crate::domain::enums::GameLifecycleState;
    use crate::domain::types::PlayerId;
    use crate::infrastructure::in_memory_repo::InMemoryGameRepository;

    fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    fn started_game_repo() -> (InMemoryGameRepository, String, String, String) {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        (repo, game_id, p1, p2)
    }

    // ---- GetGameState ------------------------------------------------------

    #[test]
    fn get_game_state_rejects_invalid_uuid() {
        let repo = InMemoryGameRepository::new();
        let err = GetGameState::new("not-a-uuid").execute(&repo).unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidGameId { .. }));
    }

    #[test]
    fn get_game_state_returns_not_found_for_missing_game() {
        let repo = InMemoryGameRepository::new();
        let err = GetGameState::new(uuid()).execute(&repo).unwrap_err();
        assert!(matches!(err, ApplicationError::GameNotFound { .. }));
    }

    #[test]
    fn get_game_state_returns_correct_game_id() {
        let (repo, game_id, _, _) = started_game_repo();
        let state = GetGameState::new(&game_id).execute(&repo).unwrap();
        assert_eq!(state.game_id, game_id);
    }

    #[test]
    fn get_game_state_returns_started_lifecycle() {
        let (repo, game_id, _, _) = started_game_repo();
        let state = GetGameState::new(&game_id).execute(&repo).unwrap();
        assert_eq!(state.lifecycle_state, GameLifecycleState::Started);
    }

    #[test]
    fn get_game_state_includes_both_players() {
        let (repo, game_id, p1, p2) = started_game_repo();
        let state = GetGameState::new(&game_id).execute(&repo).unwrap();
        assert!(state.players.contains_key(&p1));
        assert!(state.players.contains_key(&p2));
    }

    #[test]
    fn get_game_state_players_have_correct_life_totals() {
        let (repo, game_id, p1, _) = started_game_repo();
        let state = GetGameState::new(&game_id).execute(&repo).unwrap();
        assert_eq!(state.players[&p1].life_total, 20);
    }

    #[test]
    fn get_game_state_returns_empty_stack() {
        let (repo, game_id, _, _) = started_game_repo();
        let state = GetGameState::new(&game_id).execute(&repo).unwrap();
        assert!(state.stack.is_empty());
    }

    #[test]
    fn get_game_state_turn_order_has_two_players() {
        let (repo, game_id, _, _) = started_game_repo();
        let state = GetGameState::new(&game_id).execute(&repo).unwrap();
        assert_eq!(state.turn_order.len(), 2);
    }

    // ---- GetAllowedActions -------------------------------------------------

    #[test]
    fn get_allowed_actions_rejects_invalid_game_uuid() {
        let repo = InMemoryGameRepository::new();
        let err = GetAllowedActions::new("bad", uuid())
            .execute(&repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidGameId { .. }));
    }

    #[test]
    fn get_allowed_actions_returns_not_found_for_missing_game() {
        let repo = InMemoryGameRepository::new();
        let err = GetAllowedActions::new(uuid(), uuid())
            .execute(&repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::GameNotFound { .. }));
    }

    #[test]
    fn get_allowed_actions_returns_empty_in_untap_step() {
        let (repo, game_id, p1, _) = started_game_repo();
        // Game starts in UNTAP — no lands playable.
        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
        assert!(result.playable_lands.is_empty());
    }

    #[test]
    fn get_allowed_actions_returns_land_ids_in_first_main() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        // Advance to FIRST_MAIN (UNTAP → UPKEEP → DRAW → FIRST_MAIN)
        for _ in 0..3 {
            repo.find_by_id_mut(&game_id).unwrap().apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            }).unwrap();
        }

        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
        // The bootstrap hand always has 2 lands.
        assert_eq!(result.playable_lands.len(), 2);
    }

    #[test]
    fn get_allowed_actions_returns_empty_after_land_played() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        // Advance to FIRST_MAIN.
        for _ in 0..3 {
            repo.find_by_id_mut(&game_id).unwrap().apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            }).unwrap();
        }

        // Pick a land card and play it.
        let land_id = {
            let game = repo.find_by_id(&game_id).unwrap();
            game.hand(&p1)
                .unwrap()
                .iter()
                .find(|c| c.definition().types().contains(&CardType::Land))
                .unwrap()
                .instance_id()
                .to_owned()
        };

        repo.find_by_id_mut(&game_id).unwrap().apply(Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: crate::domain::types::CardInstanceId::new(&land_id),
        }).unwrap();

        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
        assert!(result.playable_lands.is_empty());
    }

    #[test]
    fn get_allowed_actions_returns_empty_for_non_active_player() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        // Advance to FIRST_MAIN.
        for _ in 0..3 {
            repo.find_by_id_mut(&game_id).unwrap().apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            }).unwrap();
        }

        // p2 is not the active player.
        let result = GetAllowedActions::new(&game_id, &p2).execute(&repo).unwrap();
        assert!(result.playable_lands.is_empty());
    }

    #[test]
    fn get_allowed_actions_filters_non_land_cards() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        // Advance to FIRST_MAIN.
        for _ in 0..3 {
            repo.find_by_id_mut(&game_id).unwrap().apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            }).unwrap();
        }

        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();

        // Verify all returned IDs are actually land cards.
        let game = repo.find_by_id(&game_id).unwrap();
        for id in &result.playable_lands {
            let card = game
                .hand(&p1)
                .unwrap()
                .iter()
                .find(|c| c.instance_id() == id)
                .unwrap();
            assert!(card.definition().types().contains(&CardType::Land));
        }
    }
}
