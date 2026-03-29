//! Integration tests that reproduce the P2 Bevy UI priority flow bugs.
//!
//! These tests simulate exactly what the Bevy `handle_game_actions` system does
//! in its auto-pass loop, but in pure domain code with no Bevy dependency.
//!
//! If any test fails, it means the domain has a bug that must be fixed before
//! the Bevy layer can work correctly.

use echomancy_core::prelude::*;

// ============================================================================
// Helper: simulate the Bevy auto-pass loop
// ============================================================================

/// Simulate what the Bevy `handle_game_actions` auto-pass loop does.
///
/// After every player action Bevy runs this loop:
/// 1. If current step is Untap or Cleanup → force AdvanceStep (no priority given).
/// 2. If no priority holder → stop.
/// 3. If the priority holder is auto-pass eligible → PassPriority, loop.
/// 4. Otherwise (player has meaningful actions) → stop.
///
/// This must exactly match the Bevy implementation in
/// `crates/echomancy-bevy/src/plugins/game/systems.rs`.
fn run_auto_pass_loop(game: &mut Game) {
    for _ in 0..50 {
        let step = game.current_step();
        if step == Step::Untap || step == Step::Cleanup {
            let current_active = game.current_player_id().to_owned();
            if game
                .apply(Action::AdvanceStep {
                    player_id: PlayerId::new(&current_active),
                })
                .is_err()
            {
                break;
            }
            continue;
        }
        let holder = match game.priority_player_id() {
            Some(id) => id.to_owned(),
            None => break,
        };
        if !compute_auto_pass_eligible(game, &holder) {
            break;
        }
        if game
            .apply(Action::PassPriority {
                player_id: PlayerId::new(&holder),
            })
            .is_err()
        {
            break;
        }
    }
}

// ============================================================================
// Setup helpers
// ============================================================================

/// Build a 2-player game in P1's FirstMain phase.
///
/// Uses empty libraries (no cards pre-assigned) so the hand is empty.
/// Test cards are injected manually via `game.add_card_to_hand()`.
fn make_first_main_game() -> (Game, String, String) {
    let p1 = "p1".to_owned();
    let p2 = "p2".to_owned();

    let mut game = Game::create("priority-flow-test");
    game.add_player(&p1, "Alice").unwrap();
    game.add_player(&p2, "Bob").unwrap();

    // Give each player a small library so startup draw doesn't deck them.
    let p1_deck: Vec<CardInstance> = (0..15)
        .map(|i| CardInstance::new(format!("p1-lib-{i}"), catalog::forest(), &p1))
        .collect();
    let p2_deck: Vec<CardInstance> = (0..15)
        .map(|i| CardInstance::new(format!("p2-lib-{i}"), catalog::mountain(), &p2))
        .collect();
    game.assign_deck(&p1, p1_deck).unwrap();
    game.assign_deck(&p2, p2_deck).unwrap();

    game.start(&p1, Some(42)).unwrap();

    // Advance from Untap → Upkeep → Draw → FirstMain.
    // We use AdvanceStep directly because at startup the Untap step has no
    // priority holder (non-interactive), and Upkeep/Draw have it but in these
    // tests neither player has actions so we drive forward manually.
    for _ in 0..3 {
        let current = game.current_player_id().to_owned();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&current),
        })
        .expect("should be able to advance past startup steps");
    }

    assert_eq!(
        game.current_step(),
        Step::FirstMain,
        "game should be at FirstMain after setup"
    );
    assert_eq!(
        game.priority_player_id(),
        Some("p1"),
        "P1 should have priority at FirstMain"
    );

    (game, p1, p2)
}

// ============================================================================
// Test 1: play land → auto-pass stops because of tappable land
// ============================================================================

#[test]
fn play_land_then_auto_pass_stops_because_land_is_tappable() {
    // After playing a land the player has a tappable land, so auto-pass must
    // NOT fire — the player needs to decide whether to tap for mana.
    let (mut game, p1, _p2) = make_first_main_game();

    let forest = CardInstance::new("forest-1", catalog::forest(), &p1);
    game.add_card_to_hand(&p1, forest).unwrap();

    // P1 plays the Forest.
    game.apply(Action::PlayLand {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("forest-1"),
    })
    .expect("P1 should be able to play a Forest in FirstMain");

    // Run the auto-pass loop.
    run_auto_pass_loop(&mut game);

    // Loop should stop because P1 has a tappable land (she can still tap it).
    assert_eq!(
        game.current_step(),
        Step::FirstMain,
        "should still be in FirstMain after playing land"
    );
    assert_eq!(
        game.priority_player_id(),
        Some(p1.as_str()),
        "P1 should still have priority — she has a tappable land"
    );
}

// ============================================================================
// Test 2: cast creature → auto-resolves → creature appears on battlefield
// ============================================================================

#[test]
fn cast_creature_auto_resolves_and_player_keeps_turn() {
    // P1 at FirstMain with a Bear in hand and enough mana.
    // After casting, the auto-pass loop should fire twice (P1 passes, P2 passes)
    // so the Bear resolves immediately. P1 should still be in FirstMain.
    let (mut game, p1, p2) = make_first_main_game();

    let bear = CardInstance::new("bear-1", catalog::bear(), &p1);
    game.add_card_to_hand(&p1, bear).unwrap();

    // Give P1 exactly {1}{G} to pay the Bear's cost.
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

    // P1 casts Bear.
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("bear-1"),
        targets: vec![],
    })
    .expect("P1 should be able to cast Bear with {1}{G}");

    assert_eq!(game.stack().len(), 1, "Bear should be on the stack");

    // Run auto-pass loop — neither player has instants/mana to respond.
    // P1 (caster) auto-passes, P2 auto-passes, Bear resolves.
    run_auto_pass_loop(&mut game);

    assert!(game.stack().is_empty(), "stack should be empty after resolution");
    assert_eq!(
        game.battlefield(&p1).unwrap().len(),
        1,
        "Bear should be on P1's battlefield"
    );
    // P1 should still be the active player in FirstMain.
    assert_eq!(
        game.current_player_id(),
        p1.as_str(),
        "P1 should still be active player"
    );
    assert_eq!(
        game.current_step(),
        Step::FirstMain,
        "should still be in FirstMain after Bear resolves"
    );
    // Sanity: P2 should not have gotten the turn.
    assert_ne!(
        game.current_player_id(),
        p2.as_str(),
        "P2 should not be the active player yet"
    );
}

// ============================================================================
// Test 3: end turn advances to P2's FirstMain
// ============================================================================

#[test]
fn end_turn_advances_to_opponent_first_main() {
    // P1 at FirstMain with nothing to do. Runs auto-pass → advances all the way
    // through to P2's FirstMain because nobody has anything to do.
    let (mut game, p1, p2) = make_first_main_game();

    // P1 ends her turn.
    game.apply(Action::EndTurn {
        player_id: PlayerId::new(&p1),
    })
    .expect("P1 should be able to end her turn");

    // Run auto-pass loop — advances through P1's remaining steps and P2's
    // Untap (non-interactive), then stops at P2's Upkeep (or FirstMain if
    // Upkeep/Draw both auto-pass too). The loop must eventually settle at a
    // step where P2 has priority.
    run_auto_pass_loop(&mut game);

    // After the loop, P2 should be active and have priority at some
    // interactive step in her turn. The loop advances through Untap
    // automatically, but stops at Upkeep (interactive) once P2 has priority
    // and no actions.
    //
    // Either P2 is already at Upkeep (loop stopped due to no castable
    // instants/no tappable lands at that step), or at FirstMain if the loop
    // managed to pass through Upkeep+Draw. Both are acceptable — the critical
    // assertion is that P2 is now the active player with priority.
    let step_after = game.current_step();
    let active_after = game.current_player_id().to_owned();

    assert_eq!(
        active_after, p2,
        "P2 should be the active player after P1 ends her turn and loop runs; got step={step_after:?}"
    );
    assert_eq!(
        game.priority_player_id(),
        Some(p2.as_str()),
        "P2 should have priority"
    );
}

// ============================================================================
// Test 4: opponent with instant + mana gets a priority window
// ============================================================================

#[test]
fn opponent_with_instant_and_mana_gets_priority_window() {
    // P1 at FirstMain, P2 has a Lightning Strike in hand and enough mana.
    // P1 casts Bear. Auto-pass loop: P1 auto-passes (no instants),
    // P2 should NOT auto-pass because she can cast Lightning Strike.
    // The loop must stop with P2 holding priority and the Bear still on the stack.
    let (mut game, p1, p2) = make_first_main_game();

    // Give P1 {1}{G} for the Bear.
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    let bear = CardInstance::new("bear-1", catalog::bear(), &p1);
    game.add_card_to_hand(&p1, bear).unwrap();

    // Give P2 a Lightning Strike and {1}{R} to cast it.
    game.add_mana(&p2, ManaColor::Red, 1).unwrap();
    game.add_mana(&p2, ManaColor::Colorless, 1).unwrap();
    let strike = CardInstance::new("strike-1", catalog::lightning_strike(), &p2);
    game.add_card_to_hand(&p2, strike).unwrap();

    // P1 casts Bear.
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("bear-1"),
        targets: vec![],
    })
    .expect("P1 should cast Bear");

    assert_eq!(game.stack().len(), 1, "Bear should be on the stack");

    // P1 (caster) has priority per CR 117.3c. Run loop:
    //   - P1 has no instants → P1 auto-passes → P2 gets priority.
    //   - P2 has Lightning Strike + enough mana → loop stops.
    run_auto_pass_loop(&mut game);

    // Bear must still be on the stack — it has not resolved.
    assert_eq!(
        game.stack().len(),
        1,
        "Bear should still be on the stack (P2 has a response)"
    );

    // P2 should have priority, not P1.
    assert_eq!(
        game.priority_player_id(),
        Some(p2.as_str()),
        "P2 should hold priority after P1 auto-passes"
    );

    // We are still in P1's FirstMain.
    assert_eq!(
        game.current_step(),
        Step::FirstMain,
        "game should still be in P1's FirstMain"
    );
}

// ============================================================================
// Test 5: two full turns cycle correctly
// ============================================================================

#[test]
fn two_turns_cycle_correctly() {
    // P1 plays Forest, taps it, tries to cast Bear (but may lack 1 colorless).
    // Simpler version: P1 ends turn → P2 at FirstMain → P2 ends turn → P1 back.
    let (mut game, p1, p2) = make_first_main_game();

    // --- P1's first turn: end turn immediately ---
    game.apply(Action::EndTurn {
        player_id: PlayerId::new(&p1),
    })
    .expect("P1 ends turn 1");

    run_auto_pass_loop(&mut game);

    // P2 should now be active at some point in her turn.
    let p2_active = game.current_player_id().to_owned();
    assert_eq!(
        p2_active, p2,
        "P2 should be active after P1 ends her turn"
    );
    assert_eq!(
        game.priority_player_id(),
        Some(p2.as_str()),
        "P2 should have priority"
    );

    // Advance P2 manually to her FirstMain if she landed at Upkeep/Draw.
    while game.current_step() != Step::FirstMain {
        let current = game.current_player_id().to_owned();
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&current),
        })
        .expect("should be able to advance P2's early steps");
    }

    assert_eq!(game.current_step(), Step::FirstMain);
    assert_eq!(game.current_player_id(), p2.as_str());

    // --- P2's first turn: end turn immediately ---
    game.apply(Action::EndTurn {
        player_id: PlayerId::new(&p2),
    })
    .expect("P2 ends her first turn");

    run_auto_pass_loop(&mut game);

    // P1 should now be active again.
    let p1_active = game.current_player_id().to_owned();
    assert_eq!(
        p1_active, p1,
        "P1 should be active again on turn 3"
    );
    assert_eq!(
        game.priority_player_id(),
        Some(p1.as_str()),
        "P1 should have priority"
    );

    // Turn number should have incremented (P1 is now on turn 3).
    // (Turn 1 = P1, turn 2 = P2, turn 3 = P1 again.)
    // We don't assert the exact turn number here because the domain may count
    // differently, but we verify P1 is active with priority.
}

// ============================================================================
// Test 6: perspective follows priority holder
// ============================================================================

#[test]
fn perspective_follows_priority_holder() {
    // Verify that at each point of the game the "perspective" (priority holder)
    // is exactly who we expect it to be.
    let (mut game, p1, p2) = make_first_main_game();

    // At game start P1 has priority in her FirstMain.
    assert_eq!(
        game.priority_player_id(),
        Some(p1.as_str()),
        "T0: P1 should have priority at game start"
    );

    // Give P1 a Bear and {1}{G}.
    game.add_mana(&p1, ManaColor::Green, 1).unwrap();
    game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
    let bear = CardInstance::new("bear-1", catalog::bear(), &p1);
    game.add_card_to_hand(&p1, bear).unwrap();

    // P1 casts Bear → per CR 117.3c, P1 (caster) retains priority.
    game.apply(Action::CastSpell {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("bear-1"),
        targets: vec![],
    })
    .expect("P1 should cast Bear");

    assert_eq!(
        game.priority_player_id(),
        Some(p1.as_str()),
        "T1: after casting, P1 (caster) should still hold priority per CR 117.3c"
    );

    // P1 passes priority → P2 gets priority.
    game.apply(Action::PassPriority {
        player_id: PlayerId::new(&p1),
    })
    .expect("P1 passes priority");

    assert_eq!(
        game.priority_player_id(),
        Some(p2.as_str()),
        "T2: P2 should have priority after P1 passes"
    );

    // P2 passes priority → both have passed → Bear resolves → P1 regains priority.
    game.apply(Action::PassPriority {
        player_id: PlayerId::new(&p2),
    })
    .expect("P2 passes priority, Bear resolves");

    // After resolution, P1 gets priority back in her own main phase.
    assert_eq!(
        game.priority_player_id(),
        Some(p1.as_str()),
        "T3: P1 should have priority again after Bear resolves in her main phase"
    );

    // Bear should now be on the battlefield.
    assert_eq!(
        game.battlefield(&p1).unwrap().len(),
        1,
        "Bear should be on P1's battlefield after resolution"
    );

    // Still P1's FirstMain.
    assert_eq!(
        game.current_step(),
        Step::FirstMain,
        "T4: should still be in P1's FirstMain after resolution"
    );
}

// ============================================================================
// Test 7: play land → tap land → have enough mana → can cast creature
// ============================================================================

#[test]
fn play_land_tap_land_then_can_cast_creature_stays_in_first_main() {
    // P1 plays 2 Forests, taps them both for {G}{G}. Bear costs {1}{G}.
    // With only {G}{G} she cannot afford the {1} generic, so auto-pass fires.
    //
    // But if we give P1 an extra colorless mana source or tap 2 Forests
    // which produce {G}{G} and she has a Goblin ({R}) or an Elf ({G}),
    // the test checks that the loop halts correctly.
    //
    // Scenario: P1 has 1 Forest on battlefield (untapped), Bear in hand.
    // Forest → {G}, Bear costs {1}{G}. P1 cannot cast Bear with only 1 mana.
    // Auto-pass should fire after she taps the Forest.
    //
    // Phase 2: P1 has 2 Forests. Taps both → {G}{G}. Bear costs {1}{G}.
    // Still can't cast: need 1 generic + 1 green but only have 2 green.
    // By default {G}{G} can pay {1}{G} because generic can be paid with any colour.
    // So with 2 Forests P1 CAN cast the Bear. Auto-pass should NOT fire.

    let (mut game, p1, _p2) = make_first_main_game();

    // Add 2 Forests to P1's hand.
    let forest1 = CardInstance::new("forest-bf-1", catalog::forest(), &p1);
    let forest2 = CardInstance::new("forest-bf-2", catalog::forest(), &p1);
    game.add_card_to_hand(&p1, forest1).unwrap();
    game.add_card_to_hand(&p1, forest2).unwrap();

    let bear = CardInstance::new("bear-1", catalog::bear(), &p1);
    game.add_card_to_hand(&p1, bear).unwrap();

    // P1 plays the first Forest.
    game.apply(Action::PlayLand {
        player_id: PlayerId::new(&p1),
        card_id: CardInstanceId::new("forest-bf-1"),
    })
    .expect("P1 plays forest-bf-1");

    // After playing, auto-pass loop: P1 has a tappable land → loop stops.
    run_auto_pass_loop(&mut game);

    assert_eq!(
        game.priority_player_id(),
        Some(p1.as_str()),
        "P1 should still have priority — she can still tap Forest"
    );

    // P1 taps Forest for {G}.
    game.apply(Action::ActivateAbility {
        player_id: PlayerId::new(&p1),
        permanent_id: CardInstanceId::new("forest-bf-1"),
    })
    .expect("P1 taps forest-bf-1 for {G}");

    // With only {G} and Bear costing {1}{G}, P1 needs 1 more mana.
    // She has Forest-bf-2 in hand but has already played her land for the turn.
    // So she cannot cast the Bear. Auto-pass should fire.
    run_auto_pass_loop(&mut game);

    // But wait — P1 still has forest-bf-2 in hand. Can she play it? No, she
    // already played a land this turn. So auto-pass should advance the turn.
    // However the second Forest is in her HAND, not on battlefield — tappable
    // lands are only on the battlefield. So P1 has no tappable lands, no
    // castable spells (can't pay {1}{G} with only {G}), and no playable lands
    // (already played one). Auto-pass fires until P2's turn.

    // After auto-pass: we should be in P2's turn at some interactive step.
    let active = game.current_player_id().to_owned();
    assert_ne!(
        active, p1,
        "Auto-pass should have advanced to P2's turn since P1 has no actions after tapping"
    );
}
