//! Domain specifications — pure boolean predicates that check whether an
//! action is allowed in the current game state.
//!
//! Each specification:
//! - Takes a lightweight context struct (uses references, no cloning).
//! - Returns `Ok(())` if the action is allowed.
//! - Returns `Err(GameError::...)` with a specific error explaining *why* the
//!   action is not allowed.
//!
//! Specifications do NOT mutate state. They are read-only checks consumed by
//! the Game aggregate (Phase 6).
//!
//! See `docs/specs/active/` for the design document.

pub(crate) mod can_activate_ability;
pub(crate) mod can_cast_spell;
pub(crate) mod can_declare_attacker;
pub(crate) mod can_play_land;
pub(crate) mod has_priority;
