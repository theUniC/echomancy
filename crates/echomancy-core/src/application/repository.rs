//! GameRepository trait — the persistence contract for the application layer.
//!
//! Defined here (application layer) rather than in the domain layer. This is
//! a deliberate MVP simplification: the repository abstracts over storage but
//! does not need to live in the pure domain.
//!
//! Implementations live in `crate::infrastructure`.

use crate::domain::game::Game;

/// Persistence contract for `Game` aggregates.
///
/// Implementors are responsible for storing and retrieving `Game` instances.
/// The in-memory implementation lives in `crate::infrastructure::in_memory_repo`.
pub trait GameRepository: Send + Sync {
    /// Store a game. If a game with the same ID already exists, it is replaced.
    fn save(&mut self, game: Game);

    /// Retrieve a game by its ID. Returns `None` if not found.
    fn find_by_id(&self, game_id: &str) -> Option<&Game>;

    /// Retrieve a mutable reference to a game by its ID.
    fn find_by_id_mut(&mut self, game_id: &str) -> Option<&mut Game>;

    /// Returns all games in the repository.
    fn all(&self) -> Vec<&Game>;
}
