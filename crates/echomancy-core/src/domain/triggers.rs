//! Triggered ability system.
//!
//! Triggers are conditions that fire automatically when a game event occurs.
//! In the TS codebase, `Trigger` was a struct with function fields (closures).
//! In Rust, closures stored in structs require complex lifetime management.
//!
//! MVP approach: triggers are represented as a closed-set enum of named
//! trigger patterns. This keeps the type system clean and avoids `dyn Fn`
//! complexity. For user-defined or data-driven triggers, this can be
//! extended when needed.
//!
//! Mirrors the TypeScript `Trigger` type from `triggers/Trigger.ts`.

use crate::domain::effects::Effect;

/// The game event type that a trigger listens for.
///
/// A subset of `GameEvent` variants — only those that can fire triggers
/// in the MVP rules engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TriggerEventType {
    /// A card changed zones (e.g., creature dying).
    ZoneChanged,
    /// A new step or phase started.
    StepStarted,
    /// A creature was declared as an attacker.
    CreatureDeclaredAttacker,
    /// Combat ended.
    CombatEnded,
}

/// A triggered ability: fires when `event_type` occurs and `condition` is met.
///
/// The condition is encoded as a `TriggerCondition` (closed set) rather than
/// a raw closure, keeping the type `Clone + PartialEq` and avoiding lifetime
/// issues with stored `dyn Fn`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trigger {
    /// The event category this trigger listens for.
    pub event_type: TriggerEventType,
    /// The condition that must be true for the trigger to fire.
    pub condition: TriggerCondition,
    /// The effect that executes when the trigger fires.
    pub effect: Effect,
}

impl Trigger {
    /// Create a new trigger.
    pub fn new(
        event_type: TriggerEventType,
        condition: TriggerCondition,
        effect: Effect,
    ) -> Self {
        Trigger {
            event_type,
            condition,
            effect,
        }
    }
}

/// A closed-set of trigger conditions supported in the MVP.
///
/// Each variant encodes one evaluatable condition without requiring closures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerCondition {
    /// The trigger always fires (unconditional).
    Always,
    /// Only fires when the source card itself enters the graveyard.
    SourceDies,
    /// Only fires at the beginning of a specific step for the active player.
    AtStepStart { step: crate::domain::enums::Step },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::Step;

    #[test]
    fn trigger_new_always_fires() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::NoOp,
        );
        assert_eq!(trigger.event_type, TriggerEventType::ZoneChanged);
        assert_eq!(trigger.condition, TriggerCondition::Always);
        assert_eq!(trigger.effect, Effect::NoOp);
    }

    #[test]
    fn trigger_source_dies_condition() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::SourceDies,
            Effect::draw_cards(1),
        );
        assert_eq!(trigger.condition, TriggerCondition::SourceDies);
    }

    #[test]
    fn trigger_at_step_start_condition() {
        let trigger = Trigger::new(
            TriggerEventType::StepStarted,
            TriggerCondition::AtStepStart {
                step: Step::Upkeep,
            },
            Effect::NoOp,
        );
        assert_eq!(
            trigger.condition,
            TriggerCondition::AtStepStart { step: Step::Upkeep }
        );
    }

    #[test]
    fn trigger_clone() {
        let trigger = Trigger::new(
            TriggerEventType::CreatureDeclaredAttacker,
            TriggerCondition::Always,
            Effect::NoOp,
        );
        let cloned = trigger.clone();
        assert_eq!(trigger, cloned);
    }
}
