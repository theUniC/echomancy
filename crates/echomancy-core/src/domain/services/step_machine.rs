//! Step Machine — pure turn-step sequencing.
//!
//! Advances from one `Step` to the next in the standard MTG turn order.
//! Mirrors `StepMachine.ts`.

use crate::domain::enums::Step;

/// Describes the result of advancing the step machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StepResult {
    /// The step to move to.
    pub next_step: Step,
    /// When `true` the active player should change to the next player in turn
    /// order (i.e. the new step is `Untap`, starting a new turn).
    pub should_advance_player: bool,
}

/// The canonical turn order of all steps.
const STEP_ORDER: [Step; 13] = [
    Step::Untap,
    Step::Upkeep,
    Step::Draw,
    Step::FirstMain,
    Step::BeginningOfCombat,
    Step::DeclareAttackers,
    Step::DeclareBlockers,
    Step::FirstStrikeDamage,
    Step::CombatDamage,
    Step::EndOfCombat,
    Step::SecondMain,
    Step::EndStep,
    Step::Cleanup,
];

/// Maps each `Step` variant to its index in `STEP_ORDER`.
///
/// Uses an exhaustive `match` so that adding a new `Step` variant triggers a
/// compile error here, forcing the author to update `STEP_ORDER` as well.
const fn step_index(step: Step) -> usize {
    match step {
        Step::Untap => 0,
        Step::Upkeep => 1,
        Step::Draw => 2,
        Step::FirstMain => 3,
        Step::BeginningOfCombat => 4,
        Step::DeclareAttackers => 5,
        Step::DeclareBlockers => 6,
        Step::FirstStrikeDamage => 7,
        Step::CombatDamage => 8,
        Step::EndOfCombat => 9,
        Step::SecondMain => 10,
        Step::EndStep => 11,
        Step::Cleanup => 12,
    }
}

/// Returns the next step in the turn order and whether the active player
/// should advance.
///
/// Wraps around: the step after `Cleanup` is `Untap` (new turn).
pub(crate) fn advance(current: Step) -> StepResult {
    let current_index = step_index(current);

    let next_index = (current_index + 1) % STEP_ORDER.len();
    let next_step = STEP_ORDER[next_index];

    StepResult {
        next_step,
        should_advance_player: next_step == Step::Untap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn untap_advances_to_upkeep() {
        let result = advance(Step::Untap);
        assert_eq!(result.next_step, Step::Upkeep);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn upkeep_advances_to_draw() {
        let result = advance(Step::Upkeep);
        assert_eq!(result.next_step, Step::Draw);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn draw_advances_to_first_main() {
        let result = advance(Step::Draw);
        assert_eq!(result.next_step, Step::FirstMain);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn first_main_advances_to_beginning_of_combat() {
        let result = advance(Step::FirstMain);
        assert_eq!(result.next_step, Step::BeginningOfCombat);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn beginning_of_combat_advances_to_declare_attackers() {
        let result = advance(Step::BeginningOfCombat);
        assert_eq!(result.next_step, Step::DeclareAttackers);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn declare_attackers_advances_to_declare_blockers() {
        let result = advance(Step::DeclareAttackers);
        assert_eq!(result.next_step, Step::DeclareBlockers);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn declare_blockers_advances_to_first_strike_damage() {
        let result = advance(Step::DeclareBlockers);
        assert_eq!(result.next_step, Step::FirstStrikeDamage);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn first_strike_damage_advances_to_combat_damage() {
        let result = advance(Step::FirstStrikeDamage);
        assert_eq!(result.next_step, Step::CombatDamage);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn combat_damage_advances_to_end_of_combat() {
        let result = advance(Step::CombatDamage);
        assert_eq!(result.next_step, Step::EndOfCombat);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn end_of_combat_advances_to_second_main() {
        let result = advance(Step::EndOfCombat);
        assert_eq!(result.next_step, Step::SecondMain);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn second_main_advances_to_end_step() {
        let result = advance(Step::SecondMain);
        assert_eq!(result.next_step, Step::EndStep);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn end_step_advances_to_cleanup() {
        let result = advance(Step::EndStep);
        assert_eq!(result.next_step, Step::Cleanup);
        assert!(!result.should_advance_player);
    }

    #[test]
    fn cleanup_wraps_to_untap_and_advances_player() {
        let result = advance(Step::Cleanup);
        assert_eq!(result.next_step, Step::Untap);
        assert!(result.should_advance_player);
    }

    #[test]
    fn full_turn_cycle_returns_to_untap() {
        let mut step = Step::Untap;
        for _ in 0..13 {
            let result = advance(step);
            step = result.next_step;
        }
        assert_eq!(step, Step::Untap);
    }
}
