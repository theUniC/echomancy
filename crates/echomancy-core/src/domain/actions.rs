use serde::{Deserialize, Serialize};

use crate::domain::types::{CardInstanceId, PlayerId};

/// Represents the number of cards to draw.
pub type DrawAmount = u32;

/// All actions a player can perform during a game.
///
/// Mirrors the TypeScript `Actions` discriminated union from `GameActions.ts`.
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
    ActivateAbility {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        #[serde(rename = "permanentId")]
        permanent_id: CardInstanceId,
    },

    /// Draw one or more cards from a player's library.
    DrawCard {
        #[serde(rename = "playerId")]
        player_id: PlayerId,
        amount: DrawAmount,
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
        };
        let json = serde_json::to_string(&action).unwrap();
        let decoded: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, decoded);
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
}
