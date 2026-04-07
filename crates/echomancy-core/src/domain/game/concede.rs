//! Concede handler — a player concedes and loses immediately (CR 104.3a).
//!
//! Conceding is a special action that does not use the stack. The conceding
//! player loses and the opponent wins immediately, regardless of game state.

use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::{Game, GameEndReason, GameOutcome};

/// Handle the `Concede` action.
///
/// # Rules (CR 104.3a)
///
/// A player may concede at any time. This is a special action that does not
/// use the stack. The conceding player loses immediately; the other player wins
/// with reason `Concession`.
///
/// # Errors
///
/// - `GameError::PlayerNotFound` if `player_id` is not in the game.
pub(crate) fn handle(game: &mut Game, player_id: &str) -> Result<Vec<GameEvent>, GameError> {
    // Validate the player exists.
    game.player_state(player_id)?;

    // Find the opponent (first player that is NOT the conceding player).
    let winner_id = game
        .players
        .iter()
        .find(|p| p.player_id.as_str() != player_id)
        .map(|p| p.player_id.clone())
        .ok_or(GameError::PlayerNotFound {
            player_id: crate::domain::types::PlayerId::new(player_id),
        })?;

    game.outcome = Some(GameOutcome::Win {
        winner_id,
        reason: GameEndReason::Concession,
    });
    game.lifecycle = crate::domain::enums::GameLifecycleState::Finished;

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::enums::GameLifecycleState;
    use crate::domain::game::test_helpers::make_started_game;
    use crate::domain::types::PlayerId;

    #[test]
    fn concede_finishes_game_with_opponent_winning() {
        let (mut game, p1, p2) = make_started_game();

        game.apply(Action::Concede {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.lifecycle(), GameLifecycleState::Finished);
        assert_eq!(
            game.outcome(),
            Some(&GameOutcome::Win {
                winner_id: PlayerId::new(&p2),
                reason: GameEndReason::Concession,
            })
        );
    }

    #[test]
    fn opponent_can_also_concede() {
        let (mut game, p1, p2) = make_started_game();

        game.apply(Action::Concede {
            player_id: PlayerId::new(&p2),
        })
        .unwrap();

        assert_eq!(game.lifecycle(), GameLifecycleState::Finished);
        assert_eq!(
            game.outcome(),
            Some(&GameOutcome::Win {
                winner_id: PlayerId::new(&p1),
                reason: GameEndReason::Concession,
            })
        );
    }

    #[test]
    fn concede_is_valid_regardless_of_whose_turn_it_is() {
        // p2 can concede even though it's p1's turn
        let (mut game, _p1, p2) = make_started_game();

        let result = game.apply(Action::Concede {
            player_id: PlayerId::new(&p2),
        });

        assert!(result.is_ok());
        assert_eq!(game.lifecycle(), GameLifecycleState::Finished);
    }
}
