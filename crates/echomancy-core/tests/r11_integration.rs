//! R11 Replacement Effects Framework integration tests.
//!
//! These tests exercise the full pipeline:
//!   Game setup → replacement effect registration → event interception →
//!   verify game state.
//!
//! All tests use only the public API of `echomancy-core`.
//!
//! Test scenarios:
//!   r11_1 — Prevention shield reduces incoming damage (5 − 3 = 2)
//!   r11_2 — Prevention shield fully blocks damage (2 ≤ 3 shield remaining = 1)
//!   r11_3 — Regeneration shield keeps creature alive through lethal damage
//!   r11_4 — Regeneration shield does NOT save from zero-toughness SBA
//!   r11_5 — No replacement: creature dies normally from lethal damage
//!   r11_6 — Prevention shield on creature A does not affect damage to creature B

use echomancy_core::prelude::*;

// ============================================================================
// Shared setup helpers
// ============================================================================

/// Create a 2-player game advanced to P1's FirstMain step.
///
/// The game has no rules engine; replacement effects are registered directly
/// via the public test-helper methods on `Game`.
fn make_r11_game() -> (Game, String, String) {
    let p1 = "p1".to_owned();
    let p2 = "p2".to_owned();

    let mut game = Game::create("r11-test");
    game.add_player(&p1, "Alice").unwrap();
    game.add_player(&p2, "Bob").unwrap();

    // Minimal decks — enough to avoid decking out during setup.
    let p1_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p1-lib-{i}"), catalog::forest(), &p1))
        .collect();
    let p2_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p2-lib-{i}"), catalog::mountain(), &p2))
        .collect();
    game.assign_deck(&p1, p1_deck).unwrap();
    game.assign_deck(&p2, p2_deck).unwrap();

    game.start(&p1, Some(42)).unwrap();

    // Advance: Untap → Upkeep → Draw → FirstMain.
    while game.current_step() != Step::FirstMain {
        let current = game.current_player_id().to_owned();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&current),
        })
        .expect("should be able to advance to FirstMain");
    }

    assert_eq!(game.current_step(), Step::FirstMain);

    (game, p1, p2)
}

/// Add a 2/2 Bear creature to P1's battlefield, returning its instance ID.
fn add_bear(game: &mut Game, p1: &str, instance_id: &str) {
    let bear = CardInstance::new(instance_id, catalog::bear(), p1);
    game.add_permanent_to_battlefield(p1, bear)
        .expect("should be able to add bear to battlefield");
}

// ============================================================================
// R11.1 — Prevention shield reduces incoming damage (5 − 3 = 2)
// ============================================================================

/// R11.1: Register a 3-point prevention shield on a creature, then deal 5 damage.
///   - Verify: creature has 2 damage marked (5 − 3 = 2).
///   - Verify: prevention shield is consumed (removed from registry).
#[test]
fn r11_1_prevention_shield_reduces_damage() {
    let (mut game, p1, _p2) = make_r11_game();

    add_bear(&mut game, &p1, "bear-1");

    // Register a 3-point prevention shield.
    game.register_prevention_shield("bear-1", 3);
    assert_eq!(game.replacement_effect_count(), 1, "shield should be registered");

    // Deal 5 damage through the replacement framework.
    let final_damage = game.deal_damage_to_creature("bear-1", 5);

    // The shield prevents 3, so only 2 gets through.
    assert_eq!(
        final_damage, 2,
        "deal_damage_to_creature should return final (post-replacement) damage"
    );

    // Verify the damage is marked on the creature state.
    let state = game
        .permanent_state("bear-1")
        .expect("bear should still exist");
    let cs = state.creature_state().expect("bear should have creature state");
    assert_eq!(
        cs.damage_marked_this_turn(),
        2,
        "bear should have 2 damage marked (5 − 3 prevented by shield)"
    );

    // Shield is fully consumed because all 3 budget was used.
    assert_eq!(
        game.replacement_effect_count(),
        0,
        "prevention shield should be consumed after preventing 3 of 5 damage"
    );
}

// ============================================================================
// R11.2 — Prevention shield fully blocks small damage, shield remains
// ============================================================================

/// R11.2: Register a 3-point prevention shield, then deal only 2 damage.
///   - Verify: creature has 0 damage marked.
///   - Verify: shield remains in registry with 1 remaining.
#[test]
fn r11_2_prevention_shield_fully_blocks_small_damage_and_remains() {
    let (mut game, p1, _p2) = make_r11_game();

    add_bear(&mut game, &p1, "bear-2");

    // Register a 3-point prevention shield.
    game.register_prevention_shield("bear-2", 3);
    assert_eq!(game.replacement_effect_count(), 1);

    // Deal only 2 damage — shield absorbs it all.
    let final_damage = game.deal_damage_to_creature("bear-2", 2);

    assert_eq!(
        final_damage, 0,
        "all 2 damage should be prevented by the 3-point shield"
    );

    // Creature should have no damage marked.
    let state = game
        .permanent_state("bear-2")
        .expect("bear should still exist");
    let cs = state.creature_state().expect("bear should have creature state");
    assert_eq!(
        cs.damage_marked_this_turn(),
        0,
        "bear should have 0 damage marked (all prevented by shield)"
    );

    // Shield is still active with 1 remaining (3 − 2 = 1).
    assert_eq!(
        game.replacement_effect_count(),
        1,
        "prevention shield should still be active (not fully depleted)"
    );
    assert_eq!(
        game.prevention_shield_remaining("bear-2"),
        Some(1),
        "shield should have 1 remaining budget after preventing 2 of 3"
    );
}

// ============================================================================
// R11.3 — Regeneration shield keeps creature alive through lethal damage
// ============================================================================

/// R11.3: Register a regeneration shield on a 2/2 creature, deal 3 lethal damage,
///        then run SBA.
///   - Verify: creature is still on battlefield (regeneration fired).
///   - Verify: creature is tapped (CR 701.15a).
///   - Verify: creature has 0 damage marked (damage cleared by regeneration).
///   - Verify: regeneration shield is consumed.
#[test]
fn r11_3_regeneration_shield_survives_lethal_damage() {
    let (mut game, p1, _p2) = make_r11_game();

    add_bear(&mut game, &p1, "bear-3");

    // Register a regeneration shield.
    game.register_regeneration_shield("bear-3", &p1);
    assert!(
        game.has_regeneration_shield("bear-3"),
        "regeneration shield should be registered"
    );

    // Deal 3 damage (lethal for a 2/2) — marks damage directly, bypassing
    // replacement (we test the destroy-replacement path through SBA).
    // We need to mark damage directly to simulate having received lethal damage,
    // because deal_damage_to_creature only handles damage-prevention replacements.
    // Lethal-damage → destroy → regeneration is exercised by SBA.
    let _ = game.deal_damage_to_creature("bear-3", 3);

    // Verify the damage is actually marked (no prevention shield).
    let state = game
        .permanent_state("bear-3")
        .expect("bear should exist before SBA");
    let cs = state.creature_state().expect("bear should have creature state");
    assert_eq!(
        cs.damage_marked_this_turn(),
        3,
        "bear should have 3 damage marked before SBA"
    );

    // Run SBA — the lethal damage fires a destroy event, which the regen
    // shield intercepts. The bear stays alive, tapped, with damage cleared.
    game.run_sba();

    // Bear should still be on battlefield.
    let on_bf = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "bear-3");
    assert!(
        on_bf,
        "bear should still be on battlefield after regeneration shield fires"
    );

    // Bear should NOT be in graveyard.
    let in_gy = game
        .graveyard(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "bear-3");
    assert!(
        !in_gy,
        "bear should NOT be in graveyard after regeneration saved it"
    );

    // Bear should be tapped (CR 701.15a).
    let state_after = game
        .permanent_state("bear-3")
        .expect("bear state should exist after regeneration");
    assert!(
        state_after.is_tapped(),
        "bear should be tapped after regeneration fires (CR 701.15a)"
    );

    // Bear should have 0 damage marked (cleared by regeneration).
    let cs_after = state_after
        .creature_state()
        .expect("bear should have creature state after regeneration");
    assert_eq!(
        cs_after.damage_marked_this_turn(),
        0,
        "bear should have 0 damage after regeneration cleared it (CR 701.15a)"
    );

    // Regeneration shield should be consumed.
    assert!(
        !game.has_regeneration_shield("bear-3"),
        "regeneration shield should be consumed after firing"
    );
}

// ============================================================================
// R11.4 — Regeneration shield does NOT save from zero-toughness SBA
// ============================================================================

/// R11.4: Register a regeneration shield, then reduce effective toughness to 0
///        via a Layer 7b SetPowerToughness effect. Run SBA.
///   - Verify: creature goes to graveyard (zero toughness bypasses regeneration).
///   - Verify: regeneration shield is NOT consumed.
///
/// Per CR 704.5f, zero-toughness uses "put into graveyard" not "destroy", so
/// replacement effects cannot intercept it.
#[test]
fn r11_4_regeneration_shield_does_not_save_zero_toughness() {
    let (mut game, p1, _p2) = make_r11_game();

    // Add a 2/2 bear.
    add_bear(&mut game, &p1, "bear-4");

    // Register a regeneration shield.
    game.register_regeneration_shield("bear-4", &p1);
    assert!(game.has_regeneration_shield("bear-4"), "shield should be registered");

    // Apply a Layer 7b effect that sets the bear's P/T to 0/0 until end of turn.
    // This simulates a spell-resolution effect that reduces toughness to 0.
    game.inject_set_pt_effect("bear-4", 0, 0);

    // Effective toughness should now be 0.
    assert_eq!(
        game.effective_toughness("bear-4"),
        Some(0),
        "effective toughness should be 0 after Layer 7b set effect"
    );

    // Run SBA — zero toughness fires CR 704.5f "put into graveyard" (not destroy).
    // Regeneration shield should NOT intercept this.
    game.run_sba();

    // Bear should be in graveyard.
    let in_gy = game
        .graveyard(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "bear-4");
    assert!(
        in_gy,
        "bear should be in graveyard — zero toughness bypasses regeneration (CR 704.5f)"
    );

    // Bear should NOT be on battlefield.
    let on_bf = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "bear-4");
    assert!(
        !on_bf,
        "bear should NOT be on battlefield after zero-toughness SBA"
    );

    // Regeneration shield was NOT consumed by the zero-toughness SBA (CR 704.5f
    // bypasses destroy replacement). But the source left the battlefield, so
    // all replacement effects for that source are cleaned up.
    assert!(
        !game.has_regeneration_shield("bear-4"),
        "regen shield cleaned up when source leaves battlefield"
    );
}

// ============================================================================
// R11.5 — Baseline: no replacement, creature dies normally from lethal damage
// ============================================================================

/// R11.5: Deal lethal damage to a creature with no replacement effects, run SBA.
///   - Verify: creature goes to graveyard.
///   - Verify: no replacement effects consumed.
#[test]
fn r11_5_no_replacement_creature_dies_normally() {
    let (mut game, p1, _p2) = make_r11_game();

    add_bear(&mut game, &p1, "bear-5");

    assert_eq!(game.replacement_effect_count(), 0, "no effects should be registered");

    // Deal 3 damage (lethal for a 2/2).
    game.deal_damage_to_creature("bear-5", 3);

    // Run SBA — no replacement, creature should die.
    game.run_sba();

    // Bear should be in graveyard.
    let in_gy = game
        .graveyard(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "bear-5");
    assert!(
        in_gy,
        "bear should be in graveyard after lethal damage with no replacement effects"
    );

    // Bear should NOT be on battlefield.
    let on_bf = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "bear-5");
    assert!(
        !on_bf,
        "bear should NOT be on battlefield after dying to lethal damage"
    );

    // No replacement effects were consumed (none existed).
    assert_eq!(game.replacement_effect_count(), 0);
}

// ============================================================================
// R11.6 — Prevention shield on creature A does not affect damage to creature B
// ============================================================================

/// R11.6: Register a prevention shield on creature A, deal damage to creature B.
///   - Verify: creature B takes full damage.
///   - Verify: shield on creature A is not consumed.
#[test]
fn r11_6_prevention_shield_does_not_affect_wrong_target() {
    let (mut game, p1, _p2) = make_r11_game();

    add_bear(&mut game, &p1, "bear-a");
    add_bear(&mut game, &p1, "bear-b");

    // Register a 3-point shield on creature A only.
    game.register_prevention_shield("bear-a", 3);
    assert_eq!(game.replacement_effect_count(), 1, "one shield registered (for A only)");

    // Deal 5 damage to creature B — no shield applies.
    let final_damage_b = game.deal_damage_to_creature("bear-b", 5);

    // Creature B takes full 5 damage.
    assert_eq!(
        final_damage_b, 5,
        "creature B should take full 5 damage (shield only protects creature A)"
    );

    let state_b = game
        .permanent_state("bear-b")
        .expect("bear-b should exist");
    let cs_b = state_b.creature_state().expect("bear-b should have creature state");
    assert_eq!(
        cs_b.damage_marked_this_turn(),
        5,
        "bear-b should have 5 damage marked"
    );

    // Shield on A is still intact (1 effect, same budget).
    assert_eq!(
        game.replacement_effect_count(),
        1,
        "shield on creature A should NOT be consumed when creature B took damage"
    );
    assert_eq!(
        game.prevention_shield_remaining("bear-a"),
        Some(3),
        "shield on creature A should still have 3 remaining budget"
    );
}
