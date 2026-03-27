//! Application commands — create game, join game, start game, apply action.
//!
//! Each command is a plain struct carrying its parameters. The associated
//! `execute` method takes `&mut dyn GameRepository` and returns
//! `Result<T, ApplicationError>`.
//!
//! Design:
//! - Commands validate IDs before touching the repository or domain.
//! - Domain logic stays in `Game::*` — commands are thin orchestrators.
//! - The bootstrap hand-population in `StartGame` is a temporary mechanism
//!   until the full deck/library system is implemented.

use uuid::Uuid;

use crate::application::errors::ApplicationError;
use crate::application::repository::GameRepository;
use crate::application::validation::validate_uuid;
use crate::domain::actions::Action;
use crate::domain::cards::card_definition::CardDefinition;
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{CardType, StaticAbility};
use crate::domain::game::Game;

// ============================================================================
// CreateGame
// ============================================================================

/// Command to create a new game with the given ID.
pub struct CreateGame {
    /// The UUID that will identify the game.
    game_id: String,
}

impl CreateGame {
    pub fn new(game_id: impl Into<String>) -> Self {
        Self {
            game_id: game_id.into(),
        }
    }

    /// Execute the command against the repository.
    ///
    /// # Errors
    ///
    /// - `ApplicationError::InvalidGameId` — `game_id` is not a valid UUID.
    pub fn execute(self, repo: &mut dyn GameRepository) -> Result<(), ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;

        let game = Game::create(self.game_id);
        repo.save(game);
        Ok(())
    }
}

// ============================================================================
// JoinGame
// ============================================================================

/// Command to add a player to an existing game.
pub struct JoinGame {
    /// The UUID of the game to join.
    game_id: String,
    /// The UUID of the joining player.
    player_id: String,
    /// The display name of the joining player.
    player_name: String,
}

impl JoinGame {
    pub fn new(
        game_id: impl Into<String>,
        player_id: impl Into<String>,
        player_name: impl Into<String>,
    ) -> Self {
        Self {
            game_id: game_id.into(),
            player_id: player_id.into(),
            player_name: player_name.into(),
        }
    }

    /// Execute the command against the repository.
    ///
    /// # Errors
    ///
    /// - `ApplicationError::InvalidGameId` — `game_id` is not a valid UUID.
    /// - `ApplicationError::InvalidPlayerId` — `player_id` is not a valid UUID.
    /// - `ApplicationError::GameNotFound` — no game with the given ID exists.
    /// - `ApplicationError::Domain` — domain rule violation (duplicate player, game started, etc.).
    pub fn execute(self, repo: &mut dyn GameRepository) -> Result<(), ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;
        validate_uuid(&self.player_id, |id| ApplicationError::InvalidPlayerId {
            id: id.to_owned(),
        })?;

        let game = repo
            .find_by_id_mut(&self.game_id)
            .ok_or_else(|| ApplicationError::GameNotFound {
                id: self.game_id.clone(),
            })?;

        game.add_player(&self.player_id, &self.player_name)?;
        Ok(())
    }
}

// ============================================================================
// StartGame
// ============================================================================

/// Command to start a game.
///
/// Bootstraps each player's hand with 7 predetermined cards (2 lands + 5
/// creatures) as a temporary mechanism until the full deck/library system is
/// implemented.
pub struct StartGame {
    /// The UUID of the game to start.
    game_id: String,
    /// The UUID of the player who goes first.
    starting_player_id: String,
}

impl StartGame {
    pub fn new(
        game_id: impl Into<String>,
        starting_player_id: impl Into<String>,
    ) -> Self {
        Self {
            game_id: game_id.into(),
            starting_player_id: starting_player_id.into(),
        }
    }

    /// Execute the command against the repository.
    ///
    /// # Errors
    ///
    /// - `ApplicationError::InvalidGameId` — `game_id` is not a valid UUID.
    /// - `ApplicationError::InvalidPlayerId` — `starting_player_id` is not a valid UUID.
    /// - `ApplicationError::GameNotFound` — no game with the given ID exists.
    /// - `ApplicationError::Domain` — domain rule violation (already started, wrong player count, etc.).
    pub fn execute(self, repo: &mut dyn GameRepository) -> Result<(), ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;
        validate_uuid(&self.starting_player_id, |id| {
            ApplicationError::InvalidPlayerId { id: id.to_owned() }
        })?;

        // Find the game and get player IDs before mutating.
        let player_ids: Vec<String> = {
            let game = repo
                .find_by_id(&self.game_id)
                .ok_or_else(|| ApplicationError::GameNotFound {
                    id: self.game_id.clone(),
                })?;
            game.turn_order()
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

        // Assign bootstrap hands to every player.
        for player_id in &player_ids {
            let hand = bootstrap_hand(player_id);
            let game = repo
                .find_by_id_mut(&self.game_id)
                .ok_or_else(|| ApplicationError::GameNotFound {
                    id: self.game_id.clone(),
                })?;
            game.assign_deck(player_id, hand)?;
        }

        // Start the game (shuffles libraries and deals 7-card hands).
        let game = repo
            .find_by_id_mut(&self.game_id)
            .ok_or_else(|| ApplicationError::GameNotFound {
                id: self.game_id.clone(),
            })?;
        game.start(&self.starting_player_id, Some(42))?;
        Ok(())
    }
}

// ============================================================================
// ApplyAction
// ============================================================================

/// Command to apply a player action to the game.
pub struct ApplyAction {
    /// The UUID of the game to apply the action to.
    game_id: String,
    /// The action to apply.
    action: Action,
}

impl ApplyAction {
    pub fn new(game_id: impl Into<String>, action: Action) -> Self {
        Self {
            game_id: game_id.into(),
            action,
        }
    }

    /// Execute the command against the repository.
    ///
    /// # Errors
    ///
    /// - `ApplicationError::InvalidGameId` — `game_id` is not a valid UUID.
    /// - `ApplicationError::GameNotFound` — no game with the given ID exists.
    /// - `ApplicationError::Domain` — domain rule violation.
    pub fn execute(self, repo: &mut dyn GameRepository) -> Result<(), ApplicationError> {
        validate_uuid(&self.game_id, |id| ApplicationError::InvalidGameId {
            id: id.to_owned(),
        })?;

        let game = repo
            .find_by_id_mut(&self.game_id)
            .ok_or_else(|| ApplicationError::GameNotFound {
                id: self.game_id.clone(),
            })?;

        game.apply(self.action)?;
        Ok(())
    }
}

// ============================================================================
// Bootstrap helpers (temporary until deck/library system is implemented)
// ============================================================================

/// Specification for a card to be created in the bootstrap hand.
struct CardSpec<'a> {
    def_id: &'a str,
    name: &'a str,
    types: Vec<CardType>,
    power: Option<u32>,
    toughness: Option<u32>,
    static_abilities: Vec<StaticAbility>,
}

impl<'a> CardSpec<'a> {
    fn into_card_instance(self, owner_id: &str) -> CardInstance {
        let mut def = CardDefinition::new(self.def_id, self.name, self.types);
        if let (Some(p), Some(t)) = (self.power, self.toughness) {
            def = def.with_power_toughness(p, t);
        }
        for ability in self.static_abilities {
            def = def.with_static_ability(ability);
        }
        CardInstance::new(Uuid::new_v4().to_string(), def, owner_id)
    }
}

/// Creates the temporary 7-card bootstrap hand for a player.
///
/// Composition: 2 lands (Forest, Plains) + 5 creatures.
/// This mirrors `populateStartingHands` from the TypeScript source.
fn bootstrap_hand(owner_id: &str) -> Vec<CardInstance> {
    let specs = vec![
        CardSpec {
            def_id: "forest",
            name: "Forest",
            types: vec![CardType::Land],
            power: None,
            toughness: None,
            static_abilities: vec![],
        },
        CardSpec {
            def_id: "plains",
            name: "Plains",
            types: vec![CardType::Land],
            power: None,
            toughness: None,
            static_abilities: vec![],
        },
        CardSpec {
            def_id: "grizzly-bears",
            name: "Grizzly Bears",
            types: vec![CardType::Creature],
            power: Some(2),
            toughness: Some(2),
            static_abilities: vec![],
        },
        CardSpec {
            def_id: "elite-vanguard",
            name: "Elite Vanguard",
            types: vec![CardType::Creature],
            power: Some(2),
            toughness: Some(1),
            static_abilities: vec![],
        },
        CardSpec {
            def_id: "giant-spider",
            name: "Giant Spider",
            types: vec![CardType::Creature],
            power: Some(2),
            toughness: Some(4),
            static_abilities: vec![StaticAbility::Reach],
        },
        CardSpec {
            def_id: "serra-angel",
            name: "Serra Angel",
            types: vec![CardType::Creature],
            power: Some(4),
            toughness: Some(4),
            static_abilities: vec![StaticAbility::Flying, StaticAbility::Vigilance],
        },
        CardSpec {
            def_id: "llanowar-elves",
            name: "Llanowar Elves",
            types: vec![CardType::Creature],
            power: Some(1),
            toughness: Some(1),
            static_abilities: vec![],
        },
    ];

    specs
        .into_iter()
        .map(|spec| spec.into_card_instance(owner_id))
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::PlayerId;
    use crate::infrastructure::in_memory_repo::InMemoryGameRepository;

    fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    // ---- CreateGame --------------------------------------------------------

    #[test]
    fn create_game_rejects_invalid_uuid() {
        let mut repo = InMemoryGameRepository::new();
        let err = CreateGame::new("not-a-uuid")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidGameId { .. }));
    }

    #[test]
    fn create_game_stores_game_in_repository() {
        let mut repo = InMemoryGameRepository::new();
        let id = uuid();
        CreateGame::new(&id).execute(&mut repo).unwrap();
        assert!(repo.find_by_id(&id).is_some());
    }

    // ---- JoinGame ----------------------------------------------------------

    #[test]
    fn join_game_rejects_invalid_game_uuid() {
        let mut repo = InMemoryGameRepository::new();
        let err = JoinGame::new("bad-game-id", uuid(), "Alice")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidGameId { .. }));
    }

    #[test]
    fn join_game_rejects_invalid_player_uuid() {
        let mut repo = InMemoryGameRepository::new();
        let err = JoinGame::new(uuid(), "bad-player-id", "Alice")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidPlayerId { .. }));
    }

    #[test]
    fn join_game_returns_not_found_for_missing_game() {
        let mut repo = InMemoryGameRepository::new();
        let err = JoinGame::new(uuid(), uuid(), "Alice")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::GameNotFound { .. }));
    }

    #[test]
    fn join_game_adds_player_to_game() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let player_id = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &player_id, "Alice")
            .execute(&mut repo)
            .unwrap();

        let game = repo.find_by_id(&game_id).unwrap();
        assert!(game.has_player(&player_id));
    }

    #[test]
    fn join_game_rejects_duplicate_player() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let player_id = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &player_id, "Alice")
            .execute(&mut repo)
            .unwrap();
        let err = JoinGame::new(&game_id, &player_id, "Alice")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[test]
    fn join_game_rejects_after_game_started() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();
        let p3 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let err = JoinGame::new(&game_id, &p3, "Charlie")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    // ---- StartGame ---------------------------------------------------------

    #[test]
    fn start_game_rejects_invalid_game_uuid() {
        let mut repo = InMemoryGameRepository::new();
        let err = StartGame::new("bad-game-id", uuid())
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidGameId { .. }));
    }

    #[test]
    fn start_game_rejects_invalid_starting_player_uuid() {
        let mut repo = InMemoryGameRepository::new();
        let err = StartGame::new(uuid(), "bad-player-id")
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidPlayerId { .. }));
    }

    #[test]
    fn start_game_returns_not_found_for_missing_game() {
        let mut repo = InMemoryGameRepository::new();
        let err = StartGame::new(uuid(), uuid())
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::GameNotFound { .. }));
    }

    #[test]
    fn start_game_with_less_than_two_players_is_domain_error() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();

        let err = StartGame::new(&game_id, &p1).execute(&mut repo).unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[test]
    fn start_game_with_invalid_starting_player_is_domain_error() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();
        let nonexistent = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();

        let err = StartGame::new(&game_id, &nonexistent)
            .execute(&mut repo)
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[test]
    fn start_game_succeeds_with_valid_inputs() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let game = repo.find_by_id(&game_id).unwrap();
        assert_eq!(game.current_player_id(), p1.as_str());
    }

    #[test]
    fn start_game_rejects_already_started_game() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let err = StartGame::new(&game_id, &p2).execute(&mut repo).unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[test]
    fn start_game_gives_each_player_seven_cards_in_hand() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let game = repo.find_by_id(&game_id).unwrap();
        assert_eq!(game.hand(&p1).unwrap().len(), 7);
        assert_eq!(game.hand(&p2).unwrap().len(), 7);
    }

    #[test]
    fn start_game_hand_contains_two_lands_and_five_creatures() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let game = repo.find_by_id(&game_id).unwrap();
        let hand = game.hand(&p1).unwrap();
        let lands = hand
            .iter()
            .filter(|c| c.definition().types().contains(&CardType::Land))
            .count();
        let creatures = hand
            .iter()
            .filter(|c| c.definition().types().contains(&CardType::Creature))
            .count();
        assert_eq!(lands, 2);
        assert_eq!(creatures, 5);
    }

    #[test]
    fn start_game_cards_have_unique_instance_ids() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let game = repo.find_by_id(&game_id).unwrap();
        let mut ids: Vec<&str> = game
            .hand(&p1)
            .unwrap()
            .iter()
            .chain(game.hand(&p2).unwrap().iter())
            .map(|c| c.instance_id())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 14);
    }

    // ---- ApplyAction -------------------------------------------------------

    #[test]
    fn apply_action_rejects_invalid_game_uuid() {
        let mut repo = InMemoryGameRepository::new();
        let err = ApplyAction::new(
            "bad-game-id",
            Action::AdvanceStep {
                player_id: PlayerId::new(uuid()),
            },
        )
        .execute(&mut repo)
        .unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidGameId { .. }));
    }

    #[test]
    fn apply_action_returns_not_found_for_missing_game() {
        let mut repo = InMemoryGameRepository::new();
        let err = ApplyAction::new(
            uuid(),
            Action::AdvanceStep {
                player_id: PlayerId::new(uuid()),
            },
        )
        .execute(&mut repo)
        .unwrap_err();
        assert!(matches!(err, ApplicationError::GameNotFound { .. }));
    }

    #[test]
    fn apply_action_returns_domain_error_for_unstarted_game() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, uuid(), "Bob").execute(&mut repo).unwrap();

        let err = ApplyAction::new(
            &game_id,
            Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            },
        )
        .execute(&mut repo)
        .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[test]
    fn apply_action_advance_step_succeeds_for_started_game() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        let initial_step = repo.find_by_id(&game_id).unwrap().current_step();

        ApplyAction::new(
            &game_id,
            Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            },
        )
        .execute(&mut repo)
        .unwrap();

        let new_step = repo.find_by_id(&game_id).unwrap().current_step();
        assert_ne!(initial_step, new_step);
    }

    #[test]
    fn apply_action_rejects_wrong_player_action() {
        let mut repo = InMemoryGameRepository::new();
        let game_id = uuid();
        let p1 = uuid();
        let p2 = uuid();

        CreateGame::new(&game_id).execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
        JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();
        StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

        // p2 is not the active player and cannot advance step
        let err = ApplyAction::new(
            &game_id,
            Action::AdvanceStep {
                player_id: PlayerId::new(&p2),
            },
        )
        .execute(&mut repo)
        .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    // ---- bootstrap_hand ----------------------------------------------------

    #[test]
    fn bootstrap_hand_has_seven_cards() {
        let hand = bootstrap_hand("player-1");
        assert_eq!(hand.len(), 7);
    }

    #[test]
    fn bootstrap_hand_has_two_lands_and_five_creatures() {
        let hand = bootstrap_hand("player-1");
        let lands = hand
            .iter()
            .filter(|c| c.definition().types().contains(&CardType::Land))
            .count();
        let creatures = hand
            .iter()
            .filter(|c| c.definition().types().contains(&CardType::Creature))
            .count();
        assert_eq!(lands, 2);
        assert_eq!(creatures, 5);
    }

    #[test]
    fn bootstrap_hand_has_correct_creature_names() {
        let hand = bootstrap_hand("player-1");
        let mut names: Vec<&str> = hand
            .iter()
            .filter(|c| c.definition().types().contains(&CardType::Creature))
            .map(|c| c.definition().name())
            .collect();
        names.sort_unstable();
        assert_eq!(
            names,
            vec![
                "Elite Vanguard",
                "Giant Spider",
                "Grizzly Bears",
                "Llanowar Elves",
                "Serra Angel",
            ]
        );
    }

    #[test]
    fn bootstrap_hand_serra_angel_has_flying_and_vigilance() {
        let hand = bootstrap_hand("player-1");
        let serra = hand
            .iter()
            .find(|c| c.definition().name() == "Serra Angel")
            .unwrap();
        assert!(serra.definition().static_abilities().contains(&StaticAbility::Flying));
        assert!(serra
            .definition()
            .static_abilities()
            .contains(&StaticAbility::Vigilance));
    }

    #[test]
    fn bootstrap_hand_giant_spider_has_reach() {
        let hand = bootstrap_hand("player-1");
        let spider = hand
            .iter()
            .find(|c| c.definition().name() == "Giant Spider")
            .unwrap();
        assert!(spider
            .definition()
            .static_abilities()
            .contains(&StaticAbility::Reach));
    }
}
