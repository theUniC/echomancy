//! Domain services — stateless pure functions that compute results from
//! entity/value-object inputs.
//!
//! None of these modules mutate state; all mutations are applied by the
//! caller (the Game aggregate in Phase 6).
//!
//! The `dead_code` lint is suppressed here because these are internal APIs
//! that will be consumed by the Game aggregate (Phase 6). They are fully
//! tested via `#[cfg(test)]` modules in each file.
#![allow(dead_code)]

pub mod game_automation;
pub(crate) mod combat_declarations;
pub(crate) mod combat_resolution;
pub mod game_state_export;
pub(crate) mod mana_payment;
pub(crate) mod spell_timing;
pub(crate) mod state_based_actions;
pub(crate) mod step_machine;
pub(crate) mod trigger_evaluation;
