//! Game automation helpers — pure functions that advance game state through
//! non-interactive steps without requiring player input.
//!
//! These functions encapsulate "auto-pilot" behaviour that the presentation
//! layer (Bevy) previously had to implement itself. Moving them here ensures
//! the domain model owns all the rules about which steps require player
//! interaction and which can be skipped automatically.

use crate::domain::actions::Action;
use crate::domain::enums::Step;
use crate::domain::types::PlayerId;

use super::Game;

/// Returns `true` for steps that have no player interaction and should be
/// automatically skipped.
///
/// Per CR 117.3a, players receive priority at the beginning of most steps.
/// Only Untap and Cleanup are non-interactive (no priority is given).
///
/// All other steps — including Upkeep, Draw, BeginningOfCombat, EndOfCombat,
/// and EndStep — are interactive. The auto-pass heuristic handles the common
/// case where neither player can act, keeping the experience smooth.
pub fn is_non_interactive_step(step: Step) -> bool {
    matches!(step, Step::Untap | Step::Cleanup)
}

/// Maximum iterations for auto-advance loops.
/// A turn has 12 steps, so 20 is generous while still preventing infinite loops.
const MAX_AUTO_ITERATIONS: usize = 20;

/// Advance through all non-interactive steps until an interactive step or a
/// turn change occurs.
///
/// Called after any action is applied and after turn changes so the player
/// always lands on a step where they can act (or see the result).
pub fn auto_advance_through_non_interactive(game: &mut Game, player_id: &str) {
    let mut iterations = 0;
    while is_non_interactive_step(game.current_step()) && iterations < MAX_AUTO_ITERATIONS {
        if game
            .apply(Action::AdvanceStep {
                player_id: PlayerId::new(player_id),
            })
            .is_err()
        {
            break;
        }
        iterations += 1;
    }
}

/// Advance through non-interactive steps (Untap, Upkeep, Draw) to reach
/// FirstMain where the player can actually take actions.
///
/// This is called both at startup (for P1) and whenever the active player
/// changes (for P2, P1 again, etc.). Without this, the player would need
/// to manually click "Pass Priority" through steps where nothing happens.
pub fn auto_advance_to_main_phase(game: &mut Game, player_id: &str) {
    auto_advance_through_non_interactive(game, player_id);
}

/// Auto-pass priority for both players until the stack empties.
///
/// In the MVP, no player has counterspells or instant-speed responses, so
/// when a spell is cast, we immediately resolve it by passing priority from
/// whoever has it until the stack is empty. This avoids requiring the user
/// to manually switch perspectives and click "Pass Priority" multiple times.
///
/// Max iterations guard prevents infinite loops.
pub fn auto_resolve_stack(game: &mut Game) {
    let mut iterations = 0;
    while game.stack_has_items() && iterations < MAX_AUTO_ITERATIONS {
        if let Some(priority_holder) = game.priority_player_id().map(str::to_owned) {
            if game
                .apply(Action::PassPriority {
                    player_id: PlayerId::new(&priority_holder),
                })
                .is_err()
            {
                break;
            }
        } else {
            break;
        }
        iterations += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::prebuilt_decks;

    fn make_started_game() -> (Game, String, String) {
        let p1 = uuid::Uuid::new_v4().to_string();
        let p2 = uuid::Uuid::new_v4().to_string();
        let mut game = Game::create(uuid::Uuid::new_v4().to_string());
        game.add_player(&p1, "Alice").unwrap();
        game.add_player(&p2, "Bob").unwrap();
        game.assign_deck(&p1, prebuilt_decks::green_deck(&p1))
            .unwrap();
        game.assign_deck(&p2, prebuilt_decks::red_deck(&p2))
            .unwrap();
        game.start(&p1, Some(42)).unwrap();
        (game, p1, p2)
    }

    // ---- is_non_interactive_step -------------------------------------------

    #[test]
    fn untap_is_non_interactive() {
        assert!(is_non_interactive_step(Step::Untap));
    }

    // CR 117.3a: only Untap and Cleanup are non-interactive
    #[test]
    fn upkeep_is_interactive_per_cr_117_3a() {
        assert!(!is_non_interactive_step(Step::Upkeep));
    }

    #[test]
    fn draw_is_interactive_per_cr_117_3a() {
        assert!(!is_non_interactive_step(Step::Draw));
    }

    #[test]
    fn beginning_of_combat_is_interactive_per_cr_117_3a() {
        assert!(!is_non_interactive_step(Step::BeginningOfCombat));
    }

    #[test]
    fn end_of_combat_is_interactive_per_cr_117_3a() {
        assert!(!is_non_interactive_step(Step::EndOfCombat));
    }

    #[test]
    fn end_step_is_interactive_per_cr_117_3a() {
        assert!(!is_non_interactive_step(Step::EndStep));
    }

    #[test]
    fn cleanup_is_non_interactive() {
        assert!(is_non_interactive_step(Step::Cleanup));
    }

    #[test]
    fn first_main_is_interactive() {
        assert!(!is_non_interactive_step(Step::FirstMain));
    }

    #[test]
    fn second_main_is_interactive() {
        assert!(!is_non_interactive_step(Step::SecondMain));
    }

    #[test]
    fn declare_attackers_is_interactive() {
        assert!(!is_non_interactive_step(Step::DeclareAttackers));
    }

    #[test]
    fn declare_blockers_is_interactive() {
        assert!(!is_non_interactive_step(Step::DeclareBlockers));
    }

    #[test]
    fn combat_damage_is_interactive() {
        assert!(!is_non_interactive_step(Step::CombatDamage));
    }

    #[test]
    fn first_strike_damage_is_interactive() {
        assert!(!is_non_interactive_step(Step::FirstStrikeDamage));
    }

    // ---- auto_advance_through_non_interactive ------------------------------

    #[test]
    fn auto_advance_stops_at_upkeep() {
        // CR 117.3a: Upkeep is interactive — auto-advance must stop there.
        let (mut game, p1, _) = make_started_game();
        // Game starts at Untap (non-interactive).
        assert_eq!(game.current_step(), Step::Untap);

        auto_advance_through_non_interactive(&mut game, &p1);

        // Should land on Upkeep (the first interactive step after Untap).
        assert_eq!(game.current_step(), Step::Upkeep);
    }

    #[test]
    fn auto_advance_does_not_skip_interactive_step() {
        let (mut game, p1, _) = make_started_game();
        // Manually advance to FirstMain.
        for _ in 0..3 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&p1),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::FirstMain);

        // auto_advance_through_non_interactive should not move past FirstMain.
        auto_advance_through_non_interactive(&mut game, &p1);
        assert_eq!(game.current_step(), Step::FirstMain);
    }

    // ---- auto_advance_to_main_phase ----------------------------------------

    #[test]
    fn auto_advance_to_main_phase_lands_on_upkeep() {
        // After Fix 2 (CR 117.3a), Upkeep is interactive so auto-advance stops there.
        // The function name is kept for API compatibility but the behavior is correct.
        let (mut game, p1, _) = make_started_game();
        assert_eq!(game.current_step(), Step::Untap);

        auto_advance_to_main_phase(&mut game, &p1);

        assert_eq!(game.current_step(), Step::Upkeep);
    }

    // ---- priority assigned at all interactive steps (CR 117.3a) -----------

    #[test]
    fn priority_is_assigned_at_upkeep_when_entering_that_step() {
        // CR 117.3a: active player receives priority at beginning of Upkeep.
        let (mut game, p1, _) = make_started_game();
        // Initially in Untap — no priority (non-interactive).
        assert!(game.priority_player_id().is_none() || game.current_step() == Step::Untap);

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // Untap → Upkeep

        assert_eq!(game.current_step(), Step::Upkeep);
        assert_eq!(
            game.priority_player_id(),
            Some(p1.as_str()),
            "active player should have priority at Upkeep per CR 117.3a"
        );
    }

    #[test]
    fn priority_is_assigned_at_end_step_when_entering_that_step() {
        // CR 117.3a: active player receives priority at EndStep.
        let (mut game, p1, _) = make_started_game();
        // Advance to EndStep (step index 11: Untap→Upk→Draw→FM→BoC→DA→DB→FSD→CD→EoC→SM→ES)
        let steps_to_end_step = 11;
        for _ in 0..steps_to_end_step {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        assert_eq!(game.current_step(), Step::EndStep);
        assert_eq!(
            game.priority_player_id(),
            Some(p1.as_str()),
            "active player should have priority at EndStep per CR 117.3a"
        );
    }

    // ---- auto_resolve_stack ------------------------------------------------

    #[test]
    fn auto_resolve_stack_is_no_op_when_stack_empty() {
        let (mut game, p1, _) = make_started_game();
        // Advance to Upkeep (first interactive step after Untap).
        auto_advance_to_main_phase(&mut game, &p1);
        assert!(!game.stack_has_items());
        assert_eq!(game.current_step(), Step::Upkeep);

        // Should not panic or change anything.
        auto_resolve_stack(&mut game);
        assert!(!game.stack_has_items());
        assert_eq!(game.current_step(), Step::Upkeep);
    }
}
