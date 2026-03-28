//! PlayLand handler — play a land from hand to the battlefield.

use crate::domain::enums::ZoneName;
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::types::{CardInstanceId, PlayerId};

use super::Game;

/// Handle the `PlayLand` action.
///
/// # Rules (MTG 305)
///
/// 1. Only the current player can play a land.
/// 2. Can only be played during a main phase.
/// 3. The stack must be empty.
/// 4. Only one land per turn (unless a rule increases the limit).
/// 5. The card must be in the player's hand.
/// 6. The card must be a land.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` if the player is not the current player.
/// - `GameError::InvalidPlayLandStep` if not in a main phase.
/// - `GameError::LandLimitExceeded` if the player has already played a land this turn.
/// - `GameError::CardNotFoundInHand` if the card is not in the player's hand.
/// - `GameError::CardIsNotLand` if the card is not a land.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    card_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // 1. Must be the current player
    if player_id != game.current_player_id() {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "PLAY_LAND".to_owned(),
        });
    }

    // 2. Must be during a main phase
    if !game.turn_state.is_main_phase() {
        return Err(GameError::InvalidPlayLandStep);
    }

    // 3. The stack must be empty (CR 305.1 — sorcery-speed special action)
    if !game.stack().is_empty() {
        return Err(GameError::stack_not_empty(false));
    }

    // 4. One land per turn
    if game.turn_state.has_played_land() {
        return Err(GameError::LandLimitExceeded);
    }

    // 5. Card must be in hand — find it
    let card = {
        let player = game.player_state(player_id)?;
        let card = player
            .hand
            .iter()
            .find(|c| c.instance_id() == card_id)
            .cloned()
            .ok_or_else(|| GameError::CardNotFoundInHand {
                card_id: CardInstanceId::new(card_id),
                player_id: PlayerId::new(player_id),
            })?;
        card
    };

    // 6. Card must be a land
    if !card.definition().is_land() {
        return Err(GameError::CardIsNotLand {
            card_id: CardInstanceId::new(card_id),
        });
    }

    // Remove from hand
    {
        let player = game.player_state_mut(player_id)?;
        player.hand.retain(|c| c.instance_id() != card_id);
    }

    // Place on battlefield (enter_battlefield handles ETB effects)
    let events = game.enter_battlefield(card, player_id, ZoneName::Hand);

    // Record land played
    game.record_land_played();

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::game::test_helpers::{
        add_card_to_hand, make_creature_card, make_game_in_first_main, make_land_card,
        make_started_game,
    };
    use crate::domain::types::{CardInstanceId, PlayerId};

    #[test]
    fn cannot_play_land_when_stack_is_not_empty() {
        use crate::domain::entities::the_stack::{SpellOnStack, StackItem};
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        // Put a dummy spell on the stack so it is non-empty.
        let dummy_card = make_creature_card("dummy-1", &p1, 1, 1);
        game.push_stack(StackItem::Spell(SpellOnStack {
            card: dummy_card,
            controller_id: p1.clone(),
            targets: Vec::new(),
        }));

        let err = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("land-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::StackNotEmpty { .. }));
    }

    #[test]
    fn play_land_moves_card_from_hand_to_battlefield() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        game.apply(Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("land-1"),
        })
        .unwrap();

        assert!(game.hand(&p1).unwrap().is_empty());
        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
        assert_eq!(
            game.battlefield(&p1).unwrap()[0].instance_id(),
            "land-1"
        );
    }

    #[test]
    fn play_land_emits_zone_changed_event() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        let events = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("land-1"),
            })
            .unwrap();

        assert!(events.iter().any(|e| matches!(e, GameEvent::ZoneChanged { .. })));
    }

    #[test]
    fn play_land_increments_played_lands() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        assert_eq!(game.played_lands_this_turn(), 0);
        game.apply(Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("land-1"),
        })
        .unwrap();
        assert_eq!(game.played_lands_this_turn(), 1);
    }

    #[test]
    fn cannot_play_second_land_in_same_turn() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land1 = make_land_card("land-1", &p1);
        let land2 = make_land_card("land-2", &p1);
        add_card_to_hand(&mut game, &p1, land1);
        add_card_to_hand(&mut game, &p1, land2);

        game.apply(Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("land-1"),
        })
        .unwrap();

        let err = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("land-2"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::LandLimitExceeded));
    }

    #[test]
    fn cannot_play_land_outside_main_phase() {
        let (mut game, p1, _) = make_started_game();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        // Still in Untap step
        let err = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("land-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayLandStep));
    }

    #[test]
    fn cannot_play_land_that_is_not_in_hand() {
        let (mut game, p1, _) = make_game_in_first_main();

        let err = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("missing-card"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CardNotFoundInHand { .. }));
    }

    #[test]
    fn cannot_play_non_land_as_land() {
        let (mut game, p1, _) = make_game_in_first_main();
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_card_to_hand(&mut game, &p1, creature);

        let err = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("bear-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CardIsNotLand { .. }));
    }

    #[test]
    fn opponent_cannot_play_land_on_their_non_turn() {
        let (mut game, _, p2) = make_game_in_first_main();
        let land = make_land_card("land-p2", &p2);
        add_card_to_hand(&mut game, &p2, land);

        let err = game
            .apply(Action::PlayLand {
                player_id: PlayerId::new(&p2),
                card_id: CardInstanceId::new("land-p2"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn land_gets_permanent_state_on_battlefield() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        game.apply(Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("land-1"),
        })
        .unwrap();

        // Land should have permanent state (non-creature)
        let state = game.permanent_state("land-1").unwrap();
        assert!(!state.is_tapped());
        assert!(state.creature_state().is_none());
    }
}
