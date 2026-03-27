//! AdvanceStep handler — advance to the next step/phase.

use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::Game;

/// Handle the `AdvanceStep` action.
///
/// Only the current player can advance the step.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` if the player is not the current player.
pub(crate) fn handle(game: &mut Game, player_id: &str) -> Result<Vec<GameEvent>, GameError> {
    if player_id != game.current_player_id() {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "ADVANCE_STEP".to_owned(),
        });
    }

    let events = game.perform_step_advance();
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::enums::Step;
    use crate::domain::game::test_helpers::make_started_game;
    use crate::domain::types::PlayerId;

    #[test]
    fn advance_step_moves_from_untap_to_upkeep() {
        let (mut game, p1, _) = make_started_game();
        assert_eq!(game.current_step(), Step::Untap);
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::Upkeep);
    }

    #[test]
    fn advance_step_emits_step_started_event() {
        let (mut game, p1, _) = make_started_game();
        let events = game
            .apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        assert!(events.iter().any(|e| matches!(e, GameEvent::StepStarted { .. })));
    }

    #[test]
    fn wrong_player_cannot_advance_step() {
        let (mut game, _, p2) = make_started_game();
        let err = game
            .apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p2),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn advance_through_all_steps_progresses_correctly() {
        let (mut game, p1, p2) = make_started_game();
        let steps = [
            Step::Upkeep,
            Step::Draw,
            Step::FirstMain,
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::DeclareBlockers,
            Step::CombatDamage,
            Step::EndOfCombat,
            Step::SecondMain,
            Step::EndStep,
            Step::Cleanup,
        ];
        for expected in &steps {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
            assert_eq!(game.current_step(), *expected);
        }
        // After CLEANUP, wraps around to Untap for the next player
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::Untap);
        assert_eq!(game.current_player_id(), &p2);
    }

    #[test]
    fn turn_number_increments_after_both_players_complete_turns() {
        let (mut game, p1, p2) = make_started_game();
        // Advance p1 through all steps
        for _ in 0..12 {
            // 12 steps from Untap through Cleanup
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        // Now p2's turn (turn number stays at 1 until both complete)
        assert_eq!(game.current_player_id(), &p2);
        // Advance p2 through all steps
        for _ in 0..12 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        // After both players complete one full turn, back to p1 at turn 2
        assert_eq!(game.current_player_id(), &p1);
        assert_eq!(game.turn_number(), 2);
    }

    #[test]
    fn draw_step_draws_card_after_first_turn() {
        let mut game = crate::domain::game::Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        // p1: 10-card deck; p2: 8-card deck (just enough to draw on their turn)
        let p1_cards: Vec<_> = (0..10)
            .map(|i| crate::domain::game::test_helpers::make_land_card(&format!("p1c{i}"), "p1"))
            .collect();
        let p2_cards: Vec<_> = (0..8)
            .map(|i| crate::domain::game::test_helpers::make_land_card(&format!("p2c{i}"), "p2"))
            .collect();
        game.assign_deck("p1", p1_cards).unwrap();
        game.assign_deck("p2", p2_cards).unwrap();
        game.start("p1", Some(1)).unwrap();

        // After start: p1 has 7 cards (3 remain), p2 has 7 cards (1 remains).
        // Advance p1 through all steps of their first turn
        for _ in 0..12 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        // Now p2's turn. Skip through p2's turn (p2 draws their 1 remaining card on Draw step)
        for _ in 0..12 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        // Now p1's second turn. Advance to Upkeep and then Draw.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new("p1"),
        })
        .unwrap(); // Untap → Upkeep
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new("p1"),
        })
        .unwrap(); // Upkeep → Draw (auto-draws 1 card for p1)

        // p1 had 7 in hand + drew 1 = 8 total
        assert_eq!(game.hand("p1").unwrap().len(), 8);
    }
}
