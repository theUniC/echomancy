use thiserror::Error;

use crate::domain::types::{CardInstanceId, PlayerId};

/// All errors that can occur during game processing.
///
/// Mirrors the TypeScript error classes from `GameErrors.ts`, grouped by
/// the sub-system that can raise them.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GameError {
    // ========================================================================
    // Game setup / lifecycle errors
    // ========================================================================
    #[error("Game requires at least 2 players, but got {player_count}")]
    InvalidPlayerCount { player_count: usize },

    #[error("Starting player with id '{player_id}' is not in the player list")]
    InvalidStartingPlayer { player_id: PlayerId },

    #[error("Missing deck for player(s): {player_ids}")]
    MissingDeck { player_ids: String },

    #[error("Cannot perform actions on a game that has not been started")]
    GameNotStarted,

    #[error("Game has already been started")]
    GameAlreadyStarted,

    #[error("Cannot perform actions on a game that has finished")]
    GameFinished,

    #[error("Cannot add players after the game has started")]
    CannotAddPlayerAfterStart,

    #[error("Player with id '{player_id}' has already been added to the game")]
    DuplicatePlayer { player_id: PlayerId },

    #[error("Game with id '{game_id}' not found")]
    GameNotFound { game_id: String },

    #[error("Invalid player id: '{player_id}' is not a valid UUID")]
    InvalidPlayerId { player_id: String },

    // ========================================================================
    // Player / turn errors
    // ========================================================================
    #[error("Player with id '{player_id}' not found in game")]
    PlayerNotFound { player_id: PlayerId },

    #[error(
        "Player '{player_id}' cannot perform action '{action}': only the current player can advance the step"
    )]
    InvalidPlayerAction { player_id: PlayerId, action: String },

    #[error("Cannot end turn from CLEANUP step")]
    InvalidEndTurn,

    // ========================================================================
    // Land / spell timing errors
    // ========================================================================
    #[error("Can only play lands during main phases")]
    InvalidPlayLandStep,

    #[error("Cannot play more than one land per turn")]
    LandLimitExceeded,

    #[error("Card '{card_id}' not found in hand of player '{player_id}'")]
    CardNotFoundInHand {
        card_id: CardInstanceId,
        player_id: PlayerId,
    },

    #[error("Card '{card_id}' is not a land")]
    CardIsNotLand { card_id: CardInstanceId },

    #[error("Can only cast spells during main phases")]
    InvalidCastSpellStep,

    #[error("Card '{card_id}' is not a spell")]
    CardIsNotSpell { card_id: CardInstanceId },

    #[error("Effect '{effect_name}' failed: {reason}")]
    InvalidEffectTarget { effect_name: String, reason: String },

    // ========================================================================
    // Permanent / battlefield errors
    // ========================================================================
    #[error("Permanent '{permanent_id}' not found on battlefield")]
    PermanentNotFound { permanent_id: CardInstanceId },

    #[error("Permanent '{permanent_id}' has no activated ability")]
    PermanentHasNoActivatedAbility { permanent_id: CardInstanceId },

    #[error("Player '{player_id}' has no permanent with an activatable ability")]
    NoActivatableAbility { player_id: PlayerId },

    #[error("Cannot activate ability of '{permanent_id}': {reason}")]
    CannotPayActivationCost {
        permanent_id: CardInstanceId,
        reason: String,
    },

    #[error("Permanent '{permanent_id}' is already tapped")]
    PermanentAlreadyTapped { permanent_id: CardInstanceId },

    #[error("Player '{player_id}' does not control permanent '{permanent_id}'")]
    PermanentNotControlled {
        permanent_id: CardInstanceId,
        player_id: PlayerId,
    },

    #[error("Cannot pay costs: {reason}")]
    CannotPayCosts { reason: String },

    // ========================================================================
    // Mana errors
    // ========================================================================
    #[error(
        "Player '{player_id}' has insufficient {color} mana: requested {requested}, available {available}"
    )]
    InsufficientMana {
        player_id: PlayerId,
        color: String,
        requested: u32,
        available: u32,
    },

    #[error("Invalid mana amount: {amount}. Amount must be greater than 0.")]
    InvalidManaAmount { amount: i64 },

    #[error("Insufficient mana to cast spell: {message}")]
    InsufficientManaForSpell { message: String },

    // ========================================================================
    // Creature / combat errors
    // ========================================================================
    #[error("Creature '{creature_id}' has already attacked this turn")]
    CreatureAlreadyAttacked { creature_id: CardInstanceId },

    #[error("Creature '{creature_id}' is tapped and cannot attack")]
    TappedCreatureCannotAttack { creature_id: CardInstanceId },

    #[error("Creature '{creature_id}' is tapped and cannot block")]
    TappedCreatureCannotBlock { creature_id: CardInstanceId },

    #[error("Creature '{creature_id}' is already blocking another creature")]
    CreatureAlreadyBlocking { creature_id: CardInstanceId },

    #[error("Creature '{attacker_id}' is not attacking and cannot be blocked")]
    CannotBlockNonAttackingCreature { attacker_id: CardInstanceId },

    #[error("Creature '{attacker_id}' is already blocked (MVP: only one blocker per attacker)")]
    AttackerAlreadyBlocked { attacker_id: CardInstanceId },

    #[error(
        "Creature '{blocker_id}' cannot block flying creature '{attacker_id}' (blocker must have Flying or Reach)"
    )]
    CannotBlockFlyingCreature {
        blocker_id: CardInstanceId,
        attacker_id: CardInstanceId,
    },

    #[error(
        "Creature '{creature_id}' has summoning sickness and cannot attack or use tap abilities this turn"
    )]
    CreatureHasSummoningSickness { creature_id: CardInstanceId },

    #[error("Invalid counter amount: {amount}. Amount must be greater than 0.")]
    InvalidCounterAmount { amount: i64 },

    // ========================================================================
    // Sorcery-speed timing errors
    // ========================================================================
    #[error(
        "Can only cast sorceries during your turn in your main phase when the stack is empty{hint}"
    )]
    NotYourTurn { hint: String },

    #[error("Can only cast sorceries during your main phase when the stack is empty{hint}")]
    NotMainPhase { hint: String },

    #[error("Can only cast sorceries when the stack is empty (during your main phase){hint}")]
    StackNotEmpty { hint: String },

    // ========================================================================
    // Targeting errors
    // ========================================================================
    #[error("Spell '{card_id}' requires a target but none was provided")]
    TargetRequired { card_id: String },

    #[error("Invalid target: {reason}")]
    InvalidTarget { reason: String },

    // ========================================================================
    // Mulligan errors
    // ========================================================================

    #[error("Cannot perform mulligan actions: game is not in the mulligan phase")]
    NotInMulliganPhase,

    #[error("Player '{player_id}' has already kept their opening hand")]
    PlayerAlreadyKept { player_id: PlayerId },

    #[error("Player '{player_id}' has not kept yet — cannot put cards on the bottom")]
    PlayerHasNotKeptYet { player_id: PlayerId },

    #[error(
        "Player '{player_id}' has no cards left to put on the bottom (put-back count is 0)"
    )]
    NoPutBackRequired { player_id: PlayerId },
}

impl GameError {
    /// Convenience constructor for `NotYourTurn` that mirrors the TypeScript
    /// `NotYourTurnError(isCreature)` overload.
    pub fn not_your_turn(is_creature: bool) -> Self {
        let hint = if is_creature {
            " (unless they have Flash)".to_owned()
        } else {
            String::new()
        };
        GameError::NotYourTurn { hint }
    }

    /// Convenience constructor for `NotMainPhase` that mirrors the TypeScript
    /// `NotMainPhaseError(isCreature)` overload.
    pub fn not_main_phase(is_creature: bool) -> Self {
        let hint = if is_creature {
            " (unless they have Flash)".to_owned()
        } else {
            String::new()
        };
        GameError::NotMainPhase { hint }
    }

    /// Convenience constructor for `StackNotEmpty` that mirrors the TypeScript
    /// `StackNotEmptyError(isCreature)` overload.
    pub fn stack_not_empty(is_creature: bool) -> Self {
        let hint = if is_creature {
            " (unless they have Flash)".to_owned()
        } else {
            String::new()
        };
        GameError::StackNotEmpty { hint }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{CardInstanceId, PlayerId};

    #[test]
    fn invalid_player_count_message() {
        let err = GameError::InvalidPlayerCount { player_count: 1 };
        assert_eq!(
            err.to_string(),
            "Game requires at least 2 players, but got 1"
        );
    }

    #[test]
    fn game_not_started_message() {
        let err = GameError::GameNotStarted;
        assert_eq!(
            err.to_string(),
            "Cannot perform actions on a game that has not been started"
        );
    }

    #[test]
    fn game_finished_message() {
        let err = GameError::GameFinished;
        assert_eq!(
            err.to_string(),
            "Cannot perform actions on a game that has finished"
        );
    }

    #[test]
    fn player_not_found_message() {
        let err = GameError::PlayerNotFound {
            player_id: PlayerId::new("player-42"),
        };
        assert_eq!(
            err.to_string(),
            "Player with id 'player-42' not found in game"
        );
    }

    #[test]
    fn card_not_found_in_hand_message() {
        let err = GameError::CardNotFoundInHand {
            card_id: CardInstanceId::new("card-1"),
            player_id: PlayerId::new("player-1"),
        };
        assert_eq!(
            err.to_string(),
            "Card 'card-1' not found in hand of player 'player-1'"
        );
    }

    #[test]
    fn cannot_block_flying_message() {
        let err = GameError::CannotBlockFlyingCreature {
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        };
        assert!(err
            .to_string()
            .contains("cannot block flying creature 'attacker-1'"));
    }

    #[test]
    fn not_your_turn_without_creature_hint() {
        let err = GameError::not_your_turn(false);
        let msg = err.to_string();
        assert!(msg.contains(
            "Can only cast sorceries during your turn in your main phase when the stack is empty"
        ));
        assert!(!msg.contains("Flash"));
    }

    #[test]
    fn not_your_turn_with_creature_hint() {
        let err = GameError::not_your_turn(true);
        assert!(err.to_string().contains("unless they have Flash"));
    }

    #[test]
    fn missing_deck_message() {
        let err = GameError::MissingDeck {
            player_ids: "player-1, player-2".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "Missing deck for player(s): player-1, player-2"
        );
    }

    #[test]
    fn invalid_player_action_message() {
        let err = GameError::InvalidPlayerAction {
            player_id: PlayerId::new("p1"),
            action: "ADVANCE_STEP".to_owned(),
        };
        assert!(err.to_string().contains("cannot perform action 'ADVANCE_STEP'"));
    }
}
