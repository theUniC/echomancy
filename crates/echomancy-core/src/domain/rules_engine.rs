//! Domain trait for the CLIPS rules engine integration.
//!
//! The `RulesEngine` trait is the technology-agnostic interface between the
//! game engine and the card-specific rules evaluation system. The domain
//! declares what it needs; the infrastructure provides the CLIPS implementation.
//!
//! # Design
//!
//! - The trait lives in the domain layer — it has zero knowledge of CLIPS.
//! - `ClipsEngine` in the infrastructure layer implements this trait.
//! - If CLIPS is ever replaced, only the infrastructure changes.
//!
//! # Cycle
//!
//! 1. Rust serializes `&Game` state into facts and calls `evaluate()`.
//! 2. CLIPS rules fire and produce `RulesAction` proposals.
//! 3. `RulesResult` is returned to the caller.
//! 4. The caller validates each `RulesAction` and applies valid ones to `&mut Game`.
//!
//! If CLIPS needs player input, `RulesResult::awaiting_input` is `Some(InputRequest)`.
//! The caller prompts the player, gets a `PlayerChoice`, then calls `resume_after_choice()`.

// M3: trait and types are wired to resolve_spell() in domain/game/internals.rs.

use thiserror::Error;

use crate::domain::events::GameEvent;
use crate::domain::game::Game;

// ============================================================================
// Output: actions that CLIPS proposes (CLIPS → Rust)
// ============================================================================

/// An action proposed by CLIPS rules after evaluating a game event.
///
/// These correspond to the `action-*` deftemplates in the CLIPS schema.
/// CLIPS proposes actions; Rust validates and applies them.
/// This is distinct from `domain::actions::Action` which represents player
/// input actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesAction {
    /// Deal damage from a source to a target (player or permanent ID).
    DealDamage {
        source: String,
        target: String,
        amount: u32,
    },
    /// A player draws one or more cards.
    DrawCards { player: String, amount: u32 },
    /// Destroy a permanent.
    DestroyPermanent { target: String },
    /// A player gains life.
    GainLife { player: String, amount: u32 },
    /// A player loses life.
    LoseLife { player: String, amount: u32 },
    /// Move a card between zones.
    MoveZone {
        card_id: String,
        from_zone: String,
        to_zone: String,
    },
    /// Add mana to a player's pool.
    AddMana {
        player: String,
        color: String,
        amount: u32,
    },
    /// Tap a permanent.
    Tap { permanent_id: String },
    /// Untap a permanent.
    Untap { permanent_id: String },
    /// Add a counter to a permanent.
    AddCounter {
        permanent_id: String,
        counter_type: String,
        amount: u32,
    },
    /// Create a token on the battlefield.
    CreateToken {
        controller: String,
        name: String,
        power: i32,
        toughness: i32,
        types: Vec<String>,
        keywords: Vec<String>,
    },
    /// Exile a permanent (move it from the battlefield to exile).
    Exile { target: String },
    /// Counter a spell on the stack (remove it and put in graveyard).
    CounterSpell { target: String },
    /// Return a permanent from the battlefield to its owner's hand (bounce).
    ReturnToHand { target: String },
    /// Scry N — look at top N cards, put any on top (in order) and rest on bottom.
    /// For MVP, auto-scry keeps the cards in their current order (top stays top).
    Scry { player: String, amount: u32 },
    /// Surveil N — look at top N cards, put chosen ones in graveyard (CR 701.37).
    /// For MVP, auto-surveil sends all to graveyard.
    Surveil { player: String, amount: u32 },
    /// Mill N — move top N cards from library to graveyard (CR 701.13).
    Mill { player: String, amount: u32 },
    /// Discard N cards from a player's hand (CR 701.8). MVP: auto-discard last N.
    Discard { player: String, amount: u32 },
    /// Apply a temporary power/toughness modification to a creature.
    ModifyPowerToughness {
        /// Instance ID of the permanent to modify.
        target: String,
        /// Power modifier (e.g. `+3` for Giant Growth).
        power: i32,
        /// Toughness modifier (e.g. `+3` for Giant Growth).
        toughness: i32,
        /// Duration string, e.g. `"until_end_of_turn"`.
        duration: String,
        /// Instance ID of the spell or ability that created this effect.
        source: String,
    },
    /// Create a Treasure token for the given controller (CR 111.10b).
    CreateTreasure { controller: String },
    /// Investigate — create a Clue token for the given controller (CR 701.34).
    Investigate { controller: String },
    /// Create a Food token for the given controller (CR 111.10c).
    CreateFood { controller: String },
    /// Fight mechanic — each creature deals damage equal to its power to the
    /// other creature (CR 701.14).
    Fight { creature_a: String, creature_b: String },
    /// Bolster N — put N +1/+1 counters on the creature the player controls
    /// with the least toughness (CR 701.39).
    Bolster { player: String, amount: u32 },
    /// Adapt N — if this creature has no +1/+1 counters on it, put N +1/+1
    /// counters on it (CR 701.46).
    Adapt { target: String, amount: u32 },
    /// Set a creature's power and toughness to specific values (Layer 7b).
    ///
    /// Used by effects like Turn to Frog ("becomes a 1/1").
    SetPowerToughness {
        /// Instance ID of the permanent to modify.
        target: String,
        /// The new base power value.
        power: i32,
        /// The new base toughness value.
        toughness: i32,
        /// Duration string, e.g. `"until-end-of-turn"`.
        duration: String,
        /// Instance ID of the spell or ability that created this effect.
        source: String,
    },
    /// Switch a creature's power and toughness (Layer 7d).
    ///
    /// Used by effects like Twisted Image.
    SwitchPowerToughness {
        /// Instance ID of the permanent whose P/T is switched.
        target: String,
        /// Duration string, e.g. `"until-end-of-turn"`.
        duration: String,
        /// Instance ID of the spell or ability that created this effect.
        source: String,
    },
    /// Remove all keyword abilities from a creature (Layer 6).
    ///
    /// Used by effects like Turn to Frog.
    RemoveAllAbilities {
        /// Instance ID of the permanent to strip abilities from.
        target: String,
        /// Duration string, e.g. `"until-end-of-turn"`.
        duration: String,
        /// Instance ID of the spell or ability that created this effect.
        source: String,
    },
    /// Register a damage prevention shield on a target (R11/R12).
    ///
    /// The shield prevents up to `amount` damage to the target. Once the budget
    /// is depleted, the shield is removed from the registry.
    ///
    /// Duration: `"next-occurrence"` for a single use, `"until-end-of-turn"` for
    /// a turn-long prevention effect.
    ///
    /// Scope:
    /// - `"targeted"` (default): filter on the specific `target` permanent or player.
    /// - `"all-combat"`: global combat-damage prevention (Fog, CR 615.7a);
    ///   the `target` field is ignored, `AllCombatDamage` filter is used instead.
    RegisterPreventionShield {
        /// Instance ID of the target creature or player ID.
        /// Ignored when `scope` is `"all-combat"`.
        target: String,
        /// Maximum amount of damage to prevent. `0` means prevent all.
        amount: u32,
        /// Duration string: `"next-occurrence"` or `"until-end-of-turn"`.
        duration: String,
        /// Instance ID of the spell or ability that created this shield.
        source: String,
        /// Scope string: `"targeted"` or `"all-combat"`.
        scope: String,
    },
    /// Register a regeneration shield on a creature (R11, CR 701.15).
    ///
    /// If the creature would be destroyed, it is instead tapped, all damage is
    /// removed from it, and it is removed from combat. The shield is then consumed.
    ///
    /// Duration: always `"next-occurrence"` (a regeneration shield applies once).
    RegisterRegenerationShield {
        /// Instance ID of the creature to protect.
        target: String,
        /// Instance ID of the spell or ability that created this shield.
        source: String,
    },
}

// ============================================================================
// Input request: CLIPS asking for player decision
// ============================================================================

/// A request for player input, produced when a CLIPS rule calls `(halt)`.
///
/// After the engine returns this, the caller must prompt the player, construct
/// a `PlayerChoice`, and call `resume_after_choice()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputRequest {
    /// The type of choice needed (e.g. "sacrifice", "target", "mode").
    pub(crate) input_type: String,
    /// The player who must make the choice.
    pub(crate) player: String,
    /// Human-readable prompt for the UI.
    pub(crate) prompt: String,
}

impl InputRequest {
    /// Construct a new `InputRequest`.
    pub fn new(
        input_type: impl Into<String>,
        player: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            input_type: input_type.into(),
            player: player.into(),
            prompt: prompt.into(),
        }
    }

    /// The type of choice needed (e.g. `"sacrifice"`, `"target"`, `"mode"`).
    pub fn input_type(&self) -> &str {
        &self.input_type
    }

    /// The player who must make the choice.
    pub fn player(&self) -> &str {
        &self.player
    }

    /// Human-readable prompt intended for display in the UI.
    pub fn prompt(&self) -> &str {
        &self.prompt
    }
}

/// A player's response to an `InputRequest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerChoice {
    /// The type of choice this resolves (must match `InputRequest::input_type`).
    pub(crate) input_type: String,
    /// The player making the choice.
    pub(crate) player: String,
    /// The chosen value (e.g. a card instance ID, a mode number).
    pub(crate) chosen: String,
}

impl PlayerChoice {
    /// Construct a new `PlayerChoice`.
    pub fn new(
        input_type: impl Into<String>,
        player: impl Into<String>,
        chosen: impl Into<String>,
    ) -> Self {
        Self {
            input_type: input_type.into(),
            player: player.into(),
            chosen: chosen.into(),
        }
    }

    /// The type of choice this resolves (must match the originating `InputRequest::input_type`).
    pub fn input_type(&self) -> &str {
        &self.input_type
    }

    /// The player making the choice.
    pub fn player(&self) -> &str {
        &self.player
    }

    /// The chosen value (e.g. a card instance ID, a mode number).
    pub fn chosen(&self) -> &str {
        &self.chosen
    }
}

// ============================================================================
// Result
// ============================================================================

/// The result of a rules engine evaluation cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesResult {
    /// Actions proposed by CLIPS rules, sorted by priority slot.
    pub actions: Vec<RulesAction>,
    /// Set when a CLIPS rule called `(halt)` and needs player input.
    pub awaiting_input: Option<InputRequest>,
    /// Number of rules fired during `Run(N)`.
    pub rules_fired: i64,
    /// Non-fatal warnings captured from the CLIPS router.
    pub warnings: Vec<String>,
}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur during rules engine evaluation.
#[derive(Debug, Error)]
pub enum RulesError {
    /// CLIPS environment could not be created.
    #[error("failed to create CLIPS environment")]
    EnvironmentCreationFailed,

    /// A construct (deftemplate, defrule, etc.) failed to load.
    #[error("failed to load CLIPS construct: {0}")]
    LoadFailed(String),

    /// A fact string could not be asserted.
    #[error("failed to assert CLIPS fact: {0}")]
    AssertFailed(String),

    /// Execution hit the maximum rule limit — possible infinite loop.
    #[error("CLIPS execution hit max rules limit ({limit}), possible infinite loop")]
    MaxRulesExceeded { limit: i64 },

    /// An action fact produced by CLIPS contained invalid data.
    #[error("invalid action fact: {0}")]
    InvalidAction(String),

    /// A resume_after_choice call was made with no pending input request.
    #[error("resume_after_choice called but no input is pending")]
    NoInputPending,

    /// Internal CLIPS error with a message captured from the router.
    #[error("CLIPS internal error: {0}")]
    Internal(String),
}

// ============================================================================
// Trait
// ============================================================================

/// Technology-agnostic interface for card-specific rules evaluation.
///
/// Implemented by `ClipsEngine` in the infrastructure layer.
/// The domain knows only this trait — never the CLIPS FFI.
pub trait RulesEngine: Send + Sync {
    /// Evaluate a game event against the current game state.
    ///
    /// The engine serializes `state` into CLIPS facts, asserts `event`, runs
    /// the rules, and returns the proposed actions.
    ///
    /// If a rule calls `(halt)` to request player input, the result will have
    /// `awaiting_input: Some(...)`. The caller must then call
    /// `resume_after_choice()` to continue execution.
    fn evaluate(
        &mut self,
        state: &Game,
        event: &GameEvent,
    ) -> Result<RulesResult, RulesError>;

    /// Resume rule execution after the player has made a choice.
    ///
    /// # Errors
    ///
    /// - `RulesError::NoInputPending` if called when no input was requested.
    fn resume_after_choice(
        &mut self,
        choice: &PlayerChoice,
    ) -> Result<RulesResult, RulesError>;
}
