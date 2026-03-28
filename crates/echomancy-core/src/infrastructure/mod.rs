//! Infrastructure layer — data projection and UI-facing types.
//!
//! Provides:
//! - `allowed_actions` — `AllowedActionsResult` struct consumed by the Bevy UI.
//! - `game_snapshot` — `GameSnapshot` player-relative view.
//! - `game_state_export` — serialisable snapshot of the complete game state.

pub mod allowed_actions;
pub mod game_snapshot;
pub mod game_state_export;
