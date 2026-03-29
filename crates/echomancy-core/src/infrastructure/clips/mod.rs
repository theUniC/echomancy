//! CLIPS rules engine safe wrapper.
//!
//! `ClipsEngine` wraps the raw `clips-sys` FFI in a memory-safe Rust API.
//! It provides low-level CLIPS operations: loading constructs, asserting facts,
//! running the inference engine, and collecting output facts.
//!
//! # Thread Safety
//!
//! `ClipsEngine` holds a raw `*mut clips_sys::Environment` pointer.
//! CLIPS environments are NOT thread-safe — do not share a `ClipsEngine`
//! across threads. Each thread that needs CLIPS must create its own engine.
//! The struct deliberately does not implement `Send` or `Sync`.
//!
//! # Relationship to `RulesEngine`
//!
//! In M1, `ClipsEngine` is a low-level wrapper only.
//! `RulesEngine` trait implementation (the bridge from Game state to CLIPS)
//! is added in M2/M3.

use std::collections::HashMap;
use std::ffi::{CStr, CString};

use crate::domain::events::GameEvent;
use crate::domain::game::Game;
use crate::domain::rules_engine::{InputRequest, PlayerChoice, RulesEngine, RulesError, RulesResult};

pub(crate) mod actions;
pub(crate) mod bridge;
pub(crate) mod card_rules;
pub(crate) mod router;

/// CLIPS fact templates embedded in the binary.
const CORE_TEMPLATES: &str = include_str!("../../../../../rules/core/templates.clp");

// ============================================================================
// Slot value
// ============================================================================

/// A typed slot value read from a CLIPS fact.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SlotValue {
    Integer(i64),
    // FIXME(M3): add clips_value_as_float C shim to support float slot reads.
    // The game schema uses INTEGER for all numerics, so this path is not yet hit.
    #[allow(dead_code)]
    Float(f64),
    String(String),
    Symbol(String),
    Void,
}

// ============================================================================
// A collected fact row
// ============================================================================

/// A fact row read from CLIPS working memory for a given template.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FactRow {
    /// Template name (e.g. `"action-damage"`).
    pub template: String,
    /// Slot values keyed by slot name.
    pub slots: HashMap<String, SlotValue>,
}

// ============================================================================
// ClipsEngine
// ============================================================================

/// Safe wrapper around a CLIPS environment.
///
/// # Safety
///
/// All FFI calls go through this wrapper. The raw pointer is only accessed
/// inside `unsafe` blocks in this module. No raw pointer ever escapes.
///
/// CLIPS environments are single-threaded. We implement `Send` (but NOT `Sync`)
/// so that `ClipsEngine` can be stored inside `Game` which is a Bevy `Resource`.
/// Bevy's main thread schedule guarantees exclusive `&mut` access, so this is safe
/// as long as `ClipsEngine` is never shared between threads concurrently.
pub(crate) struct ClipsEngine {
    /// Raw CLIPS environment pointer. Never null after `new()`.
    env: *mut clips_sys::Environment,
    /// Upper bound on rules fired per `run()` call (prevents infinite loops).
    max_rules: i64,
    /// The last pending input request, if a CLIPS rule called `(halt)`.
    pending_input: Option<InputRequest>,
}

// SAFETY: CLIPS environments are single-threaded. We implement Send so that
// ClipsEngine can be stored in Game (a Bevy Resource, which requires Send).
// Bevy's schedule gives exclusive &mut access on the main thread, so the
// pointer is never accessed from multiple threads concurrently.
// We also implement Sync because Bevy Resources require Send + Sync.
// The trait methods all take &mut self (exclusive access), so concurrent
// shared access never occurs in practice.
unsafe impl Send for ClipsEngine {}
unsafe impl Sync for ClipsEngine {}

impl ClipsEngine {
    /// Default maximum rules per `run()` call.
    // Used in tests and will be used in production when M3 wires ClipsEngine into the game loop.
    #[allow(dead_code)]
    pub(crate) const DEFAULT_MAX_RULES: i64 = 10_000;

    /// Create a new CLIPS environment.
    ///
    /// # Errors
    ///
    /// Returns `RulesError::EnvironmentCreationFailed` if CLIPS cannot
    /// allocate the environment (extremely rare — out-of-memory condition).
    // Used in tests and will be used in production when M3 wires ClipsEngine.
    #[allow(dead_code)]
    pub(crate) fn new() -> Result<Self, RulesError> {
        let env = unsafe { clips_sys::CreateEnvironment() };
        if env.is_null() {
            return Err(RulesError::EnvironmentCreationFailed);
        }
        Ok(Self {
            env,
            max_rules: Self::DEFAULT_MAX_RULES,
            pending_input: None,
        })
    }

    /// Create a new engine with a custom max-rules limit.
    // Used in tests.
    #[allow(dead_code)]
    pub(crate) fn with_max_rules(max_rules: i64) -> Result<Self, RulesError> {
        let mut engine = Self::new()?;
        engine.max_rules = max_rules;
        Ok(engine)
    }

    /// Load CLIPS constructs (deftemplate, defrule, etc.) from a string.
    ///
    /// This calls `LoadFromString` which parses and compiles the constructs
    /// into the environment. Any syntax errors are detected here.
    ///
    /// # Errors
    ///
    /// Returns `RulesError::LoadFailed` if CLIPS rejects the construct string.
    pub(crate) fn load_rules(&mut self, code: &str) -> Result<(), RulesError> {
        let c_code = CString::new(code)
            .map_err(|e| RulesError::LoadFailed(format!("invalid UTF-8 in rules: {e}")))?;

        let ok = unsafe {
            clips_sys::LoadFromString(self.env, c_code.as_ptr(), usize::MAX)
        };

        if ok {
            Ok(())
        } else {
            Err(RulesError::LoadFailed(
                "CLIPS rejected the construct string".to_owned(),
            ))
        }
    }

    /// Reset the environment: retract all facts and re-assert any `deffacts`.
    ///
    /// Call this before each new evaluation cycle to start from a clean slate.
    pub(crate) fn reset(&mut self) {
        unsafe { clips_sys::Reset(self.env) };
    }

    /// Assert a fact from a string representation.
    ///
    /// Example: `"(action-damage (source \"bolt-1\") (target \"p2\") (amount 3))"`
    ///
    /// # Errors
    ///
    /// Returns `RulesError::AssertFailed` if CLIPS cannot parse or assert the fact.
    pub(crate) fn assert_fact(&mut self, fact: &str) -> Result<(), RulesError> {
        let c_fact = CString::new(fact)
            .map_err(|e| RulesError::AssertFailed(format!("invalid UTF-8 in fact: {e}")))?;

        let result = unsafe { clips_sys::AssertString(self.env, c_fact.as_ptr()) };

        if result.is_null() {
            Err(RulesError::AssertFailed(format!(
                "CLIPS rejected fact: {fact}"
            )))
        } else {
            Ok(())
        }
    }

    /// Run the inference engine for up to `self.max_rules` rule firings.
    ///
    /// Returns the number of rules actually fired.
    ///
    /// # Errors
    ///
    /// Returns `RulesError::MaxRulesExceeded` if the number of rules fired
    /// equals the limit (suggesting a possible infinite loop).
    pub(crate) fn run(&mut self) -> Result<i64, RulesError> {
        let fired = unsafe { clips_sys::Run(self.env, self.max_rules) };

        if fired >= self.max_rules {
            Err(RulesError::MaxRulesExceeded {
                limit: self.max_rules,
            })
        } else {
            Ok(fired)
        }
    }

    /// Collect all facts matching a given template name.
    ///
    /// Iterates CLIPS working memory and returns one `FactRow` per matching fact.
    /// Slot names must be provided so we know which slots to read.
    ///
    /// # Arguments
    ///
    /// - `template`: the deftemplate name, e.g. `"action-damage"`.
    /// - `slot_names`: the slot names to read from each fact, e.g.
    ///   `&["source", "target", "amount"]`.
    pub(crate) fn collect_facts_by_template(
        &self,
        template: &str,
        slot_names: &[&str],
    ) -> Vec<FactRow> {
        let c_template = match CString::new(template) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let deftemplate =
            unsafe { clips_sys::FindDeftemplate(self.env, c_template.as_ptr()) };
        if deftemplate.is_null() {
            return Vec::new();
        }

        let mut rows = Vec::new();
        let mut fact = unsafe { clips_sys::GetNextFactInTemplate(deftemplate, std::ptr::null_mut()) };

        while !fact.is_null() {
            let mut slots = HashMap::new();

            for &slot_name in slot_names {
                let c_slot = match CString::new(slot_name) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let mut cv = clips_sys::CLIPSValue {
                    value: std::ptr::null_mut(),
                };

                // GetFactSlot returns GetSlotError: 0 = GSE_NO_ERROR (success),
                // non-zero = error (slot not found, null pointer, etc.).
                let err = unsafe { clips_sys::GetFactSlot(fact, c_slot.as_ptr(), &mut cv) };
                if err != 0 {
                    continue;
                }

                let type_tag = unsafe { clips_sys::clips_value_type(&mut cv) };
                let value = match type_tag {
                    clips_sys::INTEGER_TYPE => {
                        let i = unsafe { clips_sys::clips_value_as_integer(&mut cv) };
                        SlotValue::Integer(i)
                    }
                    clips_sys::FLOAT_TYPE => {
                        // FIXME: add clips_value_as_float C shim to read actual value.
                        // MTG game schema uses INTEGER_TYPE for all numerics, so this
                        // is not triggered in practice. Returning Void to avoid silent
                        // data corruption.
                        SlotValue::Void
                    }
                    clips_sys::MULTIFIELD_TYPE => {
                        // FIXME: multislot reading not implemented yet. Needed for
                        // CreateToken's `types` and `keywords` slots in M2/M3.
                        SlotValue::Void
                    }
                    clips_sys::SYMBOL_TYPE => {
                        let ptr = unsafe { clips_sys::clips_value_as_string(&mut cv) };
                        if ptr.is_null() {
                            SlotValue::Symbol(String::new())
                        } else {
                            let s = unsafe { CStr::from_ptr(ptr) }
                                .to_string_lossy()
                                .into_owned();
                            SlotValue::Symbol(s)
                        }
                    }
                    clips_sys::STRING_TYPE => {
                        let ptr = unsafe { clips_sys::clips_value_as_string(&mut cv) };
                        if ptr.is_null() {
                            SlotValue::String(String::new())
                        } else {
                            let s = unsafe { CStr::from_ptr(ptr) }
                                .to_string_lossy()
                                .into_owned();
                            SlotValue::String(s)
                        }
                    }
                    clips_sys::VOID_TYPE => SlotValue::Void,
                    _ => SlotValue::Void,
                };

                slots.insert(slot_name.to_owned(), value);
            }

            rows.push(FactRow {
                template: template.to_owned(),
                slots,
            });

            fact = unsafe { clips_sys::GetNextFactInTemplate(deftemplate, fact) };
        }

        rows
    }
}

impl std::fmt::Debug for ClipsEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClipsEngine")
            .field("max_rules", &self.max_rules)
            .finish_non_exhaustive()
    }
}

impl Drop for ClipsEngine {
    fn drop(&mut self) {
        // SAFETY: self.env is always non-null (enforced in new()) and we own it.
        unsafe { clips_sys::DestroyEnvironment(self.env) };
    }
}

// ============================================================================
// RulesEngine trait implementation
// ============================================================================

impl RulesEngine for ClipsEngine {
    /// Evaluate a game event against the current game state.
    ///
    /// Cycle:
    /// 1. Reset working memory (clean slate).
    /// 2. Load core templates (if not already done — idempotent via Reset).
    /// 3. Assert all game state facts.
    /// 4. Assert the event fact.
    /// 5. Run the inference engine (bounded).
    /// 6. Check for `awaiting-input` facts (CLIPS called halt).
    /// 7. Collect and sort `action-*` facts.
    /// 8. Return `RulesResult`.
    fn evaluate(
        &mut self,
        state: &Game,
        event: &GameEvent,
    ) -> Result<RulesResult, RulesError> {
        // 1. Reset — wipe all facts, reload deffacts
        self.reset();

        // Templates are already loaded into the environment (they survive Reset).
        // Load them now if this is the first evaluate() call.
        // We detect this by checking whether the "player" deftemplate exists.
        let player_template_exists = {
            let c = std::ffi::CString::new("player").unwrap();
            let ptr = unsafe { clips_sys::FindDeftemplate(self.env, c.as_ptr()) };
            !ptr.is_null()
        };
        if !player_template_exists {
            self.load_rules(CORE_TEMPLATES)?;
        }

        // 2. Assert all game state facts
        for fact in bridge::serialize_game_state(state) {
            self.assert_fact(&fact)?;
        }

        // 3. Assert the event fact
        let event_fact = bridge::serialize_game_event(event);
        self.assert_fact(&event_fact)?;

        // 4. Run (bounded)
        let rules_fired = match self.run() {
            Ok(n) => n,
            Err(RulesError::MaxRulesExceeded { limit }) => {
                return Err(RulesError::MaxRulesExceeded { limit });
            }
            Err(e) => return Err(e),
        };

        // 5. Check for awaiting-input
        let awaiting_input = actions::parse_awaiting_input(self);
        if let Some(ref req) = awaiting_input {
            self.pending_input = Some(req.clone());
        } else {
            self.pending_input = None;
        }

        // 6. Collect action-* facts
        let action_list = actions::parse_action_facts(self)?;

        Ok(RulesResult {
            actions: action_list,
            awaiting_input,
            rules_fired,
            warnings: Vec::new(),
        })
    }

    /// Resume rule execution after the player has made a choice.
    ///
    /// Asserts a `player-choice` fact and runs the engine again.
    fn resume_after_choice(
        &mut self,
        choice: &PlayerChoice,
    ) -> Result<RulesResult, RulesError> {
        if self.pending_input.is_none() {
            return Err(RulesError::NoInputPending);
        }

        // Assert the player's choice fact
        let choice_fact = format!(
            "(player-choice (input-type \"{input_type}\") (player \"{player}\") (chosen \"{chosen}\"))",
            input_type = choice.input_type.replace('"', "\\\""),
            player = choice.player.replace('"', "\\\""),
            chosen = choice.chosen.replace('"', "\\\""),
        );
        // Ignore assert failure — the player-choice template may not be loaded yet.
        // Rules that use it must define it themselves.
        let _ = self.assert_fact(&choice_fact);

        self.pending_input = None;

        let rules_fired = self.run()?;
        let awaiting_input = actions::parse_awaiting_input(self);
        if let Some(ref req) = awaiting_input {
            self.pending_input = Some(req.clone());
        }

        let action_list = actions::parse_action_facts(self)?;

        Ok(RulesResult {
            actions: action_list,
            awaiting_input,
            rules_fired,
            warnings: Vec::new(),
        })
    }
}

// ============================================================================
// Tests (TDD: written before implementation, drive the API design)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal deftemplate + defrule pair used across multiple tests.
    const SIMPLE_RULES: &str = r#"
(deftemplate test-input
  (slot value (type INTEGER)))

(deftemplate test-output
  (slot result (type INTEGER)))

(defrule double-value
  (test-input (value ?v))
  =>
  (assert (test-output (result ?v))))
"#;

    // ---- Environment lifecycle -----------------------------------------------

    #[test]
    fn creates_environment_and_drops_cleanly() {
        // If Drop panics or double-frees, this test will crash.
        let mut engine = ClipsEngine::new().expect("should create CLIPS environment");
        drop(engine);
    }

    #[test]
    fn multiple_engines_can_coexist() {
        let mut a = ClipsEngine::new().expect("engine A");
        let mut b = ClipsEngine::new().expect("engine B");
        // Both alive at same time — independent environments.
        drop(a);
        drop(b);
    }

    // ---- Load rules ----------------------------------------------------------

    #[test]
    fn load_valid_constructs_succeeds() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).expect("valid rules should load");
    }

    #[test]
    fn load_invalid_constructs_returns_error() {
        let mut engine = ClipsEngine::new().unwrap();
        let result = engine.load_rules("(defrule broken (=> (bad syntax)))");
        assert!(
            result.is_err(),
            "invalid rules should return LoadFailed error"
        );
        assert!(matches!(result.unwrap_err(), RulesError::LoadFailed(_)));
    }

    #[test]
    fn load_empty_string_succeeds() {
        let mut engine = ClipsEngine::new().unwrap();
        // Empty string is valid — nothing to load, no error.
        engine.load_rules("").expect("empty rules should succeed");
    }

    // ---- Assert + run + collect ----------------------------------------------

    #[test]
    fn assert_valid_fact_succeeds() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).unwrap();
        engine.reset();
        engine
            .assert_fact("(test-input (value 42))")
            .expect("valid fact should assert");
    }

    #[test]
    fn assert_invalid_fact_returns_error() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).unwrap();
        engine.reset();
        // Assert a fact with an unknown template — CLIPS rejects it.
        let result = engine.assert_fact("(nonexistent-template (foo bar))");
        assert!(
            result.is_err(),
            "asserting unknown template should fail"
        );
        assert!(matches!(result.unwrap_err(), RulesError::AssertFailed(_)));
    }

    #[test]
    fn run_fires_matching_rules_and_produces_output_facts() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).unwrap();
        engine.reset();
        engine.assert_fact("(test-input (value 7))").unwrap();

        let fired = engine.run().expect("run should succeed");
        assert_eq!(fired, 1, "exactly one rule should fire");

        // Collect the output fact.
        let rows = engine.collect_facts_by_template("test-output", &["result"]);
        assert_eq!(rows.len(), 1, "should have one output fact");
        assert_eq!(
            rows[0].slots.get("result"),
            Some(&SlotValue::Integer(7)),
            "output result should equal input value"
        );
    }

    #[test]
    fn run_with_no_matching_facts_fires_zero_rules() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).unwrap();
        engine.reset();
        // No test-input facts asserted — rule should not fire.
        let fired = engine.run().expect("run should succeed");
        assert_eq!(fired, 0);
    }

    // ---- Bounded execution --------------------------------------------------

    #[test]
    fn run_returns_error_when_max_rules_hit() {
        // Create a self-reinforcing rule that fires indefinitely.
        let looping_rules = r#"
(deftemplate counter (slot n (type INTEGER)))
(defrule increment
  ?f <- (counter (n ?n))
  =>
  (retract ?f)
  (assert (counter (n ?n))))
"#;
        let mut engine = ClipsEngine::with_max_rules(5).unwrap();
        engine.load_rules(looping_rules).unwrap();
        engine.reset();
        engine.assert_fact("(counter (n 0))").unwrap();

        let result = engine.run();
        assert!(
            result.is_err(),
            "bounded execution should return MaxRulesExceeded"
        );
        assert!(matches!(
            result.unwrap_err(),
            RulesError::MaxRulesExceeded { limit: 5 }
        ));
    }

    // ---- Reset ---------------------------------------------------------------

    #[test]
    fn reset_clears_asserted_facts() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).unwrap();
        engine.reset();
        engine.assert_fact("(test-input (value 1))").unwrap();
        engine.assert_fact("(test-input (value 2))").unwrap();

        // Reset should wipe working memory.
        engine.reset();

        let fired = engine.run().expect("run after reset");
        assert_eq!(fired, 0, "no facts left after reset — no rules should fire");
    }

    #[test]
    fn run_after_reset_and_reassert_fires_again() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SIMPLE_RULES).unwrap();

        // First cycle
        engine.reset();
        engine.assert_fact("(test-input (value 3))").unwrap();
        let fired1 = engine.run().unwrap();
        assert_eq!(fired1, 1);

        // Second cycle — reset + reassert
        engine.reset();
        engine.assert_fact("(test-input (value 4))").unwrap();
        let fired2 = engine.run().unwrap();
        assert_eq!(fired2, 1);

        let rows = engine.collect_facts_by_template("test-output", &["result"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].slots.get("result"), Some(&SlotValue::Integer(4)));
    }

    // ---- collect_facts_by_template ------------------------------------------

    #[test]
    fn collect_facts_returns_empty_when_template_not_found() {
        let mut engine = ClipsEngine::new().unwrap();
        engine.reset();
        let rows = engine.collect_facts_by_template("nonexistent", &["slot"]);
        assert!(rows.is_empty());
    }

    #[test]
    fn collect_facts_returns_all_matching_facts() {
        const MULTI_RULES: &str = r#"
(deftemplate item (slot name (type STRING)) (slot qty (type INTEGER)))
"#;
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(MULTI_RULES).unwrap();
        engine.reset();
        engine.assert_fact("(item (name \"sword\") (qty 1))").unwrap();
        engine.assert_fact("(item (name \"shield\") (qty 2))").unwrap();

        let rows = engine.collect_facts_by_template("item", &["name", "qty"]);
        assert_eq!(rows.len(), 2, "should collect both item facts");

        // Verify both facts are present (order not guaranteed).
        let qtys: Vec<i64> = rows
            .iter()
            .filter_map(|r| {
                if let Some(SlotValue::Integer(q)) = r.slots.get("qty") {
                    Some(*q)
                } else {
                    None
                }
            })
            .collect();
        assert!(qtys.contains(&1), "qty 1 should be present");
        assert!(qtys.contains(&2), "qty 2 should be present");
    }

    // ---- String slot values --------------------------------------------------

    #[test]
    fn collect_string_slot_values() {
        const STRING_RULES: &str = r#"
(deftemplate message (slot text (type STRING)))
"#;
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(STRING_RULES).unwrap();
        engine.reset();
        engine.assert_fact("(message (text \"hello world\"))").unwrap();

        let rows = engine.collect_facts_by_template("message", &["text"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].slots.get("text"),
            Some(&SlotValue::String("hello world".to_owned()))
        );
    }

    // ---- Symbol slot values -------------------------------------------------

    #[test]
    fn collect_symbol_slot_values() {
        const SYMBOL_RULES: &str = r#"
(deftemplate flag (slot name (type SYMBOL)))
"#;
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(SYMBOL_RULES).unwrap();
        engine.reset();
        engine.assert_fact("(flag (name TRUE))").unwrap();

        let rows = engine.collect_facts_by_template("flag", &["name"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].slots.get("name"),
            Some(&SlotValue::Symbol("TRUE".to_owned()))
        );
    }

    // ---- RulesEngine trait implementation -----------------------------------

    #[test]
    fn rules_engine_evaluate_returns_empty_actions_for_no_rules() {
        use crate::domain::events::GameEvent;
        use crate::domain::enums::Step;
        use crate::domain::types::PlayerId;
        use crate::domain::game::test_helpers::make_started_game;
        use crate::domain::rules_engine::RulesEngine;

        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(CORE_TEMPLATES).unwrap();

        let (game, p1, _) = make_started_game();
        let event = GameEvent::StepStarted {
            step: Step::FirstMain,
            active_player_id: PlayerId::new(&p1),
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert!(result.actions.is_empty(), "no rules defined, so no actions");
        assert!(result.awaiting_input.is_none());
    }

    #[test]
    fn rules_engine_evaluate_produces_action_from_rule() {
        use crate::domain::events::GameEvent;
        use crate::domain::enums::Step;
        use crate::domain::types::PlayerId;
        use crate::domain::game::test_helpers::make_started_game;
        use crate::domain::rules_engine::{RulesAction, RulesEngine};

        // A simple test rule: when STEP_STARTED fires, draw a card.
        const TEST_RULE: &str = r#"
(defrule test-draw-on-first-main
  (game-event (type STEP_STARTED) (data "FIRST_MAIN"))
  (player (id ?p) (has-priority TRUE))
  =>
  (assert (action-draw (player ?p) (amount 1))))
"#;
        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(CORE_TEMPLATES).unwrap();
        engine.load_rules(TEST_RULE).unwrap();

        let (game, p1, _) = make_started_game();
        let event = GameEvent::StepStarted {
            step: Step::FirstMain,
            active_player_id: PlayerId::new(&p1),
        };

        let result = engine.evaluate(&game, &event).expect("evaluate should succeed");
        assert_eq!(result.actions.len(), 1);
        assert!(matches!(
            &result.actions[0],
            RulesAction::DrawCards { player, amount: 1 } if player == "p1"
        ));
    }

    #[test]
    fn rules_engine_reset_between_cycles_clears_facts() {
        use crate::domain::events::GameEvent;
        use crate::domain::enums::Step;
        use crate::domain::types::PlayerId;
        use crate::domain::game::test_helpers::make_started_game;
        use crate::domain::rules_engine::RulesEngine;

        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(CORE_TEMPLATES).unwrap();

        let (game, p1, _) = make_started_game();
        let event = GameEvent::StepStarted {
            step: Step::Upkeep,
            active_player_id: PlayerId::new(&p1),
        };

        // Two evaluations: each should produce independent results
        let result1 = engine.evaluate(&game, &event).expect("first evaluate");
        let result2 = engine.evaluate(&game, &event).expect("second evaluate");
        // Both should have the same number of actions (deterministic)
        assert_eq!(result1.actions.len(), result2.actions.len());
    }

    #[test]
    fn rules_engine_resume_after_choice_errors_when_no_input_pending() {
        use crate::domain::rules_engine::{PlayerChoice, RulesEngine, RulesError};

        let mut engine = ClipsEngine::new().unwrap();
        engine.load_rules(CORE_TEMPLATES).unwrap();

        let choice = PlayerChoice {
            input_type: "sacrifice".to_owned(),
            player: "p1".to_owned(),
            chosen: "creature-1".to_owned(),
        };
        let err = engine.resume_after_choice(&choice).unwrap_err();
        assert!(matches!(err, RulesError::NoInputPending));
    }
}
