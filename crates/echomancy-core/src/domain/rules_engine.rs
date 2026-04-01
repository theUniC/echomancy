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
