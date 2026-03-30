//! Mulligan command handlers.
//!
//! Implements the Vancouver Mulligan (CR 103.4–103.5):
//! - Players may mulligan any number of times (shuffle hand back, draw 7).
//! - After all players keep, each player who mulliganed puts cards on the bottom
//!   equal to the number of mulligans taken.
//! - P2 always keeps immediately (no bot decision needed).
//!
//! All three handlers follow the same pattern:
//! 1. Validate — return `Err` if preconditions are not met.
//! 2. Mutate — update game state in place.
//! 3. Return events.

use rand::SeedableRng;
use rand::seq::SliceRandom;

use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::types::PlayerId;

use super::Game;

// ============================================================================
// handle_keep
// ============================================================================

/// Handle `Action::MulliganKeep`.
///
/// Sets `has_kept = true` for the player.  If they took N > 0 mulligans,
/// sets their `cards_to_put_back` to N.  If all players are now complete
/// (every player has kept and has no remaining put-back cards), transitions
/// the game to Turn 1 by clearing `mulligan_state`.
pub(crate) fn handle_keep(
    game: &mut Game,
    player_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // Validate: must be in mulligan phase
    if game.mulligan_state.is_none() {
        return Err(GameError::NotInMulliganPhase);
    }

    // Validate: player exists
    let _ = game.player_state(player_id)?;

    // Re-borrow mulligan_state mutably (player_state() borrows game immutably so
    // we need to re-access after the check)
    let mulligan_state = game
        .mulligan_state
        .as_mut()
        .expect("mulligan state checked above");

    // Validate: player has not already kept
    let status = mulligan_state
        .statuses
        .get(player_id)
        .ok_or_else(|| GameError::PlayerNotFound { player_id: PlayerId::new(player_id) })?;

    if status.has_kept {
        return Err(GameError::PlayerAlreadyKept { player_id: PlayerId::new(player_id) });
    }

    // Record the keep
    let status = mulligan_state
        .statuses
        .get_mut(player_id)
        .expect("player exists");
    status.record_keep();

    tracing::debug!(player_id, cards_to_put_back = status.cards_to_put_back, "MulliganKeep");

    // Check if all players are done
    if game.mulligan_state.as_ref().expect("exists").all_complete() {
        tracing::debug!("All players completed mulligan — transitioning to Turn 1");
        game.mulligan_state = None;
    }

    Ok(vec![])
}

// ============================================================================
// handle_redraw
// ============================================================================

/// Handle `Action::MulliganRedraw`.
///
/// Moves all cards from the player's hand back to the library, shuffles the
/// library, draws 7 new cards, and increments the mulligan count.
pub(crate) fn handle_redraw(
    game: &mut Game,
    player_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // Validate: must be in mulligan phase
    let _ = game
        .mulligan_state
        .as_ref()
        .ok_or(GameError::NotInMulliganPhase)?;

    // Validate: player exists
    let _ = game.player_state(player_id)?;

    // Validate: player has not already kept
    let mulligan_state = game.mulligan_state.as_ref().expect("checked above");
    let status = mulligan_state
        .statuses
        .get(player_id)
        .ok_or_else(|| GameError::PlayerNotFound { player_id: PlayerId::new(player_id) })?;

    if status.has_kept {
        return Err(GameError::PlayerAlreadyKept { player_id: PlayerId::new(player_id) });
    }

    let mulligan_count_before = status.mulligan_count;
    tracing::debug!(player_id, mulligan_count = mulligan_count_before + 1, "MulliganRedraw");

    // Move all hand cards back to library
    let player = game.player_state_mut(player_id)?;
    let hand_cards: Vec<_> = player.hand.drain(..).collect();
    player.library.extend(hand_cards);

    // Shuffle the library using OS entropy (matches production behaviour)
    let player = game.player_state_mut(player_id)?;
    let mut rng = rand::rngs::SmallRng::from_os_rng();
    player.library.shuffle(&mut rng);

    // Draw 7 new cards
    game.draw_cards_internal(player_id, 7);

    // Increment mulligan count
    let mulligan_state = game.mulligan_state.as_mut().expect("checked above");
    let status = mulligan_state
        .statuses
        .get_mut(player_id)
        .expect("player exists");
    status.record_mulligan();

    Ok(vec![])
}

// ============================================================================
// handle_put_card_on_bottom
// ============================================================================

/// Handle `Action::PutCardOnBottom`.
///
/// Removes the specified card from the player's hand and appends it to the
/// bottom of their library (index 0 is top; bottom = end of the Vec… wait,
/// let's look at the actual representation).
///
/// Library convention: index 0 = top (next to be drawn), last index = bottom.
/// Putting a card "on the bottom" appends it to the end of the Vec.
///
/// After the card is placed, decrements `cards_to_put_back`.  If all players
/// are now complete, transitions the game to Turn 1.
pub(crate) fn handle_put_card_on_bottom(
    game: &mut Game,
    player_id: &str,
    card_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // Validate: must be in mulligan phase
    let _ = game
        .mulligan_state
        .as_ref()
        .ok_or(GameError::NotInMulliganPhase)?;

    // Validate: player exists
    let _ = game.player_state(player_id)?;

    // Validate: player has kept
    let mulligan_state = game.mulligan_state.as_ref().expect("checked above");
    let status = mulligan_state
        .statuses
        .get(player_id)
        .ok_or_else(|| GameError::PlayerNotFound { player_id: PlayerId::new(player_id) })?;

    if !status.has_kept {
        return Err(GameError::PlayerHasNotKeptYet { player_id: PlayerId::new(player_id) });
    }

    // Validate: there are cards to put back
    if status.cards_to_put_back == 0 {
        return Err(GameError::NoPutBackRequired { player_id: PlayerId::new(player_id) });
    }

    // Validate: card is in the player's hand
    let player = game.player_state(player_id)?;
    let card_pos = player
        .hand
        .iter()
        .position(|c| c.instance_id() == card_id)
        .ok_or_else(|| {
            use crate::domain::types::CardInstanceId;
            GameError::CardNotFoundInHand {
                card_id: CardInstanceId::new(card_id),
                player_id: PlayerId::new(player_id),
            }
        })?;

    tracing::debug!(player_id, card_id, "PutCardOnBottom");

    // Remove from hand and put on bottom of library
    let player = game.player_state_mut(player_id)?;
    let card = player.hand.remove(card_pos);
    player.library.push(card); // push = append to end = bottom

    // Decrement put-back counter
    let mulligan_state = game.mulligan_state.as_mut().expect("checked above");
    let status = mulligan_state
        .statuses
        .get_mut(player_id)
        .expect("player exists");
    status.record_put_back();

    // Check if all players are done
    if game.mulligan_state.as_ref().expect("exists").all_complete() {
        tracing::debug!("All players completed put-back — transitioning to Turn 1");
        game.mulligan_state = None;
    }

    Ok(vec![])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::game::test_helpers::make_land_card;
    use crate::domain::types::{CardInstanceId, PlayerId};

    // -------------------------------------------------------------------------
    // Helper: create a game in the mulligan phase
    // -------------------------------------------------------------------------

    /// Create a 2-player game that is in the mulligan phase.
    ///
    /// Both players have been assigned 20-card decks and `start_with_mulligan`
    /// has been called with p1 as the starting player. P2 keeps immediately.
    fn make_mulligan_game() -> (Game, String, String) {
        let mut game = Game::create("test-mulligan");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();

        let p1_deck: Vec<_> = (0..20)
            .map(|i| make_land_card(&format!("p1-card-{i}"), "p1"))
            .collect();
        let p2_deck: Vec<_> = (0..20)
            .map(|i| make_land_card(&format!("p2-card-{i}"), "p2"))
            .collect();
        game.assign_deck("p1", p1_deck).unwrap();
        game.assign_deck("p2", p2_deck).unwrap();

        game.start_with_mulligan("p1", Some(42)).unwrap();

        ("p1".to_owned(), "p2".to_owned());
        (game, "p1".to_owned(), "p2".to_owned())
    }

    // -------------------------------------------------------------------------
    // keep_with_zero_mulligans_completes_phase
    // -------------------------------------------------------------------------

    #[test]
    fn keep_with_zero_mulligans_completes_phase() {
        let (mut game, p1, _p2) = make_mulligan_game();

        assert!(game.is_in_mulligan());

        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Phase should be over now (p2 already kept during start_with_mulligan)
        assert!(!game.is_in_mulligan());
    }

    // -------------------------------------------------------------------------
    // mulligan_redraws_seven_cards
    // -------------------------------------------------------------------------

    #[test]
    fn mulligan_redraws_seven_cards() {
        let (mut game, p1, _p2) = make_mulligan_game();

        // P1 starts with 7 cards in hand
        assert_eq!(game.hand(&p1).unwrap().len(), 7);

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Still 7 cards after redraw (library had 13 remaining, shuffle + draw 7)
        assert_eq!(game.hand(&p1).unwrap().len(), 7);
    }

    // -------------------------------------------------------------------------
    // mulligan_increments_count
    // -------------------------------------------------------------------------

    #[test]
    fn mulligan_increments_count() {
        let (mut game, p1, _p2) = make_mulligan_game();

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        let count = game
            .mulligan_state
            .as_ref()
            .unwrap()
            .mulligan_count(&p1);
        assert_eq!(count, 1);

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        let count = game
            .mulligan_state
            .as_ref()
            .unwrap()
            .mulligan_count(&p1);
        assert_eq!(count, 2);
    }

    // -------------------------------------------------------------------------
    // keep_after_mulligan_requires_put_back
    // -------------------------------------------------------------------------

    #[test]
    fn keep_after_mulligan_requires_put_back() {
        let (mut game, p1, _p2) = make_mulligan_game();

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Game should still be in mulligan phase (1 card to put back)
        assert!(game.is_in_mulligan());

        let cards_to_put_back = game
            .mulligan_state
            .as_ref()
            .unwrap()
            .cards_to_put_back(&p1);
        assert_eq!(cards_to_put_back, 1);
    }

    // -------------------------------------------------------------------------
    // put_card_on_bottom_removes_from_hand
    // -------------------------------------------------------------------------

    #[test]
    fn put_card_on_bottom_removes_from_hand() {
        let (mut game, p1, _p2) = make_mulligan_game();

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        let hand_before = game.hand(&p1).unwrap().len();
        let card_id = game
            .hand(&p1)
            .unwrap()
            .first()
            .unwrap()
            .instance_id()
            .to_owned();

        game.apply(Action::PutCardOnBottom {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&card_id),
        })
        .unwrap();

        let hand_after = game.hand(&p1).unwrap().len();
        assert_eq!(hand_after, hand_before - 1);

        // Card should not be in hand anymore
        let in_hand = game
            .hand(&p1)
            .unwrap()
            .iter()
            .any(|c| c.instance_id() == card_id);
        assert!(!in_hand);
    }

    // -------------------------------------------------------------------------
    // put_back_all_cards_completes_phase
    // -------------------------------------------------------------------------

    #[test]
    fn put_back_all_cards_completes_phase() {
        let (mut game, p1, _p2) = make_mulligan_game();

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // 2 mulligans taken → 2 cards to put back
        assert!(game.is_in_mulligan());

        let card_id_1 = game.hand(&p1).unwrap()[0].instance_id().to_owned();
        game.apply(Action::PutCardOnBottom {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&card_id_1),
        })
        .unwrap();

        // Still in mulligan (1 more to go)
        assert!(game.is_in_mulligan());

        let card_id_2 = game.hand(&p1).unwrap()[0].instance_id().to_owned();
        game.apply(Action::PutCardOnBottom {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&card_id_2),
        })
        .unwrap();

        // Phase complete
        assert!(!game.is_in_mulligan());
    }

    // -------------------------------------------------------------------------
    // cannot_mulligan_after_keeping
    // -------------------------------------------------------------------------

    #[test]
    fn cannot_mulligan_after_keeping() {
        let (mut game, p1, _p2) = make_mulligan_game();

        // Keep immediately
        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Now try to mulligan — game is no longer in mulligan phase
        let result = game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        });

        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // cannot_put_back_card_not_in_hand
    // -------------------------------------------------------------------------

    #[test]
    fn cannot_put_back_card_not_in_hand() {
        let (mut game, p1, _p2) = make_mulligan_game();

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        let result = game.apply(Action::PutCardOnBottom {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("nonexistent-card"),
        });

        assert!(
            matches!(result, Err(GameError::CardNotFoundInHand { .. })),
            "Expected CardNotFoundInHand, got: {result:?}"
        );
    }

    // -------------------------------------------------------------------------
    // p2_auto_keeps_during_start_with_mulligan
    // -------------------------------------------------------------------------

    #[test]
    fn p2_auto_keeps_during_start_with_mulligan() {
        let (game, _p1, p2) = make_mulligan_game();

        let state = game.mulligan_state.as_ref().unwrap();
        assert!(state.has_kept(&p2));
        assert_eq!(state.mulligan_count(&p2), 0);
        assert_eq!(state.cards_to_put_back(&p2), 0);
    }

    // -------------------------------------------------------------------------
    // mulligan_seven_times_results_in_empty_hand
    // -------------------------------------------------------------------------

    #[test]
    fn mulligan_seven_times_results_in_empty_hand() {
        let mut game = Game::create("test-big-mulligan");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();

        // Need a large enough deck to support 7 mulligans (7 draws each time)
        let p1_deck: Vec<_> = (0..60)
            .map(|i| make_land_card(&format!("p1-card-{i}"), "p1"))
            .collect();
        let p2_deck: Vec<_> = (0..20)
            .map(|i| make_land_card(&format!("p2-card-{i}"), "p2"))
            .collect();
        game.assign_deck("p1", p1_deck).unwrap();
        game.assign_deck("p2", p2_deck).unwrap();
        game.start_with_mulligan("p1", Some(1)).unwrap();

        // Mulligan 7 times
        for _ in 0..7 {
            game.apply(Action::MulliganRedraw {
                player_id: PlayerId::new("p1"),
            })
            .unwrap();
        }

        // Keep
        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new("p1"),
        })
        .unwrap();

        // 7 cards to put back
        assert_eq!(
            game.mulligan_state.as_ref().unwrap().cards_to_put_back("p1"),
            7
        );

        // Put all 7 back
        let card_ids: Vec<String> = game
            .hand("p1")
            .unwrap()
            .iter()
            .map(|c| c.instance_id().to_owned())
            .collect();

        for card_id in card_ids {
            game.apply(Action::PutCardOnBottom {
                player_id: PlayerId::new("p1"),
                card_id: CardInstanceId::new(&card_id),
            })
            .unwrap();
        }

        // Phase should be complete and hand should be empty
        assert!(!game.is_in_mulligan());
        assert_eq!(game.hand("p1").unwrap().len(), 0);
    }

    // -------------------------------------------------------------------------
    // cannot_keep_twice
    // -------------------------------------------------------------------------

    #[test]
    fn cannot_keep_twice() {
        let mut game = Game::create("test-double-keep");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();

        let p1_deck: Vec<_> = (0..20)
            .map(|i| make_land_card(&format!("p1-{i}"), "p1"))
            .collect();
        let p2_deck: Vec<_> = (0..20)
            .map(|i| make_land_card(&format!("p2-{i}"), "p2"))
            .collect();
        game.assign_deck("p1", p1_deck).unwrap();
        game.assign_deck("p2", p2_deck).unwrap();
        game.start_with_mulligan("p1", Some(42)).unwrap();

        // P1 mulligans once then keeps
        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new("p1"),
        })
        .unwrap();
        game.apply(Action::MulliganKeep {
            player_id: PlayerId::new("p1"),
        })
        .unwrap();

        // Trying to keep again should fail (player has already kept)
        let result = game.apply(Action::MulliganKeep {
            player_id: PlayerId::new("p1"),
        });

        // Either NotInMulliganPhase (if 0 put-backs left) or PlayerAlreadyKept
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // put_back_without_keeping_first_returns_error
    // -------------------------------------------------------------------------

    #[test]
    fn put_back_without_keeping_first_returns_error() {
        let (mut game, p1, _p2) = make_mulligan_game();

        game.apply(Action::MulliganRedraw {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Try to put a card back without having kept first
        let card_id = game.hand(&p1).unwrap()[0].instance_id().to_owned();
        let result = game.apply(Action::PutCardOnBottom {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&card_id),
        });

        assert!(
            matches!(result, Err(GameError::PlayerHasNotKeptYet { .. })),
            "Expected PlayerHasNotKeptYet, got: {result:?}"
        );
    }
}
