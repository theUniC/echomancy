//! PassPriority handler — pass priority to the next player or resolve the stack.

use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::Game;

/// Handle the `PassPriority` action.
///
/// When a player passes priority:
/// - If both players have now passed in this priority window:
///   - If the stack is non-empty: resolve the top item.
///   - If the stack is empty: advance the step.
/// - Otherwise: give priority to the other player.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` if the player does not have priority.
pub(crate) fn handle(game: &mut Game, player_id: &str) -> Result<Vec<GameEvent>, GameError> {
    if !game.has_priority(player_id) {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "PASS_PRIORITY".to_owned(),
        });
    }

    game.record_passed_priority(player_id);

    let events = if game.both_players_have_passed() {
        if game.stack_has_items() {
            game.resolve_top_of_stack()
        } else {
            // Both passed with empty stack — advance step
            game.clear_passed_priority();
            game.perform_step_advance()
        }
    } else {
        // Give priority to the other player
        let opponent_id = game
            .players
            .iter()
            .find(|p| p.player_id.as_str() != player_id)
            .map(|p| p.player_id.as_str().to_owned())
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: player_id.into(),
            })?;
        game.assign_priority_to(&opponent_id)
    };

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    
    use crate::domain::game::test_helpers::{
        add_card_to_hand, make_creature_card, make_game_in_first_main,
    };
    use crate::domain::types::PlayerId;

    #[test]
    fn pass_priority_gives_priority_to_opponent() {
        let (mut game, p1, p2) = make_game_in_first_main();
        assert_eq!(game.priority_player_id(), Some(p1.as_str()));

        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.priority_player_id(), Some(p2.as_str()));
    }

    #[test]
    fn both_players_passing_with_empty_stack_advances_step() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let step_before = game.current_step();

        // p1 passes
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // p2 passes
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p2),
        })
        .unwrap();

        // Step should have advanced
        assert_ne!(game.current_step(), step_before);
    }

    #[test]
    fn both_players_passing_with_spell_on_stack_resolves_spell() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_card_to_hand(&mut game, &p1, creature);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: crate::domain::types::CardInstanceId::new("bear-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert_eq!(game.stack().len(), 1);

        // CR 117.3c: p1 (caster) retains priority after casting.
        // p1 passes first, then p2.
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // p2 passes — both have passed, spell resolves
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p2),
        })
        .unwrap();

        assert!(game.stack().is_empty());
        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
    }

    #[test]
    fn player_without_priority_cannot_pass() {
        let (mut game, _p1, p2) = make_game_in_first_main();
        // p1 has priority, not p2
        let err = game
            .apply(Action::PassPriority {
                player_id: PlayerId::new(&p2),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }
}
