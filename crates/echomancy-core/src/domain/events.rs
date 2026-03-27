use serde::{Deserialize, Serialize};

use crate::domain::enums::{Step, ZoneName};
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

/// A snapshot of a card instance for event payloads.
///
/// Carries the minimum information needed by event consumers without
/// requiring a full card object reference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardInstanceSnapshot {
    #[serde(rename = "instanceId")]
    pub instance_id: CardInstanceId,
    #[serde(rename = "definitionId")]
    pub definition_id: CardDefinitionId,
    #[serde(rename = "ownerId")]
    pub owner_id: PlayerId,
}

/// All domain events that can be emitted during a game.
///
/// Mirrors the TypeScript `GameEvent` discriminated union from `GameEvents.ts`.
/// Events are produced by the rules engine and consumed by triggers, UI, and
/// any external observers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameEvent {
    /// A card moved from one zone to another.
    ZoneChanged {
        card: CardInstanceSnapshot,
        #[serde(rename = "fromZone")]
        from_zone: ZoneName,
        #[serde(rename = "toZone")]
        to_zone: ZoneName,
        #[serde(rename = "controllerId")]
        controller_id: PlayerId,
    },

    /// A new step or phase started.
    StepStarted {
        step: Step,
        #[serde(rename = "activePlayerId")]
        active_player_id: PlayerId,
    },

    /// A creature was declared as an attacker.
    CreatureDeclaredAttacker {
        creature: CardInstanceSnapshot,
        #[serde(rename = "controllerId")]
        controller_id: PlayerId,
    },

    /// The combat phase ended.
    CombatEnded {
        #[serde(rename = "activePlayerId")]
        active_player_id: PlayerId,
    },

    /// A spell resolved from the stack.
    SpellResolved {
        card: CardInstanceSnapshot,
        #[serde(rename = "controllerId")]
        controller_id: PlayerId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::{Step, ZoneName};

    fn make_snapshot() -> CardInstanceSnapshot {
        CardInstanceSnapshot {
            instance_id: CardInstanceId::new("inst-1"),
            definition_id: CardDefinitionId::new("lightning-bolt"),
            owner_id: PlayerId::new("player-1"),
        }
    }

    #[test]
    fn zone_changed_serde_roundtrip() {
        let event = GameEvent::ZoneChanged {
            card: make_snapshot(),
            from_zone: ZoneName::Hand,
            to_zone: ZoneName::Stack,
            controller_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn step_started_serde_roundtrip() {
        let event = GameEvent::StepStarted {
            step: Step::FirstMain,
            active_player_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn creature_declared_attacker_serde_roundtrip() {
        let event = GameEvent::CreatureDeclaredAttacker {
            creature: make_snapshot(),
            controller_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn combat_ended_serde_roundtrip() {
        let event = GameEvent::CombatEnded {
            active_player_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn spell_resolved_serde_roundtrip() {
        let event = GameEvent::SpellResolved {
            card: make_snapshot(),
            controller_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn event_type_tag_in_json() {
        let event = GameEvent::CombatEnded {
            active_player_id: PlayerId::new("p1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"COMBAT_ENDED\""));
    }
}
