//! DrawCard handler — a player draws one or more cards from their library.

use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::Game;

/// Handle the `DrawCard` action.
///
/// A player draws `amount` cards from the top of their library into their hand.
///
/// # Rules
///
/// 1. Only the current player (active player) may draw explicitly via this
///    action. (Automatic draw during the Draw step goes through
///    `draw_cards_internal` directly.)
/// 2. If the library is empty, the
///    "attempted draw from empty library" SBA flag is set.
///    State-based actions then end the game at the next check.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` if the player is not the current player.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    amount: u32,
) -> Result<Vec<GameEvent>, GameError> {
    // Only the current player (or a player with permission) may draw.
    if game.current_player_id() != player_id {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "DRAW_CARD".to_owned(),
        });
    }

    let events = game.draw_cards_internal(player_id, amount);

    // Run SBA after drawing (empty library check)
    let sba_events = game.perform_state_based_actions();

    Ok(events.into_iter().chain(sba_events).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::enums::GameLifecycleState;
    use crate::domain::game::test_helpers::{
        make_land_card, make_started_game,
    };
    use crate::domain::types::PlayerId;

    #[test]
    fn draw_card_moves_card_from_library_to_hand() {
        let mut game = crate::domain::game::Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let deck: Vec<_> = (0..10)
            .map(|i| make_land_card(&format!("l-{i}"), "p1"))
            .collect();
        game.assign_deck("p1", deck).unwrap();
        game.start("p1", Some(42)).unwrap();

        let library_before = game.library_count("p1").unwrap();
        let hand_before = game.hand("p1").unwrap().len();

        game.apply(Action::DrawCard {
            player_id: PlayerId::new("p1"),
            amount: 1,
        })
        .unwrap();

        assert_eq!(game.hand("p1").unwrap().len(), hand_before + 1);
        assert_eq!(game.library_count("p1").unwrap(), library_before - 1);
    }

    #[test]
    fn draw_multiple_cards_moves_correct_count() {
        let mut game = crate::domain::game::Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let deck: Vec<_> = (0..10)
            .map(|i| make_land_card(&format!("l-{i}"), "p1"))
            .collect();
        game.assign_deck("p1", deck).unwrap();
        game.start("p1", Some(42)).unwrap();

        let hand_before = game.hand("p1").unwrap().len();

        game.apply(Action::DrawCard {
            player_id: PlayerId::new("p1"),
            amount: 3,
        })
        .unwrap();

        assert_eq!(game.hand("p1").unwrap().len(), hand_before + 3);
    }

    #[test]
    fn draw_from_empty_library_triggers_loss() {
        let (mut game, p1, _) = make_started_game();
        // No deck assigned — library is empty

        // Drawing from an empty library should set the SBA flag and resolve to a loss.
        game.apply(Action::DrawCard {
            player_id: PlayerId::new(&p1),
            amount: 1,
        })
        .unwrap();

        // After SBA, the game should be finished (p1 loses).
        assert_eq!(game.lifecycle(), GameLifecycleState::Finished);
        assert!(game.outcome().is_some());
    }

    #[test]
    fn non_active_player_cannot_draw() {
        let (mut game, _, p2) = make_started_game();

        let err = game
            .apply(Action::DrawCard {
                player_id: PlayerId::new(&p2),
                amount: 1,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    // TR5: CardDrawn event is emitted on each card draw
    #[test]
    fn draw_card_emits_card_drawn_event() {
        use crate::domain::events::GameEvent;

        let mut game = crate::domain::game::Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let deck: Vec<_> = (0..10)
            .map(|i| make_land_card(&format!("l-{i}"), "p1"))
            .collect();
        game.assign_deck("p1", deck).unwrap();
        game.start("p1", Some(42)).unwrap();

        let events = game
            .apply(Action::DrawCard {
                player_id: PlayerId::new("p1"),
                amount: 1,
            })
            .unwrap();

        let card_drawn_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
            .collect();
        assert_eq!(
            card_drawn_events.len(),
            1,
            "Expected 1 CardDrawn event for drawing 1 card"
        );
    }

    #[test]
    fn draw_three_cards_emits_three_card_drawn_events() {
        use crate::domain::events::GameEvent;

        let mut game = crate::domain::game::Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let deck: Vec<_> = (0..10)
            .map(|i| make_land_card(&format!("l-{i}"), "p1"))
            .collect();
        game.assign_deck("p1", deck).unwrap();
        game.start("p1", Some(42)).unwrap();

        let events = game
            .apply(Action::DrawCard {
                player_id: PlayerId::new("p1"),
                amount: 3,
            })
            .unwrap();

        let card_drawn_count = events
            .iter()
            .filter(|e| matches!(e, GameEvent::CardDrawn { .. }))
            .count();
        assert_eq!(card_drawn_count, 3, "Expected 3 CardDrawn events for drawing 3 cards");
    }
}
