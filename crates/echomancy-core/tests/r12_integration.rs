//! R12 Prevention Effects integration tests.
//!
//! These tests exercise the full R12 pipeline:
//!   Fog (AllCombatDamage filter) and Guardian Shield (DamageToPermanent, UntilEndOfTurn)
//!
//! All tests use the public API of `echomancy-core`.
//!
//! Test scenarios:
//!   r12_1 — AllCombatDamage filter prevents combat damage (is_combat=true)
//!   r12_2 — AllCombatDamage filter does NOT prevent spell damage (is_combat=false)
//!   r12_3 — Guardian Shield (amount=0, UntilEndOfTurn) prevents combat damage to target
//!   r12_4 — Guardian Shield prevents spell damage to target
//!   r12_5 — Guardian Shield does NOT protect other creatures
//!   r12_6 — Guardian Shield cleanup: effect removed at end of turn
//!   r12_7 — Fog + Mending Light both active: timestamp ordering applies
//!   r12_8 — DamageToPlayer UntilDepleted shield depletes on player damage
//!   r12_9 — Fog card cast via CLIPS registers AllCombatDamage effect
//!   r12_10 — Guardian Shield card cast via CLIPS registers targeted prevention

use echomancy_core::prelude::*;

// ============================================================================
// Shared setup helpers
// ============================================================================

/// Create a 2-player game advanced to P1's FirstMain step.
fn make_r12_game() -> (Game, String, String) {
    let p1 = "p1".to_owned();
    let p2 = "p2".to_owned();

    let mut game = Game::create("r12-test");
    game.add_player(&p1, "Alice").unwrap();
    game.add_player(&p2, "Bob").unwrap();

    let p1_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p1-lib-{i}"), catalog::forest(), &p1))
        .collect();
    let p2_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p2-lib-{i}"), catalog::mountain(), &p2))
        .collect();
    game.assign_deck(&p1, p1_deck).unwrap();
    game.assign_deck(&p2, p2_deck).unwrap();

    game.start(&p1, Some(42)).unwrap();

    while game.current_step() != Step::FirstMain {
        let current = game.current_player_id().to_string();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&current),
        })
        .expect("should be able to advance to FirstMain");
    }

    assert_eq!(game.current_step(), Step::FirstMain);

    (game, p1, p2)
}

/// Add a 2/2 Bear creature to the given player's battlefield.
fn add_bear(game: &mut Game, player_id: &str, instance_id: &str) {
    let bear = CardInstance::new(instance_id, catalog::bear(), player_id);
    game.add_permanent_to_battlefield(player_id, bear)
        .expect("should be able to add bear to battlefield");
}

/// Add a creature with specified P/T to the given player's battlefield.
fn add_creature(game: &mut Game, player_id: &str, instance_id: &str, power: u32, toughness: u32) {
    let def = CardDefinition::new(instance_id, "Test Creature", vec![CardType::Creature])
        .with_power_toughness(power, toughness);
    let creature = CardInstance::new(instance_id, def, player_id);
    game.add_permanent_to_battlefield(player_id, creature)
        .expect("should be able to add creature to battlefield");
}

// ============================================================================
// R12.1 — AllCombatDamage filter prevents combat damage
// ============================================================================

/// R12.1: Register an AllCombatDamage prevention effect, then deal combat damage.
///   - Verify: damage is fully prevented.
///   - Verify: the UntilEndOfTurn effect remains active (not consumed).
#[test]
fn r12_1_all_combat_damage_filter_prevents_combat_damage() {
    let (mut game, p1, _p2) = make_r12_game();

    add_bear(&mut game, &p1, "bear-1");

    // Register an AllCombatDamage prevention effect (like Fog).
    game.register_all_combat_damage_prevention("fog-source-1", &p1);
    assert_eq!(game.replacement_effect_count(), 1, "AllCombatDamage effect should be registered");

    // deal_damage_to_creature uses is_combat=false (non-combat helper).
    // To test combat interception we use the direct apply with is_combat=true via the public API.
    // R12.1 exercises the framework-level guarantee.
    let final_damage = game.deal_combat_damage_to_creature("bear-1", 4);

    assert_eq!(
        final_damage, 0,
        "AllCombatDamage filter should prevent all 4 combat damage"
    );

    let state = game.permanent_state("bear-1").expect("bear should exist");
    let cs = state.creature_state().expect("creature state");
    assert_eq!(cs.damage_marked_this_turn(), 0, "no damage should be marked");

    // UntilEndOfTurn: effect remains active.
    assert_eq!(
        game.replacement_effect_count(),
        1,
        "UntilEndOfTurn effect should persist after preventing combat damage"
    );
}

// ============================================================================
// R12.2 — AllCombatDamage filter does NOT prevent spell damage
// ============================================================================

/// R12.2: Register an AllCombatDamage prevention effect, then deal non-combat damage.
///   - Verify: damage is NOT prevented.
///   - Verify: effect count unchanged.
#[test]
fn r12_2_all_combat_damage_filter_does_not_prevent_spell_damage() {
    let (mut game, p1, _p2) = make_r12_game();

    add_bear(&mut game, &p1, "bear-2");

    game.register_all_combat_damage_prevention("fog-source-2", &p1);

    // deal_damage_to_creature uses is_combat=false (spell/non-combat).
    let final_damage = game.deal_damage_to_creature("bear-2", 3);

    assert_eq!(
        final_damage, 3,
        "AllCombatDamage filter should NOT prevent spell damage (is_combat=false)"
    );

    let state = game.permanent_state("bear-2").expect("bear should exist");
    let cs = state.creature_state().expect("creature state");
    assert_eq!(cs.damage_marked_this_turn(), 3, "full 3 damage should be marked");

    // Effect count unchanged (UntilEndOfTurn, and it didn't match).
    assert_eq!(game.replacement_effect_count(), 1, "effect still active");
}

// ============================================================================
// R12.3 — Guardian Shield (amount=0, UntilEndOfTurn) prevents combat damage
// ============================================================================

/// R12.3: Register a full-prevention shield (amount=0, UntilEndOfTurn) on a creature,
///        then deal combat damage to that creature.
///   - Verify: damage fully prevented.
///   - Verify: effect persists (UntilEndOfTurn, not UntilDepleted).
#[test]
fn r12_3_guardian_shield_prevents_combat_damage_to_target() {
    let (mut game, p1, _p2) = make_r12_game();

    add_bear(&mut game, &p1, "bear-3");

    // Register a full-turn prevention shield (Guardian Shield style).
    game.register_full_turn_prevention_shield("gs-source-3", "bear-3");
    assert_eq!(game.replacement_effect_count(), 1);

    let final_damage = game.deal_combat_damage_to_creature("bear-3", 4);
    assert_eq!(final_damage, 0, "Guardian Shield should prevent 4 combat damage");

    let state = game.permanent_state("bear-3").expect("bear should exist");
    let cs = state.creature_state().expect("creature state");
    assert_eq!(cs.damage_marked_this_turn(), 0, "no damage should be marked");

    // UntilEndOfTurn: effect persists.
    assert_eq!(game.replacement_effect_count(), 1, "UntilEndOfTurn shield persists");
}

// ============================================================================
// R12.4 — Guardian Shield prevents spell damage to target
// ============================================================================

/// R12.4: Guardian Shield also prevents non-combat spell damage to the target.
#[test]
fn r12_4_guardian_shield_prevents_spell_damage_to_target() {
    let (mut game, p1, _p2) = make_r12_game();

    add_bear(&mut game, &p1, "bear-4");

    game.register_full_turn_prevention_shield("gs-source-4", "bear-4");

    // Non-combat spell damage (is_combat=false in deal_damage_to_creature).
    let final_damage = game.deal_damage_to_creature("bear-4", 3);
    assert_eq!(final_damage, 0, "Guardian Shield should prevent spell damage too");

    assert_eq!(game.replacement_effect_count(), 1, "shield persists");
}

// ============================================================================
// R12.5 — Guardian Shield does NOT protect other creatures
// ============================================================================

/// R12.5: A Guardian Shield on creature-A should not affect damage to creature-B.
#[test]
fn r12_5_guardian_shield_does_not_protect_other_creatures() {
    let (mut game, p1, _p2) = make_r12_game();

    add_bear(&mut game, &p1, "bear-5a");
    add_bear(&mut game, &p1, "bear-5b");

    // Shield on bear-5a only.
    game.register_full_turn_prevention_shield("gs-source-5", "bear-5a");

    // Damage to bear-5b should not be prevented.
    let final_damage = game.deal_damage_to_creature("bear-5b", 3);
    assert_eq!(
        final_damage, 3,
        "Guardian Shield on bear-5a should not protect bear-5b"
    );

    let state = game.permanent_state("bear-5b").expect("bear-5b should exist");
    let cs = state.creature_state().expect("creature state");
    assert_eq!(cs.damage_marked_this_turn(), 3, "bear-5b should have 3 damage marked");
}

// ============================================================================
// R12.6 — Guardian Shield expires at cleanup
// ============================================================================

/// R12.6: A UntilEndOfTurn prevention effect is removed during the Cleanup step.
#[test]
fn r12_6_guardian_shield_expires_at_cleanup() {
    let (mut game, p1, _p2) = make_r12_game();

    add_bear(&mut game, &p1, "bear-6");

    game.register_full_turn_prevention_shield("gs-source-6", "bear-6");
    assert_eq!(game.replacement_effect_count(), 1, "shield registered");

    // Simulate Cleanup step expiry.
    game.cleanup_expired_replacement_effects_pub();

    assert_eq!(
        game.replacement_effect_count(),
        0,
        "UntilEndOfTurn effect should be removed during Cleanup"
    );
}

// ============================================================================
// R12.7 — Fog + Mending Light both active: timestamp ordering
// ============================================================================

/// R12.7: Two prevention effects are both active. The older one (lower timestamp)
///        applies first. If it prevents all damage, the second is not consumed.
///
/// Casting order: Mending Light first (ts=0, 3-point depleting), then Fog (ts=1, all-combat).
/// Damage: 5 combat damage.
/// Expected: Mending Light applies first (3 prevented, 2 remain), then AllCombatDamage
///           prevents the remaining 2. Mending Light is consumed. AllCombatDamage persists.
#[test]
fn r12_7_fog_and_mending_light_both_active_timestamp_ordering() {
    let (mut game, p1, _p2) = make_r12_game();

    add_creature(&mut game, &p1, "creature-7", 4, 10_u32);

    // Register Mending Light first (lower timestamp = applied first).
    game.register_prevention_shield("creature-7", 3); // UntilDepleted{3}
    // Then register Fog (AllCombatDamage, higher timestamp).
    game.register_all_combat_damage_prevention("fog-source-7", &p1);

    assert_eq!(game.replacement_effect_count(), 2);

    // Deal 5 combat damage.
    let final_damage = game.deal_combat_damage_to_creature("creature-7", 5);
    assert_eq!(
        final_damage, 0,
        "combined shields should prevent all 5 damage"
    );

    // Mending Light (UntilDepleted{3}) is consumed (3 damage prevented).
    // AllCombatDamage (UntilEndOfTurn) should still be active.
    assert_eq!(
        game.replacement_effect_count(),
        1,
        "Mending Light depleted, AllCombatDamage persists"
    );
}

// ============================================================================
// R12.8 — DamageToPlayer UntilDepleted shield depletes correctly
// ============================================================================

/// R12.8: A player-targeted UntilDepleted shield (3 points) reduces 5-damage to 2.
#[test]
fn r12_8_damage_to_player_until_depleted_reduces_damage() {
    let (mut game, p1, _p2) = make_r12_game();

    let initial_life = game.player_life_total(&p1).expect("p1 should exist");

    // Register a 3-point player shield.
    game.register_player_prevention_shield(&p1, 3);
    assert_eq!(game.replacement_effect_count(), 1, "shield registered");

    // Deal 5 damage to the player (non-combat).
    game.deal_damage_to_player_through_framework(&p1, 5);

    let new_life = game.player_life_total(&p1).expect("p1 should exist");
    assert_eq!(
        new_life,
        initial_life - 2,
        "player should lose only 2 life (5 - 3 prevented by shield)"
    );

    // Shield depleted (3 used = budget exhausted).
    assert_eq!(
        game.replacement_effect_count(),
        0,
        "DamageToPlayer shield should be consumed"
    );
}

// ============================================================================
// R12.9 — DamageToPlayer UntilEndOfTurn + amount:0 prevents all player damage
// ============================================================================

/// R12.9-player: A DamageToPlayer effect with UntilEndOfTurn and amount=0
///               prevents all damage to that player until Cleanup.
#[test]
fn r12_player_shield_until_end_of_turn_prevents_all_damage() {
    let (mut game, p1, _p2) = make_r12_game();

    let initial_life = game.player_life_total(&p1).expect("p1 should exist");

    // Register a full-turn player prevention shield (amount=0 = prevent all).
    game.register_player_until_end_of_turn_shield(&p1);
    assert_eq!(game.replacement_effect_count(), 1, "shield registered");

    // Deal 10 damage to the player — all should be prevented.
    game.deal_damage_to_player_through_framework(&p1, 10);

    let new_life = game.player_life_total(&p1).expect("p1 should exist");
    assert_eq!(new_life, initial_life, "player should take no damage (amount=0 shield)");

    // Effect persists (UntilEndOfTurn).
    assert_eq!(game.replacement_effect_count(), 1, "UntilEndOfTurn shield persists");

    // Expires at cleanup.
    game.cleanup_expired_replacement_effects_pub();
    assert_eq!(game.replacement_effect_count(), 0, "shield removed at cleanup");
}

// ============================================================================
// R12.10 — Fog card cast via CLIPS registers AllCombatDamage effect
// ============================================================================

/// R12.10: Cast the actual Fog card via the CLIPS rules pipeline and verify
///         an AllCombatDamage replacement effect is registered.
#[test]
fn r12_9_fog_card_registers_all_combat_damage_effect_via_clips() {
    use echomancy_core::infrastructure::create_rules_engine;

    let (mut game, p1, p2) = make_r12_game();

    // Wire up the rules engine.
    let engine = create_rules_engine(&["fog"]).expect("rules engine should load");
    game.set_rules_engine(engine);

    // Add a Forest to p1's battlefield so they can tap for {G}.
    let forest = CardInstance::new("forest-r12-9", catalog::forest(), &p1);
    game.add_permanent_to_battlefield(&p1, forest).expect("add forest");

    // Tap the forest to generate {G}.
    game.apply(Action::ActivateAbility {
        player_id: PlayerId::new(&p1),
        permanent_id: CardInstanceId::new("forest-r12-9"),
        ability_index: 0,
    })
    .expect("tap forest for green mana");

    // Add a Fog card to p1's hand.
    let fog_card = CardInstance::new("fog-1", catalog::fog(), &p1);
    game.add_card_to_hand(&p1, fog_card).expect("add fog to hand");

    // P1 casts Fog (no target required).
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("fog-1"),
        targets: vec![],
        x_value: 0,
    })
    .expect("cast Fog should succeed");

    // Both players pass priority — stack resolves Fog.
    game.apply(Action::PassPriority { player_id: PlayerId::new(&p1) }).unwrap();
    game.apply(Action::PassPriority { player_id: PlayerId::new(&p2) }).unwrap();

    assert_eq!(game.stack().len(), 0, "stack should be empty after Fog resolves");

    // Verify an AllCombatDamage effect is now in the registry.
    assert_eq!(
        game.replacement_effect_count(),
        1,
        "Fog should register exactly one AllCombatDamage effect"
    );
    assert!(
        game.has_all_combat_damage_prevention(),
        "AllCombatDamage filter should be active after Fog resolves"
    );
}

// ============================================================================
// R12.10 — Guardian Shield card cast via CLIPS registers targeted prevention
// ============================================================================

/// R12.10: Cast the actual Guardian Shield card via the CLIPS rules pipeline,
///         targeting a creature. Verify a DamageToPermanent(amount=0, UntilEndOfTurn)
///         replacement effect is registered for that creature.
#[test]
fn r12_10_guardian_shield_registers_targeted_prevention_via_clips() {
    use echomancy_core::infrastructure::create_rules_engine;

    let (mut game, p1, p2) = make_r12_game();

    let engine = create_rules_engine(&["guardian-shield"]).expect("rules engine should load");
    game.set_rules_engine(engine);

    // Put a bear on the battlefield to be the target.
    add_bear(&mut game, &p1, "target-bear");

    // Add a Plains and a Forest to pay {1}{W} for Guardian Shield.
    let plains = CardInstance::new("plains-r12-10", catalog::plains(), &p1);
    game.add_permanent_to_battlefield(&p1, plains).expect("add plains");
    let forest_for_1 = CardInstance::new("forest-r12-10", catalog::forest(), &p1);
    game.add_permanent_to_battlefield(&p1, forest_for_1).expect("add forest");

    // Tap Plains for {W}.
    game.apply(Action::ActivateAbility {
        player_id: PlayerId::new(&p1),
        permanent_id: CardInstanceId::new("plains-r12-10"),
        ability_index: 0,
    })
    .expect("tap plains for white mana");

    // Tap Forest for {G} (used to pay the {1} generic).
    game.apply(Action::ActivateAbility {
        player_id: PlayerId::new(&p1),
        permanent_id: CardInstanceId::new("forest-r12-10"),
        ability_index: 0,
    })
    .expect("tap forest for generic mana");

    // Add Guardian Shield to p1's hand.
    let gs_card = CardInstance::new("gs-spell-1", catalog::guardian_shield(), &p1);
    game.add_card_to_hand(&p1, gs_card).expect("add guardian shield to hand");

    // P1 casts Guardian Shield targeting target-bear.
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("gs-spell-1"),
        targets: vec![Target::creature("target-bear")],
        x_value: 0,
    })
    .expect("cast Guardian Shield should succeed");

    // Both players pass priority — stack resolves.
    game.apply(Action::PassPriority { player_id: PlayerId::new(&p1) }).unwrap();
    game.apply(Action::PassPriority { player_id: PlayerId::new(&p2) }).unwrap();

    assert_eq!(game.stack().len(), 0, "stack should be empty");

    assert_eq!(
        game.replacement_effect_count(),
        1,
        "Guardian Shield should register exactly one prevention effect"
    );

    // The shield should prevent any damage (including spell damage) to target-bear.
    let final_damage = game.deal_damage_to_creature("target-bear", 5);
    assert_eq!(
        final_damage, 0,
        "Guardian Shield should prevent 5 spell damage to target-bear"
    );

    // Shield persists (UntilEndOfTurn).
    assert_eq!(
        game.replacement_effect_count(),
        1,
        "UntilEndOfTurn Guardian Shield persists after preventing damage"
    );
}
