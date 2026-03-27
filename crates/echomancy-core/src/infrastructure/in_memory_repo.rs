//! In-memory implementation of `GameRepository`.
//!
//! Stores games in a `HashMap<String, Game>`. Suitable for tests and the
//! single-process use-case (the Bevy game binary).

use std::collections::HashMap;

use crate::application::repository::GameRepository;
use crate::domain::game::Game;

/// A `GameRepository` that stores games in process memory.
///
/// Thread-safety is not a concern for the Bevy single-threaded game loop;
/// the `Send + Sync` bounds on `GameRepository` are satisfied because
/// `HashMap` and `Game` are both `Send + Sync`.
pub struct InMemoryGameRepository {
    games: HashMap<String, Game>,
}

impl InMemoryGameRepository {
    /// Create a new, empty repository.
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }
}

impl Default for InMemoryGameRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl GameRepository for InMemoryGameRepository {
    fn save(&mut self, game: Game) {
        self.games.insert(game.id().to_owned(), game);
    }

    fn find_by_id(&self, game_id: &str) -> Option<&Game> {
        self.games.get(game_id)
    }

    fn find_by_id_mut(&mut self, game_id: &str) -> Option<&mut Game> {
        self.games.get_mut(game_id)
    }

    fn all(&self) -> Vec<&Game> {
        self.games.values().collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game(id: &str) -> Game {
        Game::create(id)
    }

    #[test]
    fn new_repo_is_empty() {
        let repo = InMemoryGameRepository::new();
        assert!(repo.all().is_empty());
    }

    #[test]
    fn save_and_find_by_id() {
        let mut repo = InMemoryGameRepository::new();
        let game = make_game("game-1");
        repo.save(game);
        assert!(repo.find_by_id("game-1").is_some());
    }

    #[test]
    fn find_by_id_returns_none_for_missing_game() {
        let repo = InMemoryGameRepository::new();
        assert!(repo.find_by_id("nonexistent").is_none());
    }

    #[test]
    fn save_replaces_existing_game() {
        let mut repo = InMemoryGameRepository::new();
        let game1 = make_game("game-1");
        let game2 = make_game("game-1");
        repo.save(game1);
        repo.save(game2);
        assert_eq!(repo.all().len(), 1);
    }

    #[test]
    fn all_returns_all_saved_games() {
        let mut repo = InMemoryGameRepository::new();
        repo.save(make_game("game-1"));
        repo.save(make_game("game-2"));
        assert_eq!(repo.all().len(), 2);
    }

    #[test]
    fn find_by_id_mut_allows_mutation() {
        let mut repo = InMemoryGameRepository::new();
        repo.save(make_game("game-1"));

        // Add a player to verify mutability
        let game = repo.find_by_id_mut("game-1").unwrap();
        game.add_player("player-1", "Alice").unwrap();

        let game = repo.find_by_id("game-1").unwrap();
        assert!(game.has_player("player-1"));
    }

    #[test]
    fn find_by_id_mut_returns_none_for_missing_game() {
        let mut repo = InMemoryGameRepository::new();
        assert!(repo.find_by_id_mut("nonexistent").is_none());
    }
}
