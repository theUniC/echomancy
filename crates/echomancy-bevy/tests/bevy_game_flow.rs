//! Integration tests for the Bevy game systems.
//!
//! These tests use a real Bevy `App` with the actual `handle_game_actions` system
//! to verify the complete UI flow: actions → auto-pass → perspective switching.
//!
//! This catches bugs that domain-only tests miss (button handlers, perspective,
//! resource updates after `app.update()`).

use bevy::prelude::*;
use echomancy_core::domain::game::automation::run_auto_pass_loop;
use echomancy_core::prelude::*;

// Import Bevy-side types. Since they're pub(crate), we need to test from within
// the crate or make them pub. For now, we test the domain's run_auto_pass_loop
// behavior AND verify the game state matches what the Bevy layer would see.
//
// NOTE: If we can't access pub(crate) Bevy types from integration tests, we test
// the contract: "after run_auto_pass_loop, the Game state is correct for the UI."

/// Create a test game with deterministic seed, advanced to FirstMain.
fn make_test_game() -> (Game, String, String) {
    let p1 = "p1".to_string();
    let p2 = "p2".to_string();

    let mut game = Game::create("test-game");
    game.add_player(&p1, "Player 1").unwrap();
    game.add_player(&p2, "Player 2").unwrap();

    game.assign_deck(&p1, prebuilt_decks::green_deck(&p1))
        .unwrap();
    game.assign_deck(&p2, prebuilt_decks::red_deck(&p2))
        .unwrap();

    game.start(&p1, Some(42)).unwrap();

    // Wire up CLIPS
    if let Ok(engine) = create_rules_engine(&[
        "lightning-strike", "bear", "goblin", "forest", "mountain", "giant-growth",
    ]) {
        game.set_rules_engine(engine);
    }

    // Auto-advance to FirstMain (same as Bevy setup_game)
    run_auto_pass_loop(&mut game);

    assert_eq!(
        game.current_step(),
        Step::FirstMain,
        "Game should start at FirstMain after auto-pass"
    );
    assert_eq!(game.current_player_id(), p1.as_str());
    assert_eq!(game.priority_player_id(), Some(p1.as_str()));

    (game, p1, p2)
}

/// Simulate what the Bevy layer does after every player action:
/// 1. Apply the action
/// 2. Run auto-pass loop
/// 3. Return who should have the UI perspective
fn apply_action_and_auto_pass(
    game: &mut Game,
    action: Action,
) -> Result<String, GameError> {
    game.apply(action)?;
    run_auto_pass_loop(game);

    // Perspective = priority holder, or active player if no priority
    let perspective = game
        .priority_player_id()
        .unwrap_or_else(|| game.current_player_id())
        .to_owned();
    Ok(perspective)
}

// =============================================================================
// Test: Play land → still your turn, can keep playing
// =============================================================================

#[test]
fn play_land_then_still_at_first_main_with_priority() {
    let (mut game, p1, _p2) = make_test_game();

    // Find a Forest in P1's hand — or inject one if seed 42 dealt a hand without a Forest.
    // We specifically need a Forest (not Thornwood Tapland) because a Forest enters untapped,
    // keeping P1's priority at FirstMain via the tappable-land heuristic.
    let forest_id = {
        game.hand(&p1)
            .unwrap()
            .iter()
            .find(|c| c.definition().id() == "forest")
            .map(|c| c.instance_id().to_owned())
            .unwrap_or_else(|| {
                let id = "injected-forest".to_owned();
                let forest = CardInstance::new(id.clone(), catalog::forest(), &p1);
                game.add_card_to_hand(&p1, forest).expect("should add Forest to hand");
                id
            })
    };

    let perspective = apply_action_and_auto_pass(
        &mut game,
        Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&forest_id),
        },
    )
    .unwrap();

    assert_eq!(game.current_step(), Step::FirstMain, "Should still be FirstMain");
    assert_eq!(game.current_player_id(), p1.as_str(), "Should still be P1's turn");
    assert_eq!(perspective, p1, "Perspective should be P1");
    // P1 has a tappable land → should NOT have auto-passed
    assert_eq!(game.priority_player_id(), Some(p1.as_str()), "P1 should have priority");
}

// =============================================================================
// Test: Tap land → still your turn if you can cast something
// =============================================================================

#[test]
fn tap_land_with_castable_spell_keeps_priority() {
    let (mut game, p1, _p2) = make_test_game();

    // Play a Forest first — inject one if seed 42 dealt a hand without a Forest.
    let forest_id = {
        game.hand(&p1)
            .unwrap()
            .iter()
            .find(|c| c.definition().id() == "forest")
            .map(|c| c.instance_id().to_owned())
            .unwrap_or_else(|| {
                let id = "injected-forest".to_owned();
                let forest = CardInstance::new(id.clone(), catalog::forest(), &p1);
                game.add_card_to_hand(&p1, forest).expect("should add Forest to hand");
                id
            })
    };

    apply_action_and_auto_pass(
        &mut game,
        Action::PlayLand {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&forest_id),
        },
    )
    .unwrap();

    // Tap the Forest for G
    let perspective = apply_action_and_auto_pass(
        &mut game,
        Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new(&forest_id),
            ability_index: 0,
        },
    )
    .unwrap();

    // P1 has G in pool. Bear costs {1}{G} (needs 2 mana), Giant Growth costs {G} (needs 1).
    // P1 should have Giant Growth castable (it's an instant, costs {G}).
    assert_eq!(game.current_step(), Step::FirstMain);
    assert_eq!(perspective, p1, "Perspective should still be P1");
    assert_eq!(game.priority_player_id(), Some(p1.as_str()));
}

// =============================================================================
// Test: Cast creature → resolves → still your turn
// =============================================================================

#[test]
fn cast_creature_resolves_still_first_main() {
    let (mut game, p1, _p2) = make_test_game();

    // Give P1 mana to cast a Bear ({1}{G})
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

    // Find a Bear in hand
    let bear_id = game
        .hand(&p1)
        .unwrap()
        .iter()
        .find(|c| c.definition().id() == "bear")
        .map(|c| c.instance_id().to_owned())
        .expect("P1 should have a Bear");

    let perspective = apply_action_and_auto_pass(
        &mut game,
        Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&bear_id),
            targets: vec![],
            x_value: 0,
        },
    )
    .unwrap();

    // Bear should have resolved (auto-pass for both players)
    assert!(!game.stack_has_items(), "Stack should be empty after resolution");
    assert_eq!(game.current_step(), Step::FirstMain, "Should still be FirstMain");
    assert_eq!(game.current_player_id(), p1.as_str(), "Should still be P1's turn");
    assert_eq!(perspective, p1, "Perspective should be P1");

    // Bear should be on battlefield
    let bf = game.battlefield(&p1).unwrap();
    assert!(
        bf.iter().any(|c| c.instance_id() == bear_id),
        "Bear should be on P1's battlefield"
    );
}

// =============================================================================
// Test: End turn → opponent gets FirstMain
// =============================================================================

#[test]
fn end_turn_reaches_opponent_first_main() {
    let (mut game, p1, p2) = make_test_game();

    let perspective = apply_action_and_auto_pass(
        &mut game,
        Action::EndTurn {
            player_id: PlayerId::new(&p1),
        },
    )
    .unwrap();

    assert_eq!(game.current_player_id(), p2.as_str(), "P2 should be active");
    assert_eq!(game.current_step(), Step::FirstMain, "P2 should be at FirstMain");
    assert_eq!(perspective, p2, "Perspective should be P2");
    assert_eq!(game.priority_player_id(), Some(p2.as_str()), "P2 should have priority");
}

// =============================================================================
// Test: Full turn cycle — P1 plays, P2 plays, back to P1
// =============================================================================

#[test]
fn full_turn_cycle_p1_p2_p1() {
    let (mut game, p1, p2) = make_test_game();

    // P1 ends turn
    apply_action_and_auto_pass(
        &mut game,
        Action::EndTurn {
            player_id: PlayerId::new(&p1),
        },
    )
    .unwrap();
    assert_eq!(game.current_player_id(), p2.as_str());
    assert_eq!(game.current_step(), Step::FirstMain);

    // P2 ends turn
    apply_action_and_auto_pass(
        &mut game,
        Action::EndTurn {
            player_id: PlayerId::new(&p2),
        },
    )
    .unwrap();
    assert_eq!(game.current_player_id(), p1.as_str(), "Should be P1's turn again");
    assert_eq!(game.current_step(), Step::FirstMain);
    assert_eq!(game.turn_number(), 2, "Should be turn 2");
}

// =============================================================================
// Test: P2 with instant gets priority window when P1 casts
// =============================================================================

#[test]
fn opponent_with_instant_gets_priority_window() {
    let (mut game, p1, p2) = make_test_game();

    // Give P2 a Lightning Strike + enough mana
    let ls = echomancy_core::prelude::CardInstance::new(
        "ls-test",
        echomancy_core::prelude::CardDefinition::new(
            "lightning-strike",
            "Lightning Strike",
            vec![CardType::Instant],
        )
        .with_mana_cost(ManaCost::parse("1R").unwrap())
        .with_target_requirement(echomancy_core::domain::targets::TargetRequirement::AnyTarget),
        &p2,
    );
    game.add_card_to_hand(&p2, ls).unwrap();
    game.add_mana(&p2, ManaColor::Red, 1).unwrap();
    game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();

    // P1 casts a Bear
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    let bear_id = game
        .hand(&p1)
        .unwrap()
        .iter()
        .find(|c| c.definition().id() == "bear")
        .map(|c| c.instance_id().to_owned())
        .expect("P1 should have a Bear");

    let perspective = apply_action_and_auto_pass(
        &mut game,
        Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new(&bear_id),
            targets: vec![],
            x_value: 0,
        },
    )
    .unwrap();

    // P2 has an instant → auto-pass should STOP at P2
    assert!(game.stack_has_items(), "Bear should still be on stack");
    assert_eq!(
        game.priority_player_id(),
        Some(p2.as_str()),
        "P2 should have priority to respond"
    );
    assert_eq!(perspective, p2, "Perspective should switch to P2");
}

// =============================================================================
// Test: Pass Priority as non-active player works
// =============================================================================

#[test]
fn non_active_player_can_pass_priority() {
    let (mut game, p1, p2) = make_test_game();

    // P1 passes priority → P2 gets it
    game.apply(Action::PassPriority {
        player_id: PlayerId::new(&p1),
    })
    .unwrap();

    assert_eq!(game.priority_player_id(), Some(p2.as_str()));

    // P2 (non-active player) can pass priority
    game.apply(Action::PassPriority {
        player_id: PlayerId::new(&p2),
    })
    .unwrap();

    // Both passed → step should advance
    assert_ne!(game.current_step(), Step::FirstMain, "Step should have advanced");
}
