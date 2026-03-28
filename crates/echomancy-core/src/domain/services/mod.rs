//! Domain services — stateless pure functions that compute results from
//! entity/value-object inputs.
//!
//! None of these modules mutate state; all mutations are applied by the
//! caller (the Game aggregate). They are fully tested via `#[cfg(test)]`
//! modules in each file.

pub(crate) mod combat_declarations;
pub(crate) mod combat_resolution;
pub(crate) mod mana_payment;
pub(crate) mod spell_timing;
pub(crate) mod state_based_actions;
pub(crate) mod step_machine;
pub(crate) mod trigger_evaluation;
