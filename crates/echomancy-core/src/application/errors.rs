//! Application-level errors.
//!
//! `ApplicationError` wraps domain-level `GameError` and adds errors that
//! belong to the application orchestration layer (e.g. invalid IDs supplied
//! before domain logic is even reached, game-not-found lookups in the
//! repository).

use thiserror::Error;

use crate::domain::errors::GameError;

/// All errors that can occur in the application layer.
///
/// Variants are either direct wrappers around [`GameError`] (domain failures)
/// or application-level guard failures that happen before domain logic runs.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ApplicationError {
    /// A supplied game ID is not a valid UUID.
    #[error("Invalid game id: '{id}' is not a valid UUID")]
    InvalidGameId { id: String },

    /// A supplied player ID is not a valid UUID.
    #[error("Invalid player id: '{id}' is not a valid UUID")]
    InvalidPlayerId { id: String },

    /// The game could not be found in the repository.
    #[error("Game with id '{id}' not found")]
    GameNotFound { id: String },

    /// A domain-level rule was violated.
    #[error("Domain error: {0}")]
    Domain(#[from] GameError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_game_id_message() {
        let err = ApplicationError::InvalidGameId {
            id: "bad-id".to_owned(),
        };
        assert_eq!(err.to_string(), "Invalid game id: 'bad-id' is not a valid UUID");
    }

    #[test]
    fn invalid_player_id_message() {
        let err = ApplicationError::InvalidPlayerId {
            id: "not-a-uuid".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid player id: 'not-a-uuid' is not a valid UUID"
        );
    }

    #[test]
    fn game_not_found_message() {
        let err = ApplicationError::GameNotFound {
            id: "some-id".to_owned(),
        };
        assert_eq!(err.to_string(), "Game with id 'some-id' not found");
    }

    #[test]
    fn domain_error_wraps_game_error() {
        let domain = GameError::GameNotStarted;
        let app: ApplicationError = domain.into();
        assert!(app.to_string().contains("Domain error"));
    }
}
