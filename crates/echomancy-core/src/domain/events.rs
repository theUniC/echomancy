use serde::{Deserialize, Serialize};

use crate::domain::enums::{ManaColor, Step, ZoneName};
use crate::domain::targets::Target;
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

    /// A creature was declared as a blocker.
    CreatureDeclaredBlocker {
        creature: CardInstanceSnapshot,
        #[serde(rename = "controllerId")]
        controller_id: PlayerId,
        blocking: CardInstanceSnapshot,
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
        /// The targets the spell had when it was cast (CR 608.2b: re-checked at resolution).
        /// Empty for spells with no target requirement.
        #[serde(default)]
        targets: Vec<Target>,
    },

    /// Mana was added to a player's pool (from a mana ability).
    ManaAdded {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        color: ManaColor,
        amount: u32,
    },

    /// A triggered ability has fired from a permanent on the battlefield.
    ///
    /// Emitted by `execute_triggered_abilities()` before calling the rules
    /// engine so that CLIPS rules can match on this specific event type and
    /// the source permanent.
    TriggeredAbilityFires {
        /// The permanent that is the source of the trigger.
        source: CardInstanceSnapshot,
        /// The player who controls the triggered ability.
        #[serde(rename = "controllerId")]
        controller_id: PlayerId,
        /// A short string identifying the trigger category (e.g. "ETB", "DEATH",
        /// "STEP_START", "ATTACK"). Used by CLIPS rules for pattern matching.
        #[serde(rename = "triggerType")]
        trigger_type: String,
    },

    /// A player drew a card (TR5).
    ///
    /// Emitted once per individual card drawn. Enables "whenever you draw a card"
    /// triggered abilities (e.g. Niv-Mizzet, the Firemind).
    CardDrawn {
        /// The player who drew the card.
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        /// The instance ID of the drawn card.
        #[serde(rename = "cardId")]
        card_id: CardInstanceId,
    },

    /// A spell was cast (TR6).
    ///
    /// Emitted when a spell is placed on the stack. Enables "whenever you cast
    /// a spell" triggered abilities (e.g. Guttersnipe, Young Pyromancer).
    SpellCast {
        /// The instance ID of the spell on the stack.
        #[serde(rename = "cardId")]
        card_id: CardInstanceId,
        /// The definition ID of the spell (card type/name key).
        #[serde(rename = "cardDefinitionId")]
        card_definition_id: CardDefinitionId,
        /// The player who cast the spell.
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
    fn creature_declared_blocker_serde_roundtrip() {
        let event = GameEvent::CreatureDeclaredBlocker {
            creature: make_snapshot(),
            controller_id: PlayerId::new("player-2"),
            blocking: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("attacker-1"),
                definition_id: CardDefinitionId::new("bear"),
                owner_id: PlayerId::new("player-1"),
            },
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
            targets: vec![],
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

    #[test]
    fn triggered_ability_fires_serde_roundtrip() {
        let event = GameEvent::TriggeredAbilityFires {
            source: make_snapshot(),
            controller_id: PlayerId::new("player-1"),
            trigger_type: "ETB".to_owned(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn triggered_ability_fires_type_tag_in_json() {
        let event = GameEvent::TriggeredAbilityFires {
            source: make_snapshot(),
            controller_id: PlayerId::new("p1"),
            trigger_type: "DEATH".to_owned(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"TRIGGERED_ABILITY_FIRES\""));
    }

    #[test]
    fn card_drawn_serde_roundtrip() {
        let event = GameEvent::CardDrawn {
            player_id: PlayerId::new("player-1"),
            card_id: CardInstanceId::new("card-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn card_drawn_type_tag_in_json() {
        let event = GameEvent::CardDrawn {
            player_id: PlayerId::new("p1"),
            card_id: CardInstanceId::new("c1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"CARD_DRAWN\""));
    }

    #[test]
    fn spell_cast_serde_roundtrip() {
        let event = GameEvent::SpellCast {
            card_id: CardInstanceId::new("card-1"),
            card_definition_id: CardDefinitionId::new("lightning-bolt"),
            controller_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, decoded);
    }

    #[test]
    fn spell_cast_type_tag_in_json() {
        let event = GameEvent::SpellCast {
            card_id: CardInstanceId::new("c1"),
            card_definition_id: CardDefinitionId::new("def-1"),
            controller_id: PlayerId::new("p1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"SPELL_CAST\""));
    }
}
