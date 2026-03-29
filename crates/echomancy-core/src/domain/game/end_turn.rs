//! EndTurn handler — express intent to auto-pass through end of turn.

use crate::domain::enums::Step;
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::Game;

/// Handle the `EndTurn` action.
///
/// END_TURN is a player shortcut that records "auto-pass" intent.
/// The engine then advances through steps until the turn naturally ends.
///
/// # Rules
///
/// - Only the current player can end their turn.
/// - Cannot end turn from CLEANUP step.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` if the player is not the current player.
/// - `GameError::InvalidEndTurn` if the current step is CLEANUP.
pub(crate) fn handle(game: &mut Game, player_id: &str) -> Result<Vec<GameEvent>, GameError> {
    if player_id != game.current_player_id() {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "END_TURN".to_owned(),
        });
    }

    if game.current_step() == Step::Cleanup {
        return Err(GameError::InvalidEndTurn);
    }

    // Record auto-pass intent
    game.set_auto_pass(player_id);

    // Trigger auto-pass processing
    let events = game.process_auto_pass();
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::enums::Step;
    use crate::domain::game::test_helpers::{make_game_in_first_main, make_started_game};
    use crate::domain::types::PlayerId;

    #[test]
    fn end_turn_from_first_main_advances_to_next_player_untap() {
        let (mut game, p1, p2) = make_game_in_first_main();
        assert_eq!(game.current_step(), Step::FirstMain);

        game.apply(Action::EndTurn {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // After EndTurn, the game advances to the next player's FirstMain
        assert_eq!(game.current_player_id(), &p2);
        assert_eq!(game.current_step(), Step::FirstMain);
    }

    #[test]
    fn end_turn_from_cleanup_returns_error() {
        let (mut game, p1, _) = make_started_game();
        // Advance to CLEANUP (13 steps total now, index 12 = Cleanup)
        for _ in 0..12 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::Cleanup);

        let err = game
            .apply(Action::EndTurn {
                player_id: PlayerId::new(&p1),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidEndTurn));
    }

    #[test]
    fn wrong_player_cannot_end_turn() {
        let (mut game, _, p2) = make_started_game();
        let err = game
            .apply(Action::EndTurn {
                player_id: PlayerId::new(&p2),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }
}
