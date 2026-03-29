//! CanPlayLand specification — checks if a player can play a land.
//!
//! A player may play a land if:
//! 1. They are the current player (active player, their turn).
//! 2. They have not yet played a land this turn.
//! 3. The current step is a main phase (FirstMain or SecondMain).
//! 4. The stack is empty (CR 305.2).
//!
//! Note: this specification does not check priority (use `HasPriority` for
//! that) and does not check that the player has a land in hand (that check
//! is the responsibility of the action handler).
//!
//! Mirrors the TypeScript `CanPlayLand` class from
//! `game/specifications/CanPlayLand.ts`.

use crate::domain::enums::Step;
use crate::domain::errors::GameError;
use crate::domain::types::PlayerId;

/// Context required to evaluate the `CanPlayLand` specification.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CanPlayLandCtx<'a> {
    /// The ID of the player whose turn it currently is.
    pub(crate) current_player_id: &'a str,
    /// The ID of the player attempting to play a land.
    pub(crate) player_id: &'a str,
    /// Number of lands already played this turn (normally 0 or 1).
    pub(crate) played_lands_this_turn: u32,
    /// The current step in the turn sequence.
    pub(crate) current_step: Step,
    /// Whether the stack is currently empty (CR 305.2).
    pub(crate) stack_is_empty: bool,
}

/// Returns `Ok(())` if the player may play a land at this moment.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` — player is not the active player.
/// - `GameError::LandLimitExceeded` — the turn's land-play limit is already met.
/// - `GameError::InvalidPlayLandStep` — the current step is not a main phase.
/// - `GameError::StackNotEmpty` — a spell or ability is on the stack (CR 305.2).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_satisfied(ctx: &CanPlayLandCtx<'_>) -> Result<(), GameError> {
    // 1. Must be the active player.
    if ctx.player_id != ctx.current_player_id {
        return Err(GameError::InvalidPlayerAction {
            player_id: PlayerId::new(ctx.player_id),
            action: "PLAY_LAND".to_owned(),
        });
    }

    // 2. Must not have exceeded the per-turn land limit.
    if ctx.played_lands_this_turn > 0 {
        return Err(GameError::LandLimitExceeded);
    }

    // 3. Must be a main phase.
    match ctx.current_step {
        Step::FirstMain | Step::SecondMain => {}
        Step::Untap
        | Step::Upkeep
        | Step::Draw
        | Step::BeginningOfCombat
        | Step::DeclareAttackers
        | Step::DeclareBlockers
        | Step::FirstStrikeDamage
        | Step::CombatDamage
        | Step::EndOfCombat
        | Step::EndStep
        | Step::Cleanup => return Err(GameError::InvalidPlayLandStep),
    }

    // 4. Stack must be empty (CR 305.2).
    if !ctx.stack_is_empty {
        return Err(GameError::stack_not_empty(false));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn main_ctx<'a>(player: &'a str, current: &'a str, lands_played: u32) -> CanPlayLandCtx<'a> {
        CanPlayLandCtx {
            current_player_id: current,
            player_id: player,
            played_lands_this_turn: lands_played,
            current_step: Step::FirstMain,
            stack_is_empty: true,
        }
    }

    // --- happy paths --------------------------------------------------------

    #[test]
    fn active_player_can_play_land_in_first_main() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::FirstMain,
            stack_is_empty: true,
        };
        assert!(is_satisfied(&ctx).is_ok());
    }

    #[test]
    fn active_player_can_play_land_in_second_main() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::SecondMain,
            stack_is_empty: true,
        };
        assert!(is_satisfied(&ctx).is_ok());
    }

    // --- not active player --------------------------------------------------

    #[test]
    fn non_active_player_cannot_play_land() {
        let ctx = main_ctx("p2", "p1", 0);
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn error_contains_requesting_player_id_when_not_active() {
        let ctx = main_ctx("p2", "p1", 0);
        if let GameError::InvalidPlayerAction { player_id, .. } = is_satisfied(&ctx).unwrap_err() {
            assert_eq!(player_id.as_str(), "p2");
        } else {
            panic!("expected InvalidPlayerAction");
        }
    }

    // --- land limit ---------------------------------------------------------

    #[test]
    fn error_when_land_already_played_this_turn() {
        let ctx = main_ctx("p1", "p1", 1);
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::LandLimitExceeded));
    }

    // --- wrong step ---------------------------------------------------------

    #[test]
    fn cannot_play_land_during_draw_step() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::Draw,
            stack_is_empty: true,
        };
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayLandStep));
    }

    #[test]
    fn cannot_play_land_during_beginning_of_combat() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::BeginningOfCombat,
            stack_is_empty: true,
        };
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayLandStep));
    }

    #[test]
    fn cannot_play_land_during_end_step() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::EndStep,
            stack_is_empty: true,
        };
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayLandStep));
    }

    // --- stack not empty (CR 305.2) -----------------------------------------

    #[test]
    fn cannot_play_land_when_stack_is_not_empty() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::FirstMain,
            stack_is_empty: false,
        };
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::StackNotEmpty { .. }));
    }

    #[test]
    fn cannot_play_land_when_stack_not_empty_in_second_main() {
        let ctx = CanPlayLandCtx {
            current_player_id: "p1",
            player_id: "p1",
            played_lands_this_turn: 0,
            current_step: Step::SecondMain,
            stack_is_empty: false,
        };
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::StackNotEmpty { .. }));
    }

    // --- priority checked separately ----------------------------------------

    #[test]
    fn active_player_but_no_lands_played_zero_works() {
        // Land limit of 0 played means allowed (when all other checks pass).
        let ctx = main_ctx("p1", "p1", 0);
        assert!(is_satisfied(&ctx).is_ok());
    }
}
