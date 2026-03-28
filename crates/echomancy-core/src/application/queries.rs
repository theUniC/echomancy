//! Application queries — GetGameState, GetAllowedActions, and ListGames.
//!
//! Each query is a plain struct carrying its parameters. The associated
//! `execute` method takes `&dyn GameRepository` (read-only) and returns
//! `Result<T, ApplicationError>`.

use crate::application::errors::ApplicationError;
use crate::application::repository::GameRepository;
use crate::application::validation::validate_uuid;
use crate::domain::enums::{CardType, GameLifecycleState, Step};
use crate::domain::game::Game;
use crate::infrastructure::game_state_export::GameStateExport;

// ============================================================================
// GetGameState
// ============================================================================

/// Query that exports the full, unfiltered game state.
pub struct GetGameState {
    /// The UUID of the game to retrieve.
    game_id: String,
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
    /// Instance IDs of untapped lands on the player's battlefield that can be
    /// tapped to produce mana right now.
    pub tappable_lands: Vec<String>,
    /// Instance IDs of non-land spells in the player's hand that can be cast now.
    ///
    /// A spell is castable when:
    /// - The player has priority.
    /// - Timing permits (sorcery-speed: main phase, active player, empty stack).
    /// - The player can pay the mana cost from their current pool.
    pub castable_spells: Vec<String>,
    /// Instance IDs of creatures on the active player's battlefield that can
    /// legally be declared as attackers during the `DeclareAttackers` step.
    ///
    /// Empty when not in `DeclareAttackers` step or when the player is not the
    /// active player.
    pub attackable_creatures: Vec<String>,
    /// Instance IDs of creatures on the defending player's battlefield that can
    /// legally be declared as blockers during the `DeclareBlockers` step.
    ///
    /// Each entry is the blocker's instance ID. The UI can assign them to attack
    /// any currently-attacking creature. Empty outside of `DeclareBlockers`.
    pub blockable_creatures: Vec<String>,
}

/// Query that returns which actions a player can take right now.
pub struct GetAllowedActions {
    /// The UUID of the game.
    game_id: String,
    /// The UUID of the querying player.
    player_id: String,
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
    /// - `ApplicationError::InvalidPlayerId` — `player_id` is not a valid UUID.
    /// - `ApplicationError::GameNotFound` — no game with the given ID exists.
    pub fn execute(
        self,
        repo: &dyn GameRepository,
    ) -> Result<AllowedActionsResult, ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;
        validate_uuid(&self.player_id, |id| ApplicationError::InvalidPlayerId {
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

        // Collect untapped lands on the player's battlefield that can produce mana.
        // These are always tappable as long as the player has priority and they are untapped.
        let tappable_lands = collect_tappable_lands(game, &self.player_id);

        // Collect castable spells regardless of land-play eligibility.
        let castable_spells = collect_castable_spells(game, &self.player_id);

        let attackable_creatures = collect_attackable_creatures(game, &self.player_id);
        let blockable_creatures = collect_blockable_creatures(game, &self.player_id);

        if !can_play_land {
            return Ok(AllowedActionsResult {
                playable_lands: Vec::new(),
                tappable_lands,
                castable_spells,
                attackable_creatures,
                blockable_creatures,
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

        Ok(AllowedActionsResult { playable_lands, tappable_lands, castable_spells, attackable_creatures, blockable_creatures })
    }
}

// ============================================================================
// ListGames
// ============================================================================

/// A summary of a game for listing purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSummary {
    /// Unique identifier of the game.
    pub game_id: String,
    /// Number of players currently in the game.
    pub player_count: usize,
    /// Current lifecycle state of the game.
    pub lifecycle_state: GameLifecycleState,
    /// Current turn number. `None` if the game has not started.
    pub turn_number: Option<u32>,
    /// Current step. `None` if the game has not started.
    pub current_step: Option<Step>,
}

/// Query that returns a summary of all games in the repository.
pub struct ListGames;

impl ListGames {
    pub fn new() -> Self {
        Self
    }

    /// Execute the query against the repository.
    ///
    /// Returns a `Vec<GameSummary>` with one entry per game. Order is not
    /// guaranteed (depends on the repository's internal iteration order).
    ///
    /// This query is infallible — an empty repository returns an empty `Vec`.
    pub fn execute(self, repo: &dyn GameRepository) -> Vec<GameSummary> {
        repo.all().into_iter().map(game_summary_from).collect()
    }
}

impl Default for ListGames {
    fn default() -> Self {
        Self::new()
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
fn can_player_play_land(game: &Game, player_id: &str) -> bool {
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

/// Collect instance IDs of untapped lands on the player's battlefield that have
/// a mana-producing activated ability.
///
/// Tapping lands for mana is possible whenever the player has priority,
/// regardless of the game step (CR 605).
fn collect_tappable_lands(game: &Game, player_id: &str) -> Vec<String> {
    // Player must have priority to activate mana abilities.
    if game.priority_player_id() != Some(player_id) {
        return Vec::new();
    }

    game.battlefield(player_id)
        .unwrap_or(&[])
        .iter()
        .filter(|card| {
            // Must be a land with a mana ability.
            let is_land = card.definition().types().contains(&CardType::Land);
            let has_mana_ability = card
                .definition()
                .activated_ability()
                .is_some_and(|ab| ab.effect.is_mana_ability());
            if !is_land || !has_mana_ability {
                return false;
            }
            // Must be untapped.
            let is_tapped = game
                .permanent_state(card.instance_id())
                .is_some_and(|s| s.is_tapped());
            !is_tapped
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Collect instance IDs of non-land spells in the player's hand that can be cast
/// right now at sorcery speed.
///
/// Conditions (sorcery-speed only — instants/flash are a future extension):
/// 1. Player has priority.
/// 2. Player is the active player (their turn).
/// 3. Current step is a main phase (FirstMain or SecondMain).
/// 4. Stack is empty.
/// 5. Player can pay the card's mana cost from their current mana pool.
fn collect_castable_spells(game: &Game, player_id: &str) -> Vec<String> {
    use crate::domain::services::mana_payment::can_pay_cost;
    use crate::domain::value_objects::mana::ManaCost;

    // 1. Player must have priority.
    if game.priority_player_id() != Some(player_id) {
        return Vec::new();
    }

    // 2. Must be the active player's turn.
    if game.current_player_id() != player_id {
        return Vec::new();
    }

    // 3. Must be a main phase.
    let step = game.current_step();
    if !matches!(step, Step::FirstMain | Step::SecondMain) {
        return Vec::new();
    }

    // 4. Stack must be empty (sorcery speed).
    if game.stack_has_items() {
        return Vec::new();
    }

    let mana_pool = match game.mana_pool(player_id) {
        Ok(pool) => pool,
        Err(_) => return Vec::new(),
    };

    game.hand(player_id)
        .unwrap_or(&[])
        .iter()
        .filter(|card| {
            // Must not be a land.
            if card.definition().is_land() {
                return false;
            }
            // For MVP, only sorcery-speed spells (creatures and sorceries).
            // Instants can be cast at any time but their highlight is future work.
            let is_sorcery_speed = card.definition().types().iter().any(|t| {
                matches!(t, CardType::Creature | CardType::Sorcery)
            });
            if !is_sorcery_speed {
                return false;
            }
            // Must be able to afford the mana cost.
            let cost = card
                .definition()
                .mana_cost()
                .cloned()
                .unwrap_or_else(ManaCost::zero);
            can_pay_cost(mana_pool, &cost)
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Collect instance IDs of untapped, non-summoning-sick creatures on the active
/// player's battlefield that can be declared as attackers right now.
///
/// Conditions:
/// 1. Current step is `DeclareAttackers`.
/// 2. Player is the active (current) player.
/// 3. Creature is untapped.
/// 4. Creature has no summoning sickness (or has Haste).
/// 5. Creature has not already attacked this turn.
fn collect_attackable_creatures(game: &Game, player_id: &str) -> Vec<String> {
    use crate::domain::enums::StaticAbility;

    if game.current_step() != Step::DeclareAttackers {
        return Vec::new();
    }
    if game.current_player_id() != player_id {
        return Vec::new();
    }

    let battlefield = match game.battlefield(player_id) {
        Ok(bf) => bf,
        Err(_) => return Vec::new(),
    };

    battlefield
        .iter()
        .filter(|card| {
            if !card.definition().is_creature() {
                return false;
            }
            let Some(state) = game.permanent_state(card.instance_id()) else {
                return false;
            };
            if state.is_tapped() {
                return false;
            }
            let Some(cs) = state.creature_state() else {
                return false;
            };
            if cs.has_attacked_this_turn {
                return false;
            }
            // Summoning sickness check: blocked by Haste
            if cs.has_summoning_sickness
                && !card
                    .definition()
                    .static_abilities()
                    .contains(&StaticAbility::Haste)
            {
                return false;
            }
            true
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Collect instance IDs of untapped creatures on the defending player's
/// battlefield that can be declared as blockers right now.
///
/// Conditions:
/// 1. Current step is `DeclareBlockers`.
/// 2. Player is NOT the active player (i.e., they are the defending player).
/// 3. Creature is untapped.
/// 4. Creature is not already blocking.
fn collect_blockable_creatures(game: &Game, player_id: &str) -> Vec<String> {
    if game.current_step() != Step::DeclareBlockers {
        return Vec::new();
    }
    // Defending player is NOT the current (active) player.
    if game.current_player_id() == player_id {
        return Vec::new();
    }

    let battlefield = match game.battlefield(player_id) {
        Ok(bf) => bf,
        Err(_) => return Vec::new(),
    };

    battlefield
        .iter()
        .filter(|card| {
            if !card.definition().is_creature() {
                return false;
            }
            let Some(state) = game.permanent_state(card.instance_id()) else {
                return false;
            };
            if state.is_tapped() {
                return false;
            }
            let Some(cs) = state.creature_state() else {
                return false;
            };
            // Already blocking something
            if cs.blocking_creature_id.is_some() {
                return false;
            }
            true
        })
        .map(|card| card.instance_id().to_owned())
        .collect()
}

/// Build a `GameSummary` from a `&Game`.
fn game_summary_from(game: &Game) -> GameSummary {
    let lifecycle_state = game.lifecycle();
    let (turn_number, current_step) = if lifecycle_state == GameLifecycleState::Started {
        (Some(game.turn_number()), Some(game.current_step()))
    } else {
        (None, None)
    };

    GameSummary {
        game_id: game.id().to_owned(),
        player_count: game.turn_order().len(),
        lifecycle_state,
        turn_number,
        current_step,
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
    use uuid::Uuid;

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
    fn get_allowed_actions_rejects_invalid_player_uuid() {
        let repo = InMemoryGameRepository::new();
        let err = GetAllowedActions::new(uuid(), "bad-player-id")
            .execute(&repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidPlayerId { .. }));
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

    // ---- ListGames ---------------------------------------------------------

    #[test]
    fn list_games_returns_empty_for_empty_repository() {
        let repo = InMemoryGameRepository::new();
        let summaries = ListGames::new().execute(&repo);
        assert!(summaries.is_empty());
    }

    #[test]
    fn list_games_returns_one_entry_per_game() {
        let mut repo = InMemoryGameRepository::new();
        CreateGame::new(uuid()).execute(&mut repo).unwrap();
        CreateGame::new(uuid()).execute(&mut repo).unwrap();

        let summaries = ListGames::new().execute(&repo);
        assert_eq!(summaries.len(), 2);
    }

    #[test]
    fn list_games_created_game_has_zero_players_and_created_lifecycle() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        CreateGame::new(&game_id).execute(&mut repo).unwrap();

        let summaries = ListGames::new().execute(&repo);
        let summary = summaries.iter().find(|s| s.game_id == game_id).unwrap();

        assert_eq!(summary.player_count, 0);
        assert_eq!(summary.lifecycle_state, GameLifecycleState::Created);
        assert!(summary.turn_number.is_none());
        assert!(summary.current_step.is_none());
    }

    #[test]
    fn list_games_started_game_has_turn_info() {
        let (repo, game_id, _, _) = started_game_repo();

        let summaries = ListGames::new().execute(&repo);
        let summary = summaries.iter().find(|s| s.game_id == game_id).unwrap();

        assert_eq!(summary.player_count, 2);
        assert_eq!(summary.lifecycle_state, GameLifecycleState::Started);
        assert_eq!(summary.turn_number, Some(1));
        assert_eq!(summary.current_step, Some(Step::Untap));
    }

    #[test]
    fn list_games_includes_game_id_in_summary() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        CreateGame::new(&game_id).execute(&mut repo).unwrap();

        let summaries = ListGames::new().execute(&repo);
        assert!(summaries.iter().any(|s| s.game_id == game_id));
    }

    // ---- attackable_creatures -----------------------------------------------

    #[test]
    fn attackable_creatures_empty_outside_declare_attackers_step() {
        let (repo, game_id, p1, _) = started_game_repo();
        // Game starts in Untap — not DeclareAttackers.
        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
        assert!(result.attackable_creatures.is_empty());
    }

    #[test]
    fn attackable_creatures_returns_eligible_creatures_during_declare_attackers() {
        use crate::domain::game::test_helpers::{
            add_permanent_to_battlefield, clear_summoning_sickness, make_creature_card,
        };

        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        {
            let game = repo.find_by_id_mut(&game_id).unwrap();
            let creature = make_creature_card("bear-1", &p1, 2, 2);
            add_permanent_to_battlefield(game, &p1, creature);
            clear_summoning_sickness(game, "bear-1");
            // Advance to DeclareAttackers (5 steps: Untap→Upkeep→Draw→FirstMain→BeginningOfCombat→DeclareAttackers)
            for _ in 0..5 {
                game.apply(Action::AdvanceStep {
                    player_id: PlayerId::new(&p1),
                })
                .unwrap();
            }
        }

        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
        assert!(result.attackable_creatures.contains(&"bear-1".to_owned()));
    }

    #[test]
    fn attackable_creatures_excludes_tapped_creatures() {
        use crate::domain::game::test_helpers::{
            add_permanent_to_battlefield, clear_summoning_sickness, make_creature_card,
        };

        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        {
            let game = repo.find_by_id_mut(&game_id).unwrap();
            let creature = make_creature_card("bear-1", &p1, 2, 2);
            add_permanent_to_battlefield(game, &p1, creature);
            clear_summoning_sickness(game, "bear-1");
            game.tap_permanent("bear-1").unwrap();
            for _ in 0..5 {
                game.apply(Action::AdvanceStep {
                    player_id: PlayerId::new(&p1),
                })
                .unwrap();
            }
        }

        let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
        assert!(!result.attackable_creatures.contains(&"bear-1".to_owned()));
    }

    // ---- blockable_creatures ------------------------------------------------

    #[test]
    fn blockable_creatures_empty_outside_declare_blockers_step() {
        let (repo, game_id, _, p2) = started_game_repo();
        let result = GetAllowedActions::new(&game_id, &p2).execute(&repo).unwrap();
        assert!(result.blockable_creatures.is_empty());
    }

    #[test]
    fn blockable_creatures_returns_eligible_creatures_during_declare_blockers() {
        use crate::domain::game::test_helpers::{
            add_permanent_to_battlefield, clear_summoning_sickness, make_creature_card,
        };

        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        {
            let game = repo.find_by_id_mut(&game_id).unwrap();
            // Add attacker for p1
            let attacker = make_creature_card("attacker-1", &p1, 2, 2);
            add_permanent_to_battlefield(game, &p1, attacker);
            clear_summoning_sickness(game, "attacker-1");
            // Add potential blocker for p2
            let blocker = make_creature_card("blocker-1", &p2, 3, 3);
            add_permanent_to_battlefield(game, &p2, blocker);

            // Advance to DeclareAttackers
            for _ in 0..5 {
                game.apply(Action::AdvanceStep {
                    player_id: PlayerId::new(&p1),
                })
                .unwrap();
            }
            // Declare attacker
            game.apply(Action::DeclareAttacker {
                player_id: PlayerId::new(&p1),
                creature_id: crate::domain::types::CardInstanceId::new("attacker-1"),
            })
            .unwrap();
            // Advance to DeclareBlockers
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }

        // p2 is the defending player
        let result = GetAllowedActions::new(&game_id, &p2).execute(&repo).unwrap();
        assert!(result.blockable_creatures.contains(&"blocker-1".to_owned()));
    }
}
