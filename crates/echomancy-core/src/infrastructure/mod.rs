//! Infrastructure layer — implementations of application-layer contracts.
//!
//! Provides:
//! - `in_memory_repo` — `InMemoryGameRepository` implementation.
//! - `game_snapshot` — `GameSnapshot` player-relative view.
//! - `game_state_export` — serialisable snapshot of the complete game state.

pub mod game_snapshot;
pub mod game_state_export;
pub mod in_memory_repo;
