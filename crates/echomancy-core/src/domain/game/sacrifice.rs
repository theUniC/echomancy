//! Sacrifice handler — a player sacrifices a permanent they control (CR 701.17).

use crate::domain::enums::GraveyardReason;
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::Game;

/// Handle the `Sacrifice` action.
///
/// # Rules
///
/// Per CR 701.17, to sacrifice a permanent means to move it from the
/// battlefield to its owner's graveyard. Only the controller of a permanent
/// can sacrifice it. Sacrifice cannot be prevented or regenerated.
///
/// # Errors
///
/// - `InvalidPlayerAction` if the permanent is not on the player's battlefield.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    permanent_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // Verify the permanent is on the player's battlefield.
    let player = game.player_state(player_id)?;
    let found = player
        .battlefield
        .iter()
        .any(|c| c.instance_id() == permanent_id);

    if !found {
        return Err(GameError::PermanentNotFound {
            permanent_id: crate::domain::types::CardInstanceId::new(permanent_id),
        });
    }

    game.move_permanent_to_graveyard(permanent_id, GraveyardReason::Sacrifice)
}

#[cfg(test)]
mod tests {
    use crate::domain::actions::Action;
    use crate::domain::game::test_helpers::{
        add_permanent_to_battlefield, make_creature_card, make_game_in_first_main,
    };
    use crate::domain::types::{CardInstanceId, PlayerId};

    #[test]
    fn sacrifice_moves_permanent_to_graveyard() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let bear = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, bear);

        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 0);

        game.apply(Action::Sacrifice {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("bear-1"),
        })
        .unwrap();

        assert_eq!(game.battlefield(&p1).unwrap().len(), 0);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    #[test]
    fn sacrifice_fails_for_permanent_not_on_battlefield() {
        let (mut game, p1, _p2) = make_game_in_first_main();

        let result = game.apply(Action::Sacrifice {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("nonexistent"),
        });

        assert!(result.is_err());
    }

    #[test]
    fn sacrifice_fails_for_opponents_permanent() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let goblin = make_creature_card("goblin-1", &p2, 1, 1);
        add_permanent_to_battlefield(&mut game, &p2, goblin);

        // P1 tries to sacrifice P2's creature.
        let result = game.apply(Action::Sacrifice {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("goblin-1"),
        });

        assert!(result.is_err());
    }
}
