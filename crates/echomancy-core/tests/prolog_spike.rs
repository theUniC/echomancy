// Spike: Scryer Prolog embedded in Rust for State-Based Actions
//
// This test evaluates whether scryer-prolog is a viable approach for encoding
// Magic: The Gathering State-Based Actions (SBAs) as Prolog rules and querying
// them from Rust. It is deliberately isolated from the game engine.
//
// SBAs Under Test:
//   Rule 704.5g - A creature with toughness <= 0 is put into its owner's graveyard.
//   Rule 704.5h - A creature with lethal damage (damage >= toughness) is destroyed.
//   Rule 704.5a - A player with life total <= 0 loses the game.
//
// Evaluation Criteria:
//   1. Correctness  - Do Prolog rules give the same answers as a hand-written Rust function?
//   2. Ergonomics   - How painful is it to load rules, assert facts, and extract results?
//   3. Performance  - How does Prolog query time compare to native Rust iteration?
//
// Approach:
//   Rather than using assertz/1 at query time (which triggers an internal panic in
//   scryer-prolog 0.10.0 when combined with findall and use_module(library(lists))),
//   facts are embedded directly in the Prolog program string passed to
//   `consult_module_string`. This is the pattern used by the official test suite.

// ---------------------------------------------------------------------------
// Static SBA rules (no facts — loaded once)
// ---------------------------------------------------------------------------

const SBA_RULES: &str = r#"
% Rule 704.5h: creature has lethal damage (damage_marked >= toughness)
sba_destroy_lethal(CreatureId) :-
    creature(CreatureId, Toughness, DamageMarked),
    DamageMarked >= Toughness.

% Rule 704.5g: creature has zero or less toughness
sba_destroy_zero_toughness(CreatureId) :-
    creature(CreatureId, Toughness, _),
    Toughness =< 0.

% Combined: any creature that satisfies EITHER rule.
% once/1 commits to the first matching condition so a creature matching both
% (e.g. zero toughness AND lethal damage) is only yielded once by findall/3.
sba_destroy(CreatureId) :-
    creature(CreatureId, Toughness, DamageMarked),
    once((Toughness =< 0 ; DamageMarked >= Toughness)).

% Rule 704.5a: player has life total <= 0
sba_player_loses(PlayerId) :-
    player(PlayerId, Life),
    Life =< 0.

% Collect all creatures that should be destroyed (unique because creature/3 facts are unique)
all_creatures_to_destroy(Ids) :-
    findall(Id, sba_destroy(Id), Ids).

% Collect all players that should lose
all_players_losing(Ids) :-
    findall(Id, sba_player_loses(Id), Ids).
"#;

// ---------------------------------------------------------------------------
// Rust reference implementation (the ground truth for correctness tests)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreatureState {
    id: String,
    toughness: i32,
    damage_marked: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerState {
    id: String,
    life: i32,
}

fn rust_sba_creatures_to_destroy(creatures: &[CreatureState]) -> Vec<String> {
    creatures
        .iter()
        .filter(|c| c.toughness <= 0 || c.damage_marked >= c.toughness)
        .map(|c| c.id.clone())
        .collect()
}

fn rust_sba_players_losing(players: &[PlayerState]) -> Vec<String> {
    players
        .iter()
        .filter(|p| p.life <= 0)
        .map(|p| p.id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Prolog Machine helpers
// ---------------------------------------------------------------------------

use scryer_prolog::{LeafAnswer, Machine, MachineBuilder, Term};
use std::collections::BTreeMap;

/// Build a Machine with the SBA rules plus inline facts from the given
/// game state. Facts are embedded as Prolog clauses in the program string.
///
/// This avoids assertz/1 at query time, which triggers an internal panic in
/// scryer-prolog 0.10.0 when combined with findall/3.
fn build_machine_with_facts(creatures: &[CreatureState], players: &[PlayerState]) -> Machine {
    let mut program = String::from(SBA_RULES);

    // Append creature/3 facts
    for c in creatures {
        program.push_str(&format!(
            "creature({}, {}, {}).\n",
            c.id, c.toughness, c.damage_marked
        ));
    }

    // Append player/2 facts
    for p in players {
        program.push_str(&format!("player({}, {}).\n", p.id, p.life));
    }

    let mut machine = MachineBuilder::default().build();
    machine.consult_module_string("user", program);
    machine
}

/// Query `all_creatures_to_destroy(Ids)` and return the list as a Vec<String>.
fn query_creatures_to_destroy(machine: &mut Machine) -> Vec<String> {
    extract_id_list(machine, "all_creatures_to_destroy(Ids).")
}

/// Query `all_players_losing(Ids)` and return the list as a Vec<String>.
fn query_players_losing(machine: &mut Machine) -> Vec<String> {
    extract_id_list(machine, "all_players_losing(Ids).")
}

/// Run a query that binds `Ids` to a Prolog list and extract it as `Vec<String>`.
///
/// `findall/3` always succeeds with exactly one answer (a list, possibly empty),
/// so we only need to inspect the first result.
fn extract_id_list(machine: &mut Machine, goal: &str) -> Vec<String> {
    let first = machine.run_query(goal).next();

    match first {
        Some(Ok(LeafAnswer::LeafAnswer { bindings, .. })) => {
            extract_atom_list_binding(&bindings, "Ids")
        }
        Some(Ok(LeafAnswer::True)) | None => vec![],
        Some(Ok(LeafAnswer::False)) => vec![],
        Some(Ok(LeafAnswer::Exception(term))) => {
            panic!("Prolog exception in query `{goal}`: {:?}", term)
        }
        Some(Err(e)) => panic!("Error running query `{goal}`: {:?}", e),
    }
}

/// Extract a `Vec<String>` from a binding holding a Prolog list of atoms.
fn extract_atom_list_binding(bindings: &BTreeMap<String, Term>, var_name: &str) -> Vec<String> {
    match bindings.get(var_name) {
        Some(Term::List(items)) => items
            .iter()
            .map(|term| match term {
                Term::Atom(s) => s.clone(),
                other => panic!("Expected atom in list, got {:?}", other),
            })
            .collect(),
        // Empty list represented as the atom "[]"
        Some(Term::Atom(s)) if s == "[]" => vec![],
        Some(other) => panic!(
            "Expected list binding for `{var_name}`, got {:?}",
            other
        ),
        None => panic!("No binding for variable `{var_name}`"),
    }
}

// ---------------------------------------------------------------------------
// Correctness tests
// ---------------------------------------------------------------------------

#[test]
fn creature_with_lethal_damage_should_be_destroyed() {
    let creatures = vec![CreatureState {
        id: "creature_a".to_string(),
        toughness: 3,
        damage_marked: 3,
    }];
    let players = vec![];

    let mut machine = build_machine_with_facts(&creatures, &players);

    let mut prolog_result = query_creatures_to_destroy(&mut machine);
    let mut rust_result = rust_sba_creatures_to_destroy(&creatures);
    prolog_result.sort();
    rust_result.sort();

    assert_eq!(
        prolog_result, rust_result,
        "Prolog and Rust must agree: creature with lethal damage is destroyed"
    );
    assert!(prolog_result.contains(&"creature_a".to_string()));
}

#[test]
fn creature_with_insufficient_damage_should_not_be_destroyed() {
    let creatures = vec![CreatureState {
        id: "creature_b".to_string(),
        toughness: 3,
        damage_marked: 2,
    }];
    let players = vec![];

    let mut machine = build_machine_with_facts(&creatures, &players);

    let mut prolog_result = query_creatures_to_destroy(&mut machine);
    let mut rust_result = rust_sba_creatures_to_destroy(&creatures);
    prolog_result.sort();
    rust_result.sort();

    assert_eq!(
        prolog_result, rust_result,
        "Prolog and Rust must agree: creature with insufficient damage survives"
    );
    assert!(!prolog_result.contains(&"creature_b".to_string()));
}

#[test]
fn creature_with_zero_toughness_should_be_destroyed() {
    let creatures = vec![CreatureState {
        id: "creature_c".to_string(),
        toughness: 0,
        damage_marked: 0,
    }];
    let players = vec![];

    let mut machine = build_machine_with_facts(&creatures, &players);

    let mut prolog_result = query_creatures_to_destroy(&mut machine);
    let mut rust_result = rust_sba_creatures_to_destroy(&creatures);
    prolog_result.sort();
    rust_result.sort();

    assert_eq!(
        prolog_result, rust_result,
        "Prolog and Rust must agree: creature with 0 toughness is destroyed"
    );
    assert!(prolog_result.contains(&"creature_c".to_string()));
}

#[test]
fn player_with_zero_life_should_lose() {
    let creatures = vec![];
    let players = vec![PlayerState {
        id: "player_a".to_string(),
        life: 0,
    }];

    let mut machine = build_machine_with_facts(&creatures, &players);

    let mut prolog_result = query_players_losing(&mut machine);
    let mut rust_result = rust_sba_players_losing(&players);
    prolog_result.sort();
    rust_result.sort();

    assert_eq!(
        prolog_result, rust_result,
        "Prolog and Rust must agree: player with 0 life loses"
    );
    assert!(prolog_result.contains(&"player_a".to_string()));
}

#[test]
fn player_with_positive_life_should_not_lose() {
    let creatures = vec![];
    let players = vec![PlayerState {
        id: "player_b".to_string(),
        life: 1,
    }];

    let mut machine = build_machine_with_facts(&creatures, &players);

    let mut prolog_result = query_players_losing(&mut machine);
    let mut rust_result = rust_sba_players_losing(&players);
    prolog_result.sort();
    rust_result.sort();

    assert_eq!(
        prolog_result, rust_result,
        "Prolog and Rust must agree: player with 1 life does not lose"
    );
    assert!(!prolog_result.contains(&"player_b".to_string()));
}

#[test]
fn mixed_battlefield_only_lethal_creatures_destroyed() {
    let creatures = vec![
        CreatureState { id: "creature_ok".to_string(), toughness: 4, damage_marked: 2 },
        CreatureState { id: "creature_dead".to_string(), toughness: 2, damage_marked: 3 },
        CreatureState { id: "creature_zero".to_string(), toughness: -1, damage_marked: 0 },
    ];
    let players = vec![];

    let mut machine = build_machine_with_facts(&creatures, &players);

    let mut prolog_result = query_creatures_to_destroy(&mut machine);
    let mut rust_result = rust_sba_creatures_to_destroy(&creatures);
    prolog_result.sort();
    rust_result.sort();

    assert_eq!(
        prolog_result, rust_result,
        "Prolog and Rust must agree on mixed battlefield"
    );
    assert!(prolog_result.contains(&"creature_dead".to_string()));
    assert!(prolog_result.contains(&"creature_zero".to_string()));
    assert!(!prolog_result.contains(&"creature_ok".to_string()));
}

// ---------------------------------------------------------------------------
// Performance benchmark (std::time::Instant, no criterion)
// ---------------------------------------------------------------------------

#[test]
fn benchmark_prolog_vs_rust_sba() {
    use std::time::Instant;

    const ITERATIONS: usize = 50;

    let creatures = vec![
        CreatureState { id: "c1".to_string(), toughness: 4, damage_marked: 2 },  // survives
        CreatureState { id: "c2".to_string(), toughness: 2, damage_marked: 2 },  // lethal
        CreatureState { id: "c3".to_string(), toughness: 3, damage_marked: 1 },  // survives
        CreatureState { id: "c4".to_string(), toughness: 0, damage_marked: 0 },  // zero toughness
        CreatureState { id: "c5".to_string(), toughness: 5, damage_marked: 5 },  // lethal
        CreatureState { id: "c6".to_string(), toughness: 1, damage_marked: 0 },  // survives
        CreatureState { id: "c7".to_string(), toughness: -1, damage_marked: 0 }, // negative toughness
        CreatureState { id: "c8".to_string(), toughness: 6, damage_marked: 4 },  // survives
        CreatureState { id: "c9".to_string(), toughness: 2, damage_marked: 3 },  // lethal
        CreatureState { id: "c10".to_string(), toughness: 3, damage_marked: 3 }, // lethal
    ];
    let players = vec![
        PlayerState { id: "p1".to_string(), life: 5 }, // survives
        PlayerState { id: "p2".to_string(), life: 0 }, // loses
    ];

    // --- Rust benchmark ---
    let rust_start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = rust_sba_creatures_to_destroy(&creatures);
        let _ = rust_sba_players_losing(&players);
    }
    let rust_elapsed = rust_start.elapsed();

    // --- Prolog benchmark ---
    // Machine setup (consult) is OUTSIDE the timed loop; we time query execution only.
    // This models the intended production usage: load rules once, query per game tick.
    let mut machine = build_machine_with_facts(&creatures, &players);

    let prolog_start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = query_creatures_to_destroy(&mut machine);
        let _ = query_players_losing(&mut machine);
    }
    let prolog_elapsed = prolog_start.elapsed();

    let rust_per_iter = rust_elapsed / ITERATIONS as u32;
    let prolog_per_iter = prolog_elapsed / ITERATIONS as u32;

    println!("\n--- SBA Benchmark ({ITERATIONS} iterations, 10 creatures + 2 players) ---");
    println!(
        "  Rust   total: {:?} | per iteration: {:?}",
        rust_elapsed, rust_per_iter
    );
    println!(
        "  Prolog total: {:?} | per iteration: {:?}",
        prolog_elapsed, prolog_per_iter
    );
    if rust_per_iter.as_nanos() > 0 {
        let factor =
            prolog_per_iter.as_nanos() as f64 / rust_per_iter.as_nanos().max(1) as f64;
        println!("  Prolog is {factor:.1}x slower than Rust per iteration");
    }

    // Sanity-check: results must match
    let mut prolog_dead = query_creatures_to_destroy(&mut machine);
    let mut rust_dead = rust_sba_creatures_to_destroy(&creatures);
    prolog_dead.sort();
    rust_dead.sort();
    assert_eq!(prolog_dead, rust_dead, "Benchmark: creature results must agree");

    let mut prolog_losing = query_players_losing(&mut machine);
    let mut rust_losing = rust_sba_players_losing(&players);
    prolog_losing.sort();
    rust_losing.sort();
    assert_eq!(prolog_losing, rust_losing, "Benchmark: player results must agree");
}
