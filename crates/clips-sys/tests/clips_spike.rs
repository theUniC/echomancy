//! CLIPS 6.4.2 PoC spike — SBA rules evaluation.
//!
//! Tests the same 3 State-Based Action rules as a Prolog spike would:
//!   1. Creature with lethal damage → sba-destroy asserted
//!   2. Creature with zero/negative toughness → sba-destroy asserted
//!   3. Player with life <= 0 → sba-lose asserted
//!
//! Also measures per-phase timing.
//!
//! Safety note: all CLIPS FFI calls are wrapped in `unsafe`. This is a PoC.
//! A production safe wrapper would encapsulate all of this.

use clips_sys::*;
use std::ffi::{CStr, CString};
use std::time::Instant;

// ---------------------------------------------------------------------------
// SBA rules (same logic as the Prolog spike)
// ---------------------------------------------------------------------------

const SBA_RULES: &str = r#"
(deftemplate creature
    (slot id    (type SYMBOL))
    (slot toughness (type INTEGER))
    (slot damage    (type INTEGER)))

(deftemplate player
    (slot id   (type SYMBOL))
    (slot life (type INTEGER)))

(defrule sba-lethal-damage
    "A creature with damage >= toughness is destroyed."
    (creature (id ?id) (toughness ?t) (damage ?d))
    (test (>= ?d ?t))
    =>
    (assert (sba-destroy ?id)))

(defrule sba-zero-toughness
    "A creature with toughness <= 0 is destroyed."
    (creature (id ?id) (toughness ?t))
    (test (<= ?t 0))
    =>
    (assert (sba-destroy ?id)))

(defrule sba-player-loses
    "A player with life <= 0 loses the game."
    (player (id ?id) (life ?l))
    (test (<= ?l 0))
    =>
    (assert (sba-lose ?id)))
"#;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a CLIPS environment and load the SBA rules. Panics on failure.
unsafe fn create_env_with_rules() -> *mut Environment {
    let env = CreateEnvironment();
    assert!(!env.is_null(), "CreateEnvironment returned null");

    let code = CString::new(SBA_RULES).unwrap();
    let ok = LoadFromString(env, code.as_ptr(), usize::MAX);
    assert!(ok, "LoadFromString failed — check rule syntax");

    env
}

/// Assert a deftemplate creature fact.
unsafe fn assert_creature(env: *mut Environment, id: &str, toughness: i64, damage: i64) {
    let s = format!("(creature (id {id}) (toughness {toughness}) (damage {damage}))");
    let c = CString::new(s).unwrap();
    let f = AssertString(env, c.as_ptr());
    assert!(!f.is_null(), "AssertString failed for creature {id}");
}

/// Assert a deftemplate player fact.
unsafe fn assert_player(env: *mut Environment, id: &str, life: i64) {
    let s = format!("(player (id {id}) (life {life}))");
    let c = CString::new(s).unwrap();
    let f = AssertString(env, c.as_ptr());
    assert!(!f.is_null(), "AssertString failed for player {id}");
}

/// Check whether an ordered fact with the given template name and first implied slot value exists.
///
/// For example, after `(assert (sba-destroy c1))`, calling
/// `ordered_fact_exists(env, "sba-destroy", "c1")` returns true.
///
/// Strategy: iterate all facts, filter by template name (using the C helper
/// `clips_fact_relation_name`), then read the "implied" slot and compare its
/// first element (integer value from helper `clips_value_as_integer` or string
/// from `clips_value_as_string`).
///
/// For ordered facts, the implied slot is a multifield. We use Eval to check
/// via a safe CLIPS expression that returns 1 or 0, then read that integer
/// through the `clips_value_as_integer` C helper (which does the struct access in C).
unsafe fn ordered_fact_exists(env: *mut Environment, template: &str, symbol: &str) -> bool {
    // Evaluate a CLIPS expression that returns integer 1 if fact exists, 0 otherwise.
    // `any-factp` is the canonical CLIPS fact-query function.
    let expr = format!(
        "(if (any-factp ((?f {template})) (eq (nth$ 1 ?f:implied) {symbol})) then 1 else 0)"
    );
    let expr_c = CString::new(expr).unwrap();
    let mut result = CLIPSValue {
        value: std::ptr::null_mut(),
    };
    let err_code = Eval(env, expr_c.as_ptr(), &mut result);
    // EvalError: 0 = NoError, 1 = ParsingError, 2 = ProcessingError
    if err_code as u32 != 0 {
        return false;
    }
    // Use the C helper to read the integer value — avoids unsafe struct layout assumptions.
    clips_value_as_integer(&mut result) != 0
}

/// Return the number of facts currently in the environment.
unsafe fn count_facts(env: *mut Environment) -> usize {
    let mut count = 0usize;
    let mut fact = GetNextFact(env, std::ptr::null_mut());
    while !fact.is_null() {
        count += 1;
        fact = GetNextFact(env, fact);
    }
    count
}

/// Return all template names of facts currently in the environment.
unsafe fn all_fact_template_names(env: *mut Environment) -> Vec<String> {
    let mut names = Vec::new();
    let mut fact = GetNextFact(env, std::ptr::null_mut());
    while !fact.is_null() {
        let name_ptr = clips_fact_relation_name(fact);
        if !name_ptr.is_null() {
            let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
            names.push(name);
        }
        fact = GetNextFact(env, fact);
    }
    names
}

// ---------------------------------------------------------------------------
// Tests — RED phase first (these were written before the implementation above)
// ---------------------------------------------------------------------------

#[test]
fn test_env_create_destroy() {
    unsafe {
        let env = CreateEnvironment();
        assert!(!env.is_null(), "CreateEnvironment should return non-null");
        let ok = DestroyEnvironment(env);
        assert!(ok, "DestroyEnvironment should succeed");
    }
}

#[test]
fn test_load_sba_rules() {
    unsafe {
        let env = create_env_with_rules();
        // If we reach here without panic, rules loaded OK.
        DestroyEnvironment(env);
    }
}

#[test]
fn test_fact_relation_name_helper() {
    // Verifies that clips_fact_relation_name (C shim) works correctly.
    unsafe {
        let env = create_env_with_rules();
        assert_creature(env, "c1", 3, 3);

        let fact = GetNextFact(env, std::ptr::null_mut());
        assert!(!fact.is_null(), "Expected at least one fact");

        let name_ptr = clips_fact_relation_name(fact);
        assert!(!name_ptr.is_null(), "Template name should not be null");
        let name = CStr::from_ptr(name_ptr).to_string_lossy();
        assert_eq!(name, "creature", "First fact should be a creature template");

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: creature with damage == toughness → destroyed
// ---------------------------------------------------------------------------
#[test]
fn test_lethal_damage_creature_is_destroyed() {
    unsafe {
        let env = create_env_with_rules();
        assert_creature(env, "c1", 3, 3);

        let fired = Run(env, -1);
        assert!(fired > 0, "Expected at least one rule to fire, got {fired}");

        let destroyed = ordered_fact_exists(env, "sba-destroy", "c1");
        assert!(
            destroyed,
            "Creature c1 with toughness=3 damage=3 should have sba-destroy asserted"
        );

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: creature with damage < toughness → NOT destroyed
// ---------------------------------------------------------------------------
#[test]
fn test_non_lethal_damage_creature_is_not_destroyed() {
    unsafe {
        let env = create_env_with_rules();
        assert_creature(env, "c2", 3, 2);

        Run(env, -1);

        let destroyed = ordered_fact_exists(env, "sba-destroy", "c2");
        assert!(
            !destroyed,
            "Creature c2 with toughness=3 damage=2 should NOT have sba-destroy asserted"
        );

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Scenario 3: creature with toughness == 0 → destroyed (zero-toughness rule)
// ---------------------------------------------------------------------------
#[test]
fn test_zero_toughness_creature_is_destroyed() {
    unsafe {
        let env = create_env_with_rules();
        assert_creature(env, "c3", 0, 0);

        Run(env, -1);

        let destroyed = ordered_fact_exists(env, "sba-destroy", "c3");
        assert!(
            destroyed,
            "Creature c3 with toughness=0 should have sba-destroy asserted"
        );

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Scenario 4: player with life == 0 → loses
// ---------------------------------------------------------------------------
#[test]
fn test_player_at_zero_life_loses() {
    unsafe {
        let env = create_env_with_rules();
        assert_player(env, "p1", 0);

        Run(env, -1);

        let loses = ordered_fact_exists(env, "sba-lose", "p1");
        assert!(
            loses,
            "Player p1 with life=0 should have sba-lose asserted"
        );

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Scenario 5: player with life == 1 → does NOT lose
// ---------------------------------------------------------------------------
#[test]
fn test_player_at_one_life_does_not_lose() {
    unsafe {
        let env = create_env_with_rules();
        assert_player(env, "p2", 1);

        Run(env, -1);

        let loses = ordered_fact_exists(env, "sba-lose", "p2");
        assert!(
            !loses,
            "Player p2 with life=1 should NOT have sba-lose asserted"
        );

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Scenario 6: combined board state — multiple rules fire
// ---------------------------------------------------------------------------
#[test]
fn test_combined_board_state() {
    unsafe {
        let env = create_env_with_rules();

        assert_creature(env, "c1", 3, 3); // lethal → destroy
        assert_creature(env, "c2", 4, 2); // non-lethal → survive
        assert_creature(env, "c3", 0, 0); // zero toughness → destroy
        assert_player(env, "p1", 0);      // dead → lose
        assert_player(env, "p2", 5);      // alive → survive

        let fired = Run(env, -1);
        assert!(
            fired >= 3,
            "Expected at least 3 rules to fire (2 destroy + 1 lose), got {fired}"
        );

        assert!(ordered_fact_exists(env, "sba-destroy", "c1"), "c1 should be destroyed");
        assert!(!ordered_fact_exists(env, "sba-destroy", "c2"), "c2 should survive");
        assert!(ordered_fact_exists(env, "sba-destroy", "c3"), "c3 should be destroyed (0 toughness)");
        assert!(ordered_fact_exists(env, "sba-lose", "p1"), "p1 should lose");
        assert!(!ordered_fact_exists(env, "sba-lose", "p2"), "p2 should survive");

        // Verify fact counts: 5 asserted + 3 SBA results = 8 total
        let total = count_facts(env);
        assert_eq!(total, 8, "Expected 8 total facts (5 asserted + 3 SBA)");

        // Verify template names are present
        let names = all_fact_template_names(env);
        assert!(names.contains(&"sba-destroy".to_string()));
        assert!(names.contains(&"sba-lose".to_string()));
        assert!(names.contains(&"creature".to_string()));
        assert!(names.contains(&"player".to_string()));

        DestroyEnvironment(env);
    }
}

// ---------------------------------------------------------------------------
// Performance benchmark: 50 iterations of full SBA check cycle
// ---------------------------------------------------------------------------
#[test]
fn bench_sba_50_iterations() {
    const ITERATIONS: u32 = 50;

    unsafe {
        // --- Phase 1: Measure environment create + load rules ---
        let t0 = Instant::now();
        for _ in 0..ITERATIONS {
            let e = CreateEnvironment();
            let code = CString::new(SBA_RULES).unwrap();
            LoadFromString(e, code.as_ptr(), usize::MAX);
            DestroyEnvironment(e);
        }
        let env_create_avg = t0.elapsed() / ITERATIONS;

        // Create one long-lived env for the remaining benchmarks
        let env = create_env_with_rules();

        // --- Phase 2: Full SBA cycle (Reset + Assert + Run) ---
        let t1 = Instant::now();
        for _ in 0..ITERATIONS {
            Reset(env); // clears all facts, re-asserts deffacts

            assert_creature(env, "c1", 3, 3);
            assert_creature(env, "c2", 4, 2);
            assert_creature(env, "c3", 0, 0);
            assert_player(env, "p1", 0);
            assert_player(env, "p2", 5);

            Run(env, -1);
        }
        let cycle_avg = t1.elapsed() / ITERATIONS;

        // --- Phase 3: Single fact assertion (Reset + 1 Assert, no Run) ---
        let t2 = Instant::now();
        for _ in 0..ITERATIONS {
            Reset(env);
            assert_creature(env, "c1", 3, 3);
        }
        let assert_avg = t2.elapsed() / ITERATIONS;

        // --- Phase 4: Run only (Reset + Assert + Run, 2 facts) ---
        let t3 = Instant::now();
        for _ in 0..ITERATIONS {
            Reset(env);
            assert_creature(env, "c1", 3, 3);
            assert_player(env, "p1", 0);
            Run(env, -1);
        }
        let run_two_facts_avg = t3.elapsed() / ITERATIONS;

        // --- Phase 5: Fact query (ordered_fact_exists) ---
        Reset(env);
        assert_creature(env, "c1", 3, 3);
        Run(env, -1);

        let t4 = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = ordered_fact_exists(env, "sba-destroy", "c1");
        }
        let query_avg = t4.elapsed() / ITERATIONS;

        // Print results (visible with -- --nocapture)
        println!("\n");
        println!("=== CLIPS 6.4.2 Performance Report ({ITERATIONS} iterations) ===");
        println!("  Environment create + load rules  : {:>10?} avg", env_create_avg);
        println!("  Full SBA cycle (5 facts + run)   : {:>10?} avg", cycle_avg);
        println!("  Single fact assertion (Reset+1)   : {:>10?} avg", assert_avg);
        println!("  Reset + 2 asserts + Run           : {:>10?} avg", run_two_facts_avg);
        println!("  Fact query (any-factp Eval)       : {:>10?} avg", query_avg);
        println!("==============================================================");
        println!();

        // Sanity check: all averages are positive durations
        assert!(cycle_avg.as_nanos() > 0);
        assert!(env_create_avg.as_nanos() > 0);

        DestroyEnvironment(env);
    }
}
