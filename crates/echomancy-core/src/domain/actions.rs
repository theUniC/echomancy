use serde::{Deserialize, Serialize};

use crate::domain::targets::Target;
use crate::domain::types::{CardInstanceId, PlayerId};

/// Represents the number of cards to draw.
pub type DrawAmount = u32;

/// All actions a player can perform during a game.
///
/// Each variant carries the data required to process that action.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    /// Advance to the next step/phase of the current turn.
    AdvanceStep {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
    },

    /// End the current player's turn and pass to the next player.
    EndTurn {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
    },

    /// Play a land card from hand to the battlefield.
    PlayLand {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "cardId")]
        card_id: CardInstanceId,
    },

    /// Cast a spell from hand onto the stack.
    CastSpell {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "cardId")]
        card_id: CardInstanceId,
        /// Chosen targets for the spell. Empty when the spell requires no targets.
        /// `#[serde(default)]` ensures old serialized actions without this field
        /// still deserialize correctly (backward compatibility).
        #[serde(default)]
        targets: Vec<Target>,
        /// Chosen value of X for spells with X in their mana cost (CR 107.3).
        /// Defaults to 0 for backward compatibility and non-X spells.
        #[serde(default, rename = "xValue")]
        x_value: u32,
    },

    /// Pass priority to the next player.
    PassPriority {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
    },

    /// Declare a creature as an attacker in the declare-attackers step.
    DeclareAttacker {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "creatureId")]
        creature_id: CardInstanceId,
    },

    /// Declare a creature as a blocker in the declare-blockers step.
    DeclareBlocker {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "blockerId")]
        blocker_id: CardInstanceId,
        #[serde(rename = "attackerId")]
        attacker_id: CardInstanceId,
    },

    /// Activate a permanent's activated ability.
    ///
    /// `ability_index` selects which ability to activate when the permanent
    /// has more than one (CR 602.1). Defaults to `0` for backward compatibility
    /// with serialized actions that pre-date this field.
    ActivateAbility {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "permanentId")]
        permanent_id: CardInstanceId,
        /// Zero-based index into the permanent's `activated_abilities` list.
        #[serde(default, rename = "abilityIndex")]
        ability_index: usize,
    },

    /// Draw one or more cards from a player's library.
    DrawCard {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        amount: DrawAmount,
    },

    /// Sacrifice a permanent you control (CR 701.17).
    Sacrifice {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "permanentId")]
        permanent_id: CardInstanceId,
    },

    // -------------------------------------------------------------------------
    // Mulligan actions (only valid during the mulligan phase)
    // -------------------------------------------------------------------------

    /// Keep the current opening hand and complete the mulligan phase for this player.
    ///
    /// If the player has taken N mulligans, they will enter the put-back sub-step
    /// and must place N cards on the bottom of their library.
    MulliganKeep {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
    },

    /// Shuffle the current hand back into the library and draw 7 new cards,
    /// incrementing the player's mulligan count by 1.
    MulliganRedraw {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
    },

    /// Place the specified card from the player's hand on the bottom of their library.
    ///
    /// Valid only after `MulliganKeep` when the player has cards to put back.
    PutCardOnBottom {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "cardId")]
        card_id: CardInstanceId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_step_serde_roundtrip() {
        let action = Action::AdvanceStep {
            player_id: PlayerId::new("player-1"),
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn cast_spell_serde_roundtrip() {
        let action = Action::CastSpell {
            player_id: PlayerId::new("player-1"),
            card_id: CardInstanceId::new("card-abc"),
            targets: vec![],
            x_value: 0,
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn cast_spell_with_player_target_serde_roundtrip() {
        use crate::domain::targets::Target;
        let action = Action::CastSpell {
            player_id: PlayerId::new("player-1"),
            card_id: CardInstanceId::new("card-abc"),
            targets: vec![Target::player("player-2")],
            x_value: 0,
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn cast_spell_with_creature_target_serde_roundtrip() {
        use crate::domain::targets::Target;
        let action = Action::CastSpell {
            player_id: PlayerId::new("player-1"),
            card_id: CardInstanceId::new("card-abc"),
            targets: vec![Target::creature("perm-42")],
            x_value: 0,
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn cast_spell_old_json_without_targets_deserializes_with_empty_vec() {
        // Backward compatibility: JSON produced before the targets field was added
        // should still deserialize successfully (serde default = empty vec).
        let old_json = r#"{"type":"CAST_SPELL","playerId":"p1","cardId":"c1"}"#;
        let decoded: Action = serde_json::from_str(old_json).unwrap();
        assert_eq!(
            decoded,
            Action::CastSpell {
                player_id: PlayerId::new("p1"),
                card_id: CardInstanceId::new("c1"),
                targets: vec![],
                x_value: 0,
            }
        );
    }

    #[test]
    fn cast_spell_x_value_serde_roundtrip() {
        let action = Action::CastSpell {
            player_id: PlayerId::new("player-1"),
            card_id: CardInstanceId::new("fireball"),
            targets: vec![],
            x_value: 5,
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn cast_spell_old_json_without_x_value_deserializes_to_zero() {
        // Backward compatibility: JSON without xValue field defaults to 0.
        let old_json = r#"{"type":"CAST_SPELL","playerId":"p1","cardId":"c1","targets":[]}"#;
        let decoded: Action = serde_json::from_str(old_json).unwrap();
        assert_eq!(
            decoded,
            Action::CastSpell {
                player_id: PlayerId::new("p1"),
                card_id: CardInstanceId::new("c1"),
                targets: vec![],
                x_value: 0,
            }
        );
    }

    #[test]
    fn declare_blocker_serde_roundtrip() {
        let action = Action::DeclareBlocker {
            player_id: PlayerId::new("player-2"),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn draw_card_serde_roundtrip() {
        let action = Action::DrawCard {
            player_id: PlayerId::new("player-1"),
            amount: 7,
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn action_type_tag_in_json() {
        let action = Action::EndTurn {
            player_id: PlayerId::new("p1"),
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"type\":\"END_TURN\""));
    }

    // ---- ActivateAbility serde (R15 / CR 602) --------------------------------

    #[test]
    fn activate_ability_serde_roundtrip_with_index() {
        let action = Action::ActivateAbility {
            player_id: PlayerId::new("player-1"),
            permanent_id: CardInstanceId::new("land-1"),
            ability_index: 1,
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
    }

    #[test]
    fn activate_ability_old_json_without_index_deserializes_to_index_0() {
        // Backward compatibility: JSON produced before ability_index was added
        // should still deserialize correctly (serde default = 0).
        let old_json = r#"{"type":"ACTIVATE_ABILITY","playerId":"p1","permanentId":"land-1"}"#;
        let decoded: Action = serde_json::from_str(old_json).unwrap();
        assert_eq!(
            decoded,
            Action::ActivateAbility {
                player_id: PlayerId::new("p1"),
                permanent_id: CardInstanceId::new("land-1"),
                ability_index: 0,
            }
        );
    }
}
