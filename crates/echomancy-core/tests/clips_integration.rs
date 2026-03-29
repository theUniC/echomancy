//! End-to-end integration tests for the CLIPS rules engine pipeline.
//!
//! These tests prove the full pipeline works:
//!   Game setup → set_rules_engine → cast spell → auto_resolve_stack → Game mutation
//!
//! All tests use a REAL ClipsEngine (via `create_rules_engine`), not mocks.
//! Only the public API of `echomancy-core` is used.

use echomancy_core::prelude::*;
use echomancy_core::domain::game::automation::{auto_advance_to_main_phase, auto_resolve_stack};

// ============================================================================
// Setup helper
// ============================================================================

/// Create a 2-player game with real CLIPS engine, advanced to P1's FirstMain.
///
/// Returns `(game, p1_id, p2_id)`.
///
/// Cards in the engine: all card IDs present in both starter decks plus
/// the specific cards used in tests.
fn make_test_game() -> (Game, String, String) {
    let p1 = "p1".to_owned();
    let p2 = "p2".to_owned();

    let mut game = Game::create("integration-test");
    game.add_player(&p1, "Alice").unwrap();
    game.add_player(&p2, "Bob").unwrap();

    // Assign minimal libraries (enough not to deck out during setup).
    // We give each player a small library so opening hand draw doesn't fail.
    let p1_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p1-lib-{i}"), catalog::forest(), &p1))
        .collect();
    let p2_deck: Vec<CardInstance> = (0..10)
        .map(|i| CardInstance::new(format!("p2-lib-{i}"), catalog::mountain(), &p2))
        .collect();
    game.assign_deck(&p1, p1_deck).unwrap();
    game.assign_deck(&p2, p2_deck).unwrap();

    game.start(&p1, Some(42)).unwrap();

    // Set up the CLIPS engine with all card IDs relevant to these tests.
    let engine = create_rules_engine(&[
        "lightning-strike",
        "divination",
        "bear",
        "forest",
        "mountain",
        "giant-growth",
    ])
    .expect("CLIPS engine should initialise");
    game.set_rules_engine(engine);

    // Advance through non-interactive steps (Untap) to reach the first
    // interactive step. Per CR 117.3a, Upkeep is now interactive so
    // auto_advance_to_main_phase stops there. Advance manually to FirstMain.
    auto_advance_to_main_phase(&mut game, &p1);

    // Manually advance through Upkeep and Draw to reach FirstMain.
    // (Both are interactive but the integration tests need to start at FirstMain.)
    while game.current_step() != echomancy_core::prelude::Step::FirstMain {
        let current = game.current_player_id().to_owned();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&current),
        })
        .expect("should be able to advance to FirstMain");
    }

    assert_eq!(
        game.current_step(),
        echomancy_core::prelude::Step::FirstMain,
        "setup should land on FirstMain"
    );

    (game, p1, p2)
}

// ============================================================================
// Test 1: Lightning Strike kills a creature
// ============================================================================

#[test]
fn lightning_strike_kills_creature() {
    let (mut game, p1, p2) = make_test_game();

    // Give P1 enough mana to cast Lightning Strike ({1}{R}).
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Red, 1).unwrap();

    // Add Lightning Strike to P1's hand.
    let strike = CardInstance::new(
        "strike-1",
        catalog::lightning_strike(),
        &p1,
    );
    game.add_card_to_hand(&p1, strike).unwrap();

    // Put a Bear (2/2) on P2's battlefield.
    let bear = CardInstance::new("bear-1", catalog::bear(), &p2);
    game.add_permanent_to_battlefield(&p2, bear).unwrap();

    let p2_battlefield_before = game.battlefield(&p2).unwrap().len();
    assert_eq!(p2_battlefield_before, 1, "P2 should have a Bear on battlefield");

    // Cast Lightning Strike targeting the Bear.
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("strike-1"),
        targets: vec![Target::creature("bear-1")],
    })
    .expect("P1 should be able to cast Lightning Strike targeting a creature");

    assert_eq!(game.stack().len(), 1, "spell should be on stack");

    // Auto-resolve the stack (both players pass priority).
    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // Assert: Bear is no longer on P2's battlefield.
    let p2_battlefield_after = game.battlefield(&p2).unwrap().len();
    assert_eq!(
        p2_battlefield_after, 0,
        "Bear should have been destroyed by Lightning Strike"
    );

    // Assert: P2's graveyard has the Bear.
    let p2_graveyard = game.graveyard(&p2).unwrap();
    assert_eq!(p2_graveyard.len(), 1, "Bear should be in P2's graveyard");
    assert_eq!(
        p2_graveyard[0].instance_id(),
        "bear-1",
        "the destroyed permanent should be the Bear"
    );

    // Assert: Lightning Strike is in P1's graveyard.
    let p1_graveyard = game.graveyard(&p1).unwrap();
    assert!(
        p1_graveyard.iter().any(|c| c.instance_id() == "strike-1"),
        "Lightning Strike should be in P1's graveyard after resolution"
    );
}

// ============================================================================
// Test 2: Lightning Strike damages a player
// ============================================================================

#[test]
fn lightning_strike_damages_player() {
    let (mut game, p1, p2) = make_test_game();

    // Give P1 enough mana ({1}{R}).
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Red, 1).unwrap();

    // Add Lightning Strike to P1's hand.
    let strike = CardInstance::new("strike-2", catalog::lightning_strike(), &p1);
    game.add_card_to_hand(&p1, strike).unwrap();

    let p2_life_before = game.player_life_total(&p2).unwrap();

    // Cast Lightning Strike targeting P2.
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("strike-2"),
        targets: vec![Target::player(&p2)],
    })
    .expect("P1 should be able to cast Lightning Strike targeting a player");

    assert_eq!(game.stack().len(), 1, "spell should be on stack");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // Assert: P2's life total decreased by 3.
    let p2_life_after = game.player_life_total(&p2).unwrap();
    assert_eq!(
        p2_life_after,
        p2_life_before - 3,
        "Lightning Strike should deal 3 damage to P2 (was {p2_life_before}, expected {}, got {p2_life_after})",
        p2_life_before - 3
    );

    // P1 should be unaffected.
    assert_eq!(
        game.player_life_total(&p1).unwrap(),
        20,
        "P1 (caster) should be unaffected"
    );
}

// ============================================================================
// Test 3: Divination draws two cards
// ============================================================================

#[test]
fn divination_draws_two_cards() {
    let (mut game, p1, p2) = make_test_game();

    // Give P1 enough mana ({2}{U}).
    game.add_mana(&p1, ManaColor::Colorless, 2).unwrap();
    game.add_mana(&p1, ManaColor::Blue, 1).unwrap();

    // Add extra cards to P1's library so draws don't fail silently.
    let extra_lib_1 = CardInstance::new("extra-lib-1", catalog::forest(), &p1);
    let extra_lib_2 = CardInstance::new("extra-lib-2", catalog::forest(), &p1);
    game.add_card_to_library_top(&p1, extra_lib_1).unwrap();
    game.add_card_to_library_top(&p1, extra_lib_2).unwrap();

    // Add Divination to P1's hand.
    let divination = CardInstance::new("div-1", catalog::divination(), &p1);
    game.add_card_to_hand(&p1, divination).unwrap();

    let p1_hand_size_before = game.hand(&p1).unwrap().len(); // includes divination

    // Cast Divination (no target).
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("div-1"),
        targets: vec![],
    })
    .expect("P1 should be able to cast Divination");

    assert_eq!(game.stack().len(), 1, "spell should be on stack");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // Assert: P1's hand size increased by 2 (net: -1 for casting +2 for draws = +1).
    let p1_hand_size_after = game.hand(&p1).unwrap().len();
    assert_eq!(
        p1_hand_size_after,
        p1_hand_size_before - 1 + 2,
        "Divination should draw 2 cards for P1: before={p1_hand_size_before}, expected {}, got {p1_hand_size_after}",
        p1_hand_size_before - 1 + 2
    );

    // P2 should not have drawn any cards.
    let p2_hand_size_after = game.hand(&p2).unwrap().len();
    assert_eq!(
        game.hand(&p2).unwrap().len(),
        p2_hand_size_after,
        "P2 should not have drawn any cards"
    );
}

// ============================================================================
// Test 4: Vanilla creature resolves to battlefield
// ============================================================================

#[test]
fn vanilla_creature_resolves_to_battlefield() {
    let (mut game, p1, _p2) = make_test_game();

    // Give P1 mana to cast a Bear ({1}{G}).
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();

    // Add a Bear to P1's hand.
    let bear = CardInstance::new("bear-cast-1", catalog::bear(), &p1);
    game.add_card_to_hand(&p1, bear).unwrap();

    let p1_battlefield_before = game.battlefield(&p1).unwrap().len();

    // Cast Bear (creature, no target needed).
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("bear-cast-1"),
        targets: vec![],
    })
    .expect("P1 should be able to cast a Bear");

    assert_eq!(game.stack().len(), 1, "creature should be on stack");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // Assert: Bear is on P1's battlefield with a PermanentState.
    let p1_battlefield_after = game.battlefield(&p1).unwrap().len();
    assert_eq!(
        p1_battlefield_after,
        p1_battlefield_before + 1,
        "Bear should have entered the battlefield"
    );

    let bear_on_field = game
        .battlefield(&p1)
        .unwrap()
        .iter()
        .find(|c| c.instance_id() == "bear-cast-1");
    assert!(bear_on_field.is_some(), "Bear should be on P1's battlefield");

    // PermanentState should exist for the Bear.
    let perm_state = game.permanent_state("bear-cast-1");
    assert!(
        perm_state.is_some(),
        "Bear should have a PermanentState on the battlefield"
    );
}

// ============================================================================
// Test 5: Spell without CLIPS rule is a no-op (resolves, goes to graveyard)
// ============================================================================

#[test]
fn spell_without_clips_rule_is_noop() {
    let (mut game, p1, p2) = make_test_game();

    // Give P1 mana to cast Giant Growth ({G}).
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();

    // Add Giant Growth to P1's hand.
    // Giant Growth has no .clp rule — it's expected to resolve without any effect.
    let growth = CardInstance::new("growth-1", catalog::giant_growth(), &p1);
    game.add_card_to_hand(&p1, growth).unwrap();

    let p1_life_before = game.player_life_total(&p1).unwrap();
    let p2_life_before = game.player_life_total(&p2).unwrap();
    let p1_battlefield_before = game.battlefield(&p1).unwrap().len();
    let p2_battlefield_before = game.battlefield(&p2).unwrap().len();

    // Cast Giant Growth (no target — it has TargetRequirement::None in the catalog).
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("growth-1"),
        targets: vec![],
    })
    .expect("P1 should be able to cast Giant Growth");

    assert_eq!(game.stack().len(), 1, "spell should be on stack");

    auto_resolve_stack(&mut game);

    assert_eq!(game.stack().len(), 0, "stack should be empty after resolution");

    // Assert: Giant Growth is in P1's graveyard.
    let p1_graveyard = game.graveyard(&p1).unwrap();
    assert!(
        p1_graveyard.iter().any(|c| c.instance_id() == "growth-1"),
        "Giant Growth should be in P1's graveyard after resolution"
    );

    // Assert: no game state change (no damage, no permanents added).
    assert_eq!(
        game.player_life_total(&p1).unwrap(),
        p1_life_before,
        "P1 life total should be unchanged"
    );
    assert_eq!(
        game.player_life_total(&p2).unwrap(),
        p2_life_before,
        "P2 life total should be unchanged"
    );
    assert_eq!(
        game.battlefield(&p1).unwrap().len(),
        p1_battlefield_before,
        "P1 battlefield should be unchanged"
    );
    assert_eq!(
        game.battlefield(&p2).unwrap().len(),
        p2_battlefield_before,
        "P2 battlefield should be unchanged"
    );
}
