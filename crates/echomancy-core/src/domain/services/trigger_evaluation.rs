//! TriggerEvaluation — scan battlefields for matching triggers.
//!
//! Stateless service that identifies which triggered abilities should fire in
//! response to a game event. Returns a list of triggers ready for the caller
//! (Game aggregate) to execute.
//!
//! MVP limitations (mirrors `TriggerEvaluation.ts`):
//! - Triggers execute immediately (not placed on the stack in the MVP).
//! - No APNAP ordering.
//! - No targeting within trigger effects.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::effects::Effect;
use crate::domain::enums::ZoneName;
use crate::domain::events::GameEvent;
use crate::domain::triggers::{TriggerCondition, TriggerEventType};

// ============================================================================
// Data structures
// ============================================================================

/// A permanent on the battlefield with its controller.
pub(crate) struct PermanentOnBattlefield<'a> {
    pub permanent: &'a CardInstance,
    pub controller_id: &'a str,
}

/// A triggered ability that has been matched and is ready to execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TriggeredAbilityInfo {
    /// The effect to execute when this trigger fires.
    pub effect: Effect,
    /// The player who controls the triggered ability.
    pub controller_id: String,
    /// The instance ID of the permanent that is the source.
    pub source_id: String,
}

// ============================================================================
// Service functions
// ============================================================================

/// Returns all triggered abilities that match the given `event` for the
/// permanents supplied.
///
/// # Parameters
///
/// - `permanents` — all permanents on all battlefields, each paired with
///   their controller ID. The caller (Game aggregate) provides this.
/// - `event` — the game event that may trigger abilities.
///
/// # Returns
///
/// `Vec<TriggeredAbilityInfo>` in the order the permanents were supplied.
pub(crate) fn find_matching_triggers<'a>(
    permanents: &[PermanentOnBattlefield<'a>],
    event: &GameEvent,
) -> Vec<TriggeredAbilityInfo> {
    let mut triggered = Vec::new();

    for entry in permanents {
        for trigger in entry.permanent.definition().triggers() {
            // Fast path: skip if the event type doesn't match.
            if !event_matches_trigger_type(event, &trigger.event_type) {
                continue;
            }

            // Evaluate the condition.
            if evaluate_condition(&trigger.condition, event, entry.permanent) {
                triggered.push(TriggeredAbilityInfo {
                    effect: trigger.effect.clone(),
                    controller_id: entry.controller_id.to_owned(),
                    source_id: entry.permanent.instance_id().to_owned(),
                });
            }
        }
    }

    triggered
}

// ============================================================================
// Private helpers
// ============================================================================

/// Returns `true` if the `GameEvent` variant corresponds to the
/// `TriggerEventType`.
///
/// Every combination of `(GameEvent, TriggerEventType)` is listed
/// exhaustively so that adding a new variant to either enum causes a
/// compile error, preventing a silent miss.
fn event_matches_trigger_type(event: &GameEvent, event_type: &TriggerEventType) -> bool {
    match (event, event_type) {
        // Matching pairs.
        (GameEvent::ZoneChanged { .. }, TriggerEventType::ZoneChanged) => true,
        (GameEvent::StepStarted { .. }, TriggerEventType::StepStarted) => true,
        (GameEvent::CreatureDeclaredAttacker { .. }, TriggerEventType::CreatureDeclaredAttacker) => {
            true
        }
        (GameEvent::CombatEnded { .. }, TriggerEventType::CombatEnded) => true,
        // Non-matching combinations — ZoneChanged event.
        (GameEvent::ZoneChanged { .. }, TriggerEventType::StepStarted) => false,
        (GameEvent::ZoneChanged { .. }, TriggerEventType::CreatureDeclaredAttacker) => false,
        (GameEvent::ZoneChanged { .. }, TriggerEventType::CombatEnded) => false,
        // Non-matching combinations — StepStarted event.
        (GameEvent::StepStarted { .. }, TriggerEventType::ZoneChanged) => false,
        (GameEvent::StepStarted { .. }, TriggerEventType::CreatureDeclaredAttacker) => false,
        (GameEvent::StepStarted { .. }, TriggerEventType::CombatEnded) => false,
        // Non-matching combinations — CreatureDeclaredAttacker event.
        (GameEvent::CreatureDeclaredAttacker { .. }, TriggerEventType::ZoneChanged) => false,
        (GameEvent::CreatureDeclaredAttacker { .. }, TriggerEventType::StepStarted) => false,
        (GameEvent::CreatureDeclaredAttacker { .. }, TriggerEventType::CombatEnded) => false,
        // Non-matching combinations — CombatEnded event.
        (GameEvent::CombatEnded { .. }, TriggerEventType::ZoneChanged) => false,
        (GameEvent::CombatEnded { .. }, TriggerEventType::StepStarted) => false,
        (GameEvent::CombatEnded { .. }, TriggerEventType::CreatureDeclaredAttacker) => false,
        // Non-matching combinations — SpellResolved event (no trigger type maps to it yet).
        (GameEvent::SpellResolved { .. }, TriggerEventType::ZoneChanged) => false,
        (GameEvent::SpellResolved { .. }, TriggerEventType::StepStarted) => false,
        (GameEvent::SpellResolved { .. }, TriggerEventType::CreatureDeclaredAttacker) => false,
        (GameEvent::SpellResolved { .. }, TriggerEventType::CombatEnded) => false,
        // Non-matching combinations — CreatureDeclaredBlocker event (no trigger type maps to it yet).
        (GameEvent::CreatureDeclaredBlocker { .. }, TriggerEventType::ZoneChanged) => false,
        (GameEvent::CreatureDeclaredBlocker { .. }, TriggerEventType::StepStarted) => false,
        (GameEvent::CreatureDeclaredBlocker { .. }, TriggerEventType::CreatureDeclaredAttacker) => {
            false
        }
        (GameEvent::CreatureDeclaredBlocker { .. }, TriggerEventType::CombatEnded) => false,
    }
}

/// Evaluates a `TriggerCondition` for the given event and source permanent.
fn evaluate_condition(
    condition: &TriggerCondition,
    event: &GameEvent,
    source: &CardInstance,
) -> bool {
    match condition {
        TriggerCondition::Always => true,
        TriggerCondition::SourceDies => {
            // Fires when the source card itself moved to the graveyard.
            match event {
                GameEvent::ZoneChanged {
                    card,
                    to_zone,
                    ..
                } => {
                    card.instance_id == source.instance_id().into()
                        && *to_zone == ZoneName::Graveyard
                }
                _ => false,
            }
        }
        TriggerCondition::AtStepStart { step } => match event {
            GameEvent::StepStarted { step: event_step, .. } => event_step == step,
            _ => false,
        },
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::effects::Effect;
    use crate::domain::enums::{CardType, Step, ZoneName};
    use crate::domain::events::{CardInstanceSnapshot, GameEvent};
    use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};
    use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

    // ---- helpers ------------------------------------------------------------

    fn make_card_with_trigger(id: &str, owner: &str, trigger: Trigger) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_trigger(trigger);
        CardInstance::new(id, def, owner)
    }

    fn make_card_no_triggers(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(1, 1);
        CardInstance::new(id, def, owner)
    }

    fn snapshot(id: &str) -> CardInstanceSnapshot {
        CardInstanceSnapshot {
            instance_id: CardInstanceId::new(id),
            definition_id: CardDefinitionId::new(id),
            owner_id: PlayerId::new("p1"),
        }
    }

    fn zone_changed_event(
        instance_id: &str,
        from: ZoneName,
        to: ZoneName,
    ) -> GameEvent {
        GameEvent::ZoneChanged {
            card: snapshot(instance_id),
            from_zone: from,
            to_zone: to,
            controller_id: PlayerId::new("p1"),
        }
    }

    fn step_started_event(step: Step) -> GameEvent {
        GameEvent::StepStarted {
            step,
            active_player_id: PlayerId::new("p1"),
        }
    }

    fn combat_ended_event() -> GameEvent {
        GameEvent::CombatEnded {
            active_player_id: PlayerId::new("p1"),
        }
    }

    // ---- Always condition --------------------------------------------------

    #[test]
    fn always_condition_fires_on_matching_event_type() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::draw_cards(1),
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = zone_changed_event("some-card", ZoneName::Hand, ZoneName::Graveyard);
        let result = find_matching_triggers(&[entry], &event);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].effect, Effect::draw_cards(1));
        assert_eq!(result[0].controller_id, "p1");
        assert_eq!(result[0].source_id, "c1");
    }

    #[test]
    fn always_condition_does_not_fire_on_wrong_event_type() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::draw_cards(1),
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = step_started_event(Step::Upkeep);
        let result = find_matching_triggers(&[entry], &event);
        assert!(result.is_empty());
    }

    // ---- SourceDies condition ----------------------------------------------

    #[test]
    fn source_dies_fires_when_source_moves_to_graveyard() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::SourceDies,
            Effect::draw_cards(1),
        );
        let card = make_card_with_trigger("dying-card", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = zone_changed_event("dying-card", ZoneName::Battlefield, ZoneName::Graveyard);
        let result = find_matching_triggers(&[entry], &event);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn source_dies_does_not_fire_when_different_card_dies() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::SourceDies,
            Effect::draw_cards(1),
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        // A different card moves to the graveyard.
        let event = zone_changed_event("other-card", ZoneName::Battlefield, ZoneName::Graveyard);
        let result = find_matching_triggers(&[entry], &event);
        assert!(result.is_empty());
    }

    #[test]
    fn source_dies_does_not_fire_when_card_moves_to_hand_not_graveyard() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::SourceDies,
            Effect::draw_cards(1),
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        // Source card moves to hand (bounce), not graveyard.
        let event = zone_changed_event("c1", ZoneName::Battlefield, ZoneName::Hand);
        let result = find_matching_triggers(&[entry], &event);
        assert!(result.is_empty());
    }

    // ---- AtStepStart condition --------------------------------------------

    #[test]
    fn at_step_start_fires_on_matching_step() {
        let trigger = Trigger::new(
            TriggerEventType::StepStarted,
            TriggerCondition::AtStepStart { step: Step::Upkeep },
            Effect::draw_cards(2),
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = step_started_event(Step::Upkeep);
        let result = find_matching_triggers(&[entry], &event);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].effect, Effect::draw_cards(2));
    }

    #[test]
    fn at_step_start_does_not_fire_on_different_step() {
        let trigger = Trigger::new(
            TriggerEventType::StepStarted,
            TriggerCondition::AtStepStart { step: Step::Upkeep },
            Effect::draw_cards(1),
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = step_started_event(Step::Draw);
        let result = find_matching_triggers(&[entry], &event);
        assert!(result.is_empty());
    }

    // ---- No triggers -------------------------------------------------------

    #[test]
    fn card_with_no_triggers_returns_empty() {
        let card = make_card_no_triggers("c1", "p1");
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = zone_changed_event("c1", ZoneName::Hand, ZoneName::Graveyard);
        let result = find_matching_triggers(&[entry], &event);
        assert!(result.is_empty());
    }

    // ---- Multiple permanents ---------------------------------------------

    #[test]
    fn multiple_permanents_multiple_triggers() {
        let t1 = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::draw_cards(1),
        );
        let t2 = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::draw_cards(2),
        );
        let c1 = make_card_with_trigger("c1", "p1", t1);
        let c2 = make_card_with_trigger("c2", "p2", t2);

        let entries = [
            PermanentOnBattlefield {
                permanent: &c1,
                controller_id: "p1",
            },
            PermanentOnBattlefield {
                permanent: &c2,
                controller_id: "p2",
            },
        ];

        let event = zone_changed_event("x", ZoneName::Hand, ZoneName::Battlefield);
        let result = find_matching_triggers(&entries, &event);

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.controller_id == "p1"));
        assert!(result.iter().any(|t| t.controller_id == "p2"));
    }

    #[test]
    fn combat_ended_triggers_fire_on_combat_ended_event() {
        let trigger = Trigger::new(
            TriggerEventType::CombatEnded,
            TriggerCondition::Always,
            Effect::NoOp,
        );
        let card = make_card_with_trigger("c1", "p1", trigger);
        let entry = PermanentOnBattlefield {
            permanent: &card,
            controller_id: "p1",
        };

        let event = combat_ended_event();
        let result = find_matching_triggers(&[entry], &event);
        assert_eq!(result.len(), 1);
    }
}
