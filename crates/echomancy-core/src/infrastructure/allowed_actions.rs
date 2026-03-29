//! Allowed-actions result — a plain data struct describing what a player can do right now.
//!
//! Moved here from the application layer so the Bevy binary can depend on it without
//! pulling in the old CQRS/repository machinery.

/// Describes the set of actions a player can legally take at this moment.
///
/// Computed directly from `&Game` by the Bevy plugin and used to drive UI highlights.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedActionsResult {
    /// Instance IDs of land cards in the player's hand that can be played now.
    pub playable_lands: Vec<String>,
    /// Instance IDs of untapped lands on the player's battlefield that can be
    /// tapped to produce mana right now.
    pub tappable_lands: Vec<String>,
    /// Instance IDs of non-land spells in the player's hand that can be cast now.
    ///
    /// A spell is castable when:
    /// - The player has priority.
    /// - Timing permits (sorcery-speed: main phase, active player, empty stack).
    /// - The player can pay the mana cost from their current pool.
    pub castable_spells: Vec<String>,
    /// Subset of `castable_spells`: instance IDs of spells that require the player
    /// to choose a target before the spell is dispatched.
    ///
    /// When a spell's `CardDefinition::target_requirement` is not `None`, it appears
    /// here so the UI can enter target-selection mode instead of casting immediately.
    pub spells_needing_targets: Vec<String>,
    /// Instance IDs of creatures on the active player's battlefield that can
    /// legally be declared as attackers during the `DeclareAttackers` step.
    ///
    /// Empty when not in `DeclareAttackers` step or when the player is not the
    /// active player.
    pub attackable_creatures: Vec<String>,
    /// Instance IDs of creatures on the defending player's battlefield that can
    /// legally be declared as blockers during the `DeclareBlockers` step.
    ///
    /// Each entry is the blocker's instance ID. The UI can assign them to attack
    /// any currently-attacking creature. Empty outside of `DeclareBlockers`.
    pub blockable_creatures: Vec<String>,
}
