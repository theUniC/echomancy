//! Integration tests for the application layer.
//!
//! These tests exercise the full command/query/snapshot stack from the outside
//! of the crate, using only the public API.

use echomancy_core::application::commands::{ApplyAction, CreateGame, JoinGame, StartGame};
use echomancy_core::application::errors::ApplicationError;
use echomancy_core::application::queries::{GetAllowedActions, GetGameState};
use echomancy_core::application::repository::GameRepository;
use echomancy_core::domain::actions::Action;
use echomancy_core::domain::enums::{CardType, GameLifecycleState, Step};
use echomancy_core::domain::types::PlayerId;
use echomancy_core::infrastructure::game_snapshot::{CardRegistry, create_game_snapshot};
use echomancy_core::infrastructure::in_memory_repo::InMemoryGameRepository;
use uuid::Uuid;

// ============================================================================
// Test card registry
// ============================================================================

struct SimpleRegistry;

impl CardRegistry for SimpleRegistry {
    fn card_name(&self, id: &str) -> String {
        match id {
            "forest" => "Forest".to_owned(),
            "plains" => "Plains".to_owned(),
            "grizzly-bears" => "Grizzly Bears".to_owned(),
            "elite-vanguard" => "Elite Vanguard".to_owned(),
            "giant-spider" => "Giant Spider".to_owned(),
            "serra-angel" => "Serra Angel".to_owned(),
            "llanowar-elves" => "Llanowar Elves".to_owned(),
            other => other.to_owned(),
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Create and start a 2-player game, returning (repo, game_id, p1_id, p2_id).
fn setup_started_game() -> (InMemoryGameRepository, String, String, String) {
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

/// Advance to FIRST_MAIN (UNTAP → UPKEEP → DRAW → FIRST_MAIN).
fn advance_to_first_main(repo: &mut InMemoryGameRepository, game_id: &str, player_id: &str) {
    for _ in 0..3 {
        ApplyAction::new(
            game_id,
            Action::AdvanceStep {
                player_id: PlayerId::new(player_id),
            },
        )
        .execute(repo)
        .unwrap();
    }
}

// ============================================================================
// Full workflow integration tests
// ============================================================================

#[test]
fn full_game_lifecycle_creates_starts_and_applies_actions() {
    let mut repo = InMemoryGameRepository::new();
    let game_id = uuid();
    let p1 = uuid();
    let p2 = uuid();

    // Create
    CreateGame::new(&game_id).execute(&mut repo).unwrap();
    assert!(repo.find_by_id(&game_id).is_some());

    // Join
    JoinGame::new(&game_id, &p1, "Alice").execute(&mut repo).unwrap();
    JoinGame::new(&game_id, &p2, "Bob").execute(&mut repo).unwrap();

    // Start
    StartGame::new(&game_id, &p1).execute(&mut repo).unwrap();

    let game = repo.find_by_id(&game_id).unwrap();
    assert_eq!(game.lifecycle(), GameLifecycleState::Started);
    assert_eq!(game.current_player_id(), p1.as_str());

    // Apply action
    let step_before = game.current_step();
    ApplyAction::new(
        &game_id,
        Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        },
    )
    .execute(&mut repo)
    .unwrap();

    let game = repo.find_by_id(&game_id).unwrap();
    assert_ne!(game.current_step(), step_before);
}

#[test]
fn get_game_state_query_reflects_game_state() {
    let (repo, game_id, p1, p2) = setup_started_game();

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();

    assert_eq!(state.game_id, game_id);
    assert_eq!(state.lifecycle_state, GameLifecycleState::Started);
    assert_eq!(state.current_player_id, p1);
    assert!(state.players.contains_key(&p1));
    assert!(state.players.contains_key(&p2));
    assert_eq!(state.players[&p1].life_total, 20);
    assert_eq!(state.players[&p2].life_total, 20);
}

#[test]
fn get_allowed_actions_returns_land_cards_in_first_main() {
    let (mut repo, game_id, p1, _) = setup_started_game();
    advance_to_first_main(&mut repo, &game_id, &p1);

    let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();

    // Bootstrap hand has 2 lands.
    assert_eq!(result.playable_lands.len(), 2);
}

#[test]
fn get_allowed_actions_empty_in_untap_step() {
    let (repo, game_id, p1, _) = setup_started_game();

    let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
    assert!(result.playable_lands.is_empty());
}

#[test]
fn play_land_removes_it_from_allowed_actions() {
    let (mut repo, game_id, p1, _) = setup_started_game();
    advance_to_first_main(&mut repo, &game_id, &p1);

    // Get a land card to play.
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

    ApplyAction::new(
        &game_id,
        Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: echomancy_core::domain::types::CardInstanceId::new(&land_id),
        },
    )
    .execute(&mut repo)
    .unwrap();

    let result = GetAllowedActions::new(&game_id, &p1).execute(&repo).unwrap();
    assert!(result.playable_lands.is_empty());
}

// ============================================================================
// GameSnapshot integration tests
// ============================================================================

#[test]
fn game_snapshot_shows_viewer_hand_hides_opponent_hand() {
    let (repo, game_id, p1, p2) = setup_started_game();

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();

    // Snapshot from p1's perspective.
    let snap = create_game_snapshot(&state, &p1, &SimpleRegistry).unwrap();

    assert_eq!(snap.viewer_player_id, p1);
    // p1 sees their own hand (7 cards).
    assert_eq!(snap.private_player_state.hand.len(), 7);
    // p1 sees p2 as opponent with hand_size but not hand cards.
    let opp = snap.opponent_states.iter().find(|o| o.player_id == p2).unwrap();
    assert_eq!(opp.hand_size, 7);
}

#[test]
fn game_snapshot_public_state_is_symmetric() {
    let (repo, game_id, p1, p2) = setup_started_game();

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();

    let snap1 = create_game_snapshot(&state, &p1, &SimpleRegistry).unwrap();
    let snap2 = create_game_snapshot(&state, &p2, &SimpleRegistry).unwrap();

    // Both players see the same public game state.
    assert_eq!(snap1.public_game_state, snap2.public_game_state);
}

#[test]
fn game_snapshot_phase_advances_correctly() {
    let (mut repo, game_id, p1, _) = setup_started_game();

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();
    let snap_before = create_game_snapshot(&state, &p1, &SimpleRegistry).unwrap();
    assert_eq!(snap_before.public_game_state.current_step, Step::Untap);
    assert_eq!(snap_before.public_game_state.current_phase, "Beginning");

    advance_to_first_main(&mut repo, &game_id, &p1);

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();
    let snap_after = create_game_snapshot(&state, &p1, &SimpleRegistry).unwrap();
    assert_eq!(snap_after.public_game_state.current_step, Step::FirstMain);
    assert_eq!(snap_after.public_game_state.current_phase, "Precombat Main");
}

#[test]
fn game_snapshot_priority_hints_reflect_game_state() {
    let (repo, game_id, p1, p2) = setup_started_game();

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();

    let snap_p1 = create_game_snapshot(&state, &p1, &SimpleRegistry).unwrap();
    let snap_p2 = create_game_snapshot(&state, &p2, &SimpleRegistry).unwrap();

    // p1 starts with priority.
    assert!(snap_p1.ui_hints.as_ref().unwrap().can_pass_priority);
    assert!(!snap_p2.ui_hints.as_ref().unwrap().can_pass_priority);
}

#[test]
fn game_snapshot_card_names_resolved_from_registry() {
    let (mut repo, game_id, p1, _) = setup_started_game();
    advance_to_first_main(&mut repo, &game_id, &p1);

    let state = GetGameState::new(&game_id).execute(&repo).unwrap();
    let snap = create_game_snapshot(&state, &p1, &SimpleRegistry).unwrap();

    // Every card in hand should have a non-empty name.
    for card in &snap.private_player_state.hand {
        assert!(!card.name.is_empty(), "Card name should not be empty");
    }
}

#[test]
fn error_handling_invalid_game_id_rejected_at_every_entry_point() {
    let mut repo = InMemoryGameRepository::new();

    let create_err = CreateGame::new("not-a-uuid")
        .execute(&mut repo)
        .unwrap_err();
    assert!(matches!(create_err, ApplicationError::InvalidGameId { .. }));

    let join_err = JoinGame::new("not-a-uuid", uuid(), "Alice")
        .execute(&mut repo)
        .unwrap_err();
    assert!(matches!(join_err, ApplicationError::InvalidGameId { .. }));

    let start_err = StartGame::new("not-a-uuid", uuid())
        .execute(&mut repo)
        .unwrap_err();
    assert!(matches!(start_err, ApplicationError::InvalidGameId { .. }));

    let apply_err = ApplyAction::new(
        "not-a-uuid",
        Action::AdvanceStep {
            player_id: PlayerId::new(uuid()),
        },
    )
    .execute(&mut repo)
    .unwrap_err();
    assert!(matches!(apply_err, ApplicationError::InvalidGameId { .. }));

    let get_state_err = GetGameState::new("not-a-uuid").execute(&repo).unwrap_err();
    assert!(matches!(get_state_err, ApplicationError::InvalidGameId { .. }));

    let get_actions_err = GetAllowedActions::new("not-a-uuid", uuid())
        .execute(&repo)
        .unwrap_err();
    assert!(matches!(
        get_actions_err,
        ApplicationError::InvalidGameId { .. }
    ));
}

#[test]
fn game_not_found_error_returned_when_game_missing() {
    let mut repo = InMemoryGameRepository::new();
    let nonexistent = uuid();

    let join_err = JoinGame::new(&nonexistent, uuid(), "Alice")
        .execute(&mut repo)
        .unwrap_err();
    assert!(matches!(join_err, ApplicationError::GameNotFound { .. }));

    let start_err = StartGame::new(&nonexistent, uuid())
        .execute(&mut repo)
        .unwrap_err();
    assert!(matches!(start_err, ApplicationError::GameNotFound { .. }));

    let apply_err = ApplyAction::new(
        &nonexistent,
        Action::AdvanceStep {
            player_id: PlayerId::new(uuid()),
        },
    )
    .execute(&mut repo)
    .unwrap_err();
    assert!(matches!(apply_err, ApplicationError::GameNotFound { .. }));

    let get_state_err = GetGameState::new(&nonexistent).execute(&repo).unwrap_err();
    assert!(matches!(get_state_err, ApplicationError::GameNotFound { .. }));

    let get_actions_err = GetAllowedActions::new(&nonexistent, uuid())
        .execute(&repo)
        .unwrap_err();
    assert!(matches!(
        get_actions_err,
        ApplicationError::GameNotFound { .. }
    ));
}
