//! HasPriority specification — checks if a player currently holds priority.
//!
//! In Magic, the player with priority may cast spells, activate abilities,
//! or pass priority. This is the most fundamental action-window gate.

use crate::domain::errors::GameError;
use crate::domain::types::PlayerId;

/// Context required to evaluate the `HasPriority` specification.
///
/// Uses references to avoid unnecessary cloning.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct HasPriorityCtx<'a> {
    /// The ID of the player who currently holds priority (from game state).
    pub(crate) priority_player_id: &'a str,
    /// The ID of the player whose action window is being evaluated.
    pub(crate) player_id: &'a str,
}

/// Returns `Ok(())` if the player currently has priority.
///
/// Returns `Err(GameError::InvalidPlayerAction)` if the player does not
/// hold priority and therefore cannot take priority-based actions.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_satisfied(ctx: &HasPriorityCtx<'_>) -> Result<(), GameError> {
    if ctx.player_id == ctx.priority_player_id {
        Ok(())
    } else {
        Err(GameError::InvalidPlayerAction {
            player_id: PlayerId::new(ctx.player_id),
            action: "priority action".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(priority: &'a str, player: &'a str) -> HasPriorityCtx<'a> {
        HasPriorityCtx {
            priority_player_id: priority,
            player_id: player,
        }
    }

    #[test]
    fn player_with_priority_satisfies_spec() {
        let result = is_satisfied(&ctx("p1", "p1"));
        assert!(result.is_ok());
    }

    #[test]
    fn player_without_priority_does_not_satisfy_spec() {
        let result = is_satisfied(&ctx("p1", "p2"));
        assert!(result.is_err());
    }

    #[test]
    fn error_variant_is_invalid_player_action() {
        let err = is_satisfied(&ctx("p1", "p2")).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    #[test]
    fn error_contains_the_requesting_player_id() {
        let err = is_satisfied(&ctx("p1", "p2")).unwrap_err();
        if let GameError::InvalidPlayerAction { player_id, .. } = err {
            assert_eq!(player_id.as_str(), "p2");
        } else {
            panic!("expected InvalidPlayerAction");
        }
    }
}
