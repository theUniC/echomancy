//! Infrastructure layer — data projection and UI-facing types.
//!
//! Provides:
//! - `allowed_actions` — `AllowedActionsResult` struct consumed by the Bevy UI.
//! - `game_snapshot` — `GameSnapshot` player-relative view.
//! - `game_state_export` — serialisable snapshot of the complete game state.
//! - `clips` — CLIPS rules engine safe wrapper (`ClipsEngine`).
//! - `mtgjson` — MTGJSON AtomicCards.json loader and parser.

pub mod allowed_actions;
pub(crate) mod clips;
pub mod legal_actions;

// Re-export the public factory function for creating a CLIPS rules engine.
pub use clips::create_rules_engine;
pub mod game_snapshot;
pub mod game_state_export;
pub(crate) mod mtgjson;
