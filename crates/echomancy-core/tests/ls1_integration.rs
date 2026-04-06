//! LS1 Layer System integration tests.
//!
//! These tests cover the 6 manual UI scenarios from the LS1 test plan section
//! in `docs/TEST_PLAN.md`. Each test exercises the full pipeline:
//!   Game setup → rules engine → cast spell → resolve → layer-aware query
//!
//! All tests use only the public API of `echomancy-core`.

use echomancy_core::prelude::*;
use echomancy_core::domain::game::automation::auto_resolve_stack;

// ============================================================================
// Shared test setup helper
// ============================================================================

/// Create a 2-player game with a real CLIPS engine, advanced to P1's FirstMain.
///
/// The engine is loaded with all card IDs used in LS1 scenarios.
fn make_ls1_game() -> (Game, String, String) {
    let p1 = "p1".to_owned();
    let p2 = "p2".to_owned();

    let mut game = Game::create("ls1-test");
    game.add_player(&p1, "Alice").unwrap();
    game.add_player(&p2, "Bob").unwrap();

    // Minimal libraries — enough to avoid decking out during setup.
    let p1_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p1-lib-{i}"), catalog::forest(), &p1))
        .collect();
    let p2_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p2-lib-{i}"), catalog::mountain(), &p2))
        .collect();
    game.assign_deck(&p1, p1_deck).unwrap();
    game.assign_deck(&p2, p2_deck).unwrap();

    game.start(&p1, Some(42)).unwrap();

    let engine = create_rules_engine(&[
        "bear",
        "forest",
        "mountain",
        "turn-to-frog",
        "giant-growth",
        "titanic-growth",
        "twisted-image",
        "lightning-strike",
        "ironbark-wall",
        "ancient-guardian",
    ])
    .expect("CLIPS engine should initialise");
    game.set_rules_engine(engine);

    // Advance: Untap → Upkeep → Draw → FirstMain.
    while game.current_step() != echomancy_core::prelude::Step::FirstMain {
        let current = game.current_player_id().to_owned();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&current),
        })
        .expect("should be able to advance to FirstMain");
    }

    assert_eq!(game.current_step(), Step::FirstMain, "setup must land on FirstMain");

    (game, p1, p2)
}

/// Advance the game past all remaining steps of the current turn until the
/// Cleanup step is reached and processed. Effects with `UntilEndOfTurn`
/// duration expire during Cleanup (CR 514.2).
fn advance_to_cleanup(game: &mut Game, active_player: &str) {
    while game.current_step() != Step::Cleanup {
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(active_player),
        })
        .expect("AdvanceStep should succeed");
    }
    // Trigger the Cleanup step itself so effects expire.
    game.apply(Action::AdvanceStep {
        player_id: PlayerId::new(active_player),
    })
    .expect("Cleanup AdvanceStep should succeed");
}

// ============================================================================
// LS1.1 — Turn to Frog sets base P/T (Layer 7b)
// ============================================================================

/// LS1.1: Cast Turn to Frog on a Bear (2/2).
///   - Verify: effective_power = 1, effective_toughness = 1 (Layer 7b set 1/1)
///   - Verify: Bear's effective abilities do NOT include any abilities it had
///   - Advance to cleanup: effective P/T returns to 2/2 (effect expired)
#[test]
fn ls1_1_turn_to_frog_sets_base_pt() {
    let (mut game, p1, _p2) = make_ls1_game();

    // Bear (2/2) on battlefield.
    let bear = CardInstance::new("bear-1", catalog::bear(), &p1);
    game.add_permanent_to_battlefield(&p1, bear).unwrap();

    // Turn to Frog costs {1}{U}.
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Blue, 1).unwrap();

    let ttf = CardInstance::new("ttf-1", catalog::turn_to_frog(), &p1);
    game.add_card_to_hand(&p1, ttf).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("ttf-1"),
        targets: vec![Target::creature("bear-1")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Turn to Frog");

    assert_eq!(game.stack().len(), 1, "Turn to Frog should be on stack");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // Layer 7b: Turn to Frog sets base P/T to 1/1.
    assert_eq!(
        game.effective_power("bear-1"),
        Some(1),
        "effective power should be 1 after Turn to Frog (Layer 7b)"
    );
    assert_eq!(
        game.effective_toughness("bear-1"),
        Some(1),
        "effective toughness should be 1 after Turn to Frog (Layer 7b)"
    );

    // Layer 6: Turn to Frog removes all abilities.
    // A Bear (2/2) has no static abilities, so we just verify the abilities list is empty.
    let abilities = game.effective_abilities("bear-1").unwrap_or_default();
    assert!(
        abilities.is_empty(),
        "Bear should have no abilities after Turn to Frog (Layer 6), got: {abilities:?}"
    );

    // Advance to Cleanup: effect expires, Bear returns to 2/2.
    advance_to_cleanup(&mut game, &p1);

    assert_eq!(
        game.effective_power("bear-1"),
        Some(2),
        "Bear should return to power 2 after Cleanup (effect expired)"
    );
    assert_eq!(
        game.effective_toughness("bear-1"),
        Some(2),
        "Bear should return to toughness 2 after Cleanup (effect expired)"
    );
}

// ============================================================================
// LS1.2 — Turn to Frog + Giant Growth (Layer 7b before 7c)
// ============================================================================

/// LS1.2: Cast Turn to Frog on a Bear (becomes 1/1), then cast Giant Growth (+3/+3).
///   - Verify: effective_power = 4, effective_toughness = 4 (NOT 5/5)
///     because Layer 7b (set) applies before Layer 7c (modify).
///   - Advance to cleanup: Bear returns to 2/2 (both effects expire).
#[test]
fn ls1_2_turn_to_frog_plus_giant_growth() {
    let (mut game, p1, _p2) = make_ls1_game();

    // Bear (2/2) on battlefield.
    let bear = CardInstance::new("bear-2", catalog::bear(), &p1);
    game.add_permanent_to_battlefield(&p1, bear).unwrap();

    // Cast Turn to Frog ({1}{U}) on the Bear.
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Blue, 1).unwrap();
    let ttf = CardInstance::new("ttf-2", catalog::turn_to_frog(), &p1);
    game.add_card_to_hand(&p1, ttf).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("ttf-2"),
        targets: vec![Target::creature("bear-2")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Turn to Frog");

    auto_resolve_stack(&mut game);

    // Turn to Frog applied: Bear should be 1/1 now.
    assert_eq!(game.effective_power("bear-2"), Some(1), "after Turn to Frog: power = 1");
    assert_eq!(game.effective_toughness("bear-2"), Some(1), "after Turn to Frog: toughness = 1");

    // Cast Giant Growth ({G}) on the same Bear.
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    let gg = CardInstance::new("gg-2", catalog::giant_growth(), &p1);
    game.add_card_to_hand(&p1, gg).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("gg-2"),
        targets: vec![Target::creature("bear-2")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Giant Growth");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty");

    // Layer 7b sets base to 1/1; Layer 7c adds +3/+3 → effective 4/4, NOT 5/5.
    assert_eq!(
        game.effective_power("bear-2"),
        Some(4),
        "effective power should be 4 (1 from Layer 7b + 3 from Layer 7c)"
    );
    assert_eq!(
        game.effective_toughness("bear-2"),
        Some(4),
        "effective toughness should be 4 (1 from Layer 7b + 3 from Layer 7c)"
    );

    // Advance to Cleanup: both effects expire, Bear returns to 2/2.
    advance_to_cleanup(&mut game, &p1);

    assert_eq!(
        game.effective_power("bear-2"),
        Some(2),
        "Bear should return to power 2 after Cleanup"
    );
    assert_eq!(
        game.effective_toughness("bear-2"),
        Some(2),
        "Bear should return to toughness 2 after Cleanup"
    );
}

// ============================================================================
// LS1.3 — Twisted Image switches P/T (Layer 7d) + SBA kills the Wall
// ============================================================================

/// LS1.3: Cast Ironbark Wall (0/4), then cast Twisted Image on it.
///   - After resolution: effective_power = 4, effective_toughness = 0 (Layer 7d switch)
///   - SBA: Wall has 0 effective toughness → immediately destroyed (goes to graveyard)
///   - Wall should NOT be on battlefield after resolution.
#[test]
fn ls1_3_twisted_image_switches_pt_and_sba_kills_wall() {
    let (mut game, p1, _p2) = make_ls1_game();

    // Ironbark Wall (0/4) on battlefield.
    let wall = CardInstance::new("wall-3", catalog::ironbark_wall(), &p1);
    game.add_permanent_to_battlefield(&p1, wall).unwrap();

    // Verify base P/T before Twisted Image.
    assert_eq!(game.effective_power("wall-3"), Some(0), "Ironbark Wall should have power 0 before Twisted Image");
    assert_eq!(game.effective_toughness("wall-3"), Some(4), "Ironbark Wall should have toughness 4 before Twisted Image");

    // Add a library card so Twisted Image's draw doesn't fail.
    let lib_card = CardInstance::new("lib-3", catalog::forest(), &p1);
    game.add_card_to_library_top(&p1, lib_card).unwrap();

    // Cast Twisted Image ({U}).
    game.add_mana(&p1, ManaColor::Blue, 1).unwrap();
    let ti = CardInstance::new("ti-3", catalog::twisted_image(), &p1);
    game.add_card_to_hand(&p1, ti).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("ti-3"),
        targets: vec![Target::creature("wall-3")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Twisted Image");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // SBA: Wall with 0 toughness should have been moved to graveyard, not battlefield.
    let wall_on_bf = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "wall-3");
    assert!(
        !wall_on_bf,
        "Ironbark Wall should NOT be on battlefield after SBA (0 effective toughness)"
    );

    let wall_in_gy = game
        .graveyard(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "wall-3");
    assert!(
        wall_in_gy,
        "Ironbark Wall should be in P1's graveyard after SBA killed it (0 toughness)"
    );
}

// ============================================================================
// LS1.4 — Turn to Frog removes Indestructible (Layer 6)
// ============================================================================

/// LS1.4: Cast Ancient Guardian (4/5 Indestructible), then cast Turn to Frog.
///   - Verify: effective_power = 1, effective_toughness = 1 (Layer 7b)
///   - Verify: effective_abilities does NOT contain Indestructible (Layer 6 removed it)
///   - Deal damage via Lightning Strike → Guardian dies (no longer Indestructible).
#[test]
fn ls1_4_turn_to_frog_removes_indestructible() {
    let (mut game, p1, _p2) = make_ls1_game();

    // Ancient Guardian (4/5 Indestructible) on P1's battlefield.
    let guardian = CardInstance::new("guardian-4", catalog::ancient_guardian(), &p1);
    game.add_permanent_to_battlefield(&p1, guardian).unwrap();

    // Verify it starts as Indestructible.
    let abilities_before = game.effective_abilities("guardian-4").unwrap_or_default();
    assert!(
        abilities_before.contains(&StaticAbility::Indestructible),
        "Ancient Guardian should have Indestructible before Turn to Frog"
    );

    // Cast Turn to Frog ({1}{U}) targeting the Guardian.
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Blue, 1).unwrap();
    let ttf = CardInstance::new("ttf-4", catalog::turn_to_frog(), &p1);
    game.add_card_to_hand(&p1, ttf).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("ttf-4"),
        targets: vec![Target::creature("guardian-4")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Turn to Frog on Ancient Guardian");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after Turn to Frog resolution");

    // Layer 7b: P/T = 1/1.
    assert_eq!(
        game.effective_power("guardian-4"),
        Some(1),
        "effective power should be 1 after Turn to Frog"
    );
    assert_eq!(
        game.effective_toughness("guardian-4"),
        Some(1),
        "effective toughness should be 1 after Turn to Frog"
    );

    // Layer 6: Indestructible has been removed by Turn to Frog.
    let abilities_after = game.effective_abilities("guardian-4").unwrap_or_default();
    assert!(
        !abilities_after.contains(&StaticAbility::Indestructible),
        "Ancient Guardian should NOT have Indestructible after Turn to Frog (Layer 6 removed it)"
    );

    // Now cast Lightning Strike ({1}{R}) targeting the Guardian.
    // With 1 toughness and no Indestructible, 3 damage should kill it.
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Red, 1).unwrap();
    let strike = CardInstance::new("strike-4", catalog::lightning_strike(), &p1);
    game.add_card_to_hand(&p1, strike).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("strike-4"),
        targets: vec![Target::creature("guardian-4")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Lightning Strike on the now-mortal Guardian");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after Lightning Strike resolution");

    // Guardian should be dead: in graveyard, not on battlefield.
    let guardian_on_bf = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "guardian-4");
    assert!(
        !guardian_on_bf,
        "Ancient Guardian should NOT be on battlefield after lethal damage (Indestructible removed)"
    );

    let guardian_in_gy = game
        .graveyard(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "guardian-4");
    assert!(
        guardian_in_gy,
        "Ancient Guardian should be in P1's graveyard after dying to Lightning Strike"
    );
}

// ============================================================================
// LS1.5 — Titanic Growth (Layer 7c pump)
// ============================================================================

/// LS1.5: Cast a Bear (2/2), then cast Titanic Growth (+4/+4).
///   - Verify: effective_power = 6, effective_toughness = 6 (Layer 7c: 2+4)
///   - Advance to Cleanup: Bear returns to 2/2.
#[test]
fn ls1_5_titanic_growth_layer7c_pump() {
    let (mut game, p1, _p2) = make_ls1_game();

    // Bear (2/2) on battlefield.
    let bear = CardInstance::new("bear-5", catalog::bear(), &p1);
    game.add_permanent_to_battlefield(&p1, bear).unwrap();

    // Verify base P/T.
    assert_eq!(game.effective_power("bear-5"), Some(2), "Bear starts at power 2");
    assert_eq!(game.effective_toughness("bear-5"), Some(2), "Bear starts at toughness 2");

    // Cast Titanic Growth ({1}{G}) on the Bear.
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    let tg = CardInstance::new("tg-5", catalog::titanic_growth(), &p1);
    game.add_card_to_hand(&p1, tg).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("tg-5"),
        targets: vec![Target::creature("bear-5")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Titanic Growth");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after Titanic Growth resolution");

    // Layer 7c: 2/2 + 4/4 = 6/6.
    assert_eq!(
        game.effective_power("bear-5"),
        Some(6),
        "effective power should be 6 after Titanic Growth (Layer 7c: 2 + 4)"
    );
    assert_eq!(
        game.effective_toughness("bear-5"),
        Some(6),
        "effective toughness should be 6 after Titanic Growth (Layer 7c: 2 + 4)"
    );

    // Advance to Cleanup: Titanic Growth expires, Bear returns to 2/2.
    advance_to_cleanup(&mut game, &p1);

    assert_eq!(
        game.effective_power("bear-5"),
        Some(2),
        "Bear should return to power 2 after Cleanup (Titanic Growth expired)"
    );
    assert_eq!(
        game.effective_toughness("bear-5"),
        Some(2),
        "Bear should return to toughness 2 after Cleanup (Titanic Growth expired)"
    );
}

// ============================================================================
// LS1.6 — Stacked effects: Titanic Growth + Twisted Image
// ============================================================================

/// LS1.6: Cast Ironbark Wall (0/4), cast Titanic Growth (+4/+4 → 4/8),
///        then cast Twisted Image (switch → 8/4).
///   - Verify: effective_power = 8, effective_toughness = 4.
///   - Verify: Wall survives (toughness = 4 > 0 → no SBA death).
#[test]
fn ls1_6_stacked_titanic_growth_plus_twisted_image() {
    let (mut game, p1, _p2) = make_ls1_game();

    // Ironbark Wall (0/4) on battlefield.
    let wall = CardInstance::new("wall-6", catalog::ironbark_wall(), &p1);
    game.add_permanent_to_battlefield(&p1, wall).unwrap();

    // Verify base P/T.
    assert_eq!(game.effective_power("wall-6"), Some(0), "Ironbark Wall starts at power 0");
    assert_eq!(game.effective_toughness("wall-6"), Some(4), "Ironbark Wall starts at toughness 4");

    // Step 1: Cast Titanic Growth ({1}{G}) on the Wall.
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    let tg = CardInstance::new("tg-6", catalog::titanic_growth(), &p1);
    game.add_card_to_hand(&p1, tg).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("tg-6"),
        targets: vec![Target::creature("wall-6")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Titanic Growth on the Wall");

    auto_resolve_stack(&mut game);

    // After Titanic Growth: Layer 7c gives 0+4=4 / 4+4=8.
    assert_eq!(
        game.effective_power("wall-6"),
        Some(4),
        "after Titanic Growth: effective power should be 4 (0 + 4)"
    );
    assert_eq!(
        game.effective_toughness("wall-6"),
        Some(8),
        "after Titanic Growth: effective toughness should be 8 (4 + 4)"
    );

    // Step 2: Add a library card so Twisted Image's draw doesn't fail.
    let lib_card = CardInstance::new("lib-6", catalog::forest(), &p1);
    game.add_card_to_library_top(&p1, lib_card).unwrap();

    // Cast Twisted Image ({U}) on the Wall.
    game.add_mana(&p1, ManaColor::Blue, 1).unwrap();
    let ti = CardInstance::new("ti-6", catalog::twisted_image(), &p1);
    game.add_card_to_hand(&p1, ti).unwrap();

    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("ti-6"),
        targets: vec![Target::creature("wall-6")],
        x_value: 0,
    })
    .expect("P1 should be able to cast Twisted Image on the Wall");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after Twisted Image resolution");

    // Layer evaluation order: 7c (Titanic Growth +4/+4) then 7d (switch).
    // Base: 0/4. After Layer 7c: 4/8. After Layer 7d switch: 8/4.
    assert_eq!(
        game.effective_power("wall-6"),
        Some(8),
        "effective power should be 8 after Titanic Growth + Twisted Image (switched)"
    );
    assert_eq!(
        game.effective_toughness("wall-6"),
        Some(4),
        "effective toughness should be 4 after Titanic Growth + Twisted Image (switched)"
    );

    // Wall should survive — effective toughness = 4 > 0.
    let wall_on_bf = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .any(|c| c.instance_id() == "wall-6");
    assert!(
        wall_on_bf,
        "Ironbark Wall should still be on battlefield (effective toughness = 4 > 0)"
    );
}
