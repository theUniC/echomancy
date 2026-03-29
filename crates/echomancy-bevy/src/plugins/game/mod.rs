//! GamePlugin — bridge between the echomancy-core domain and Bevy's ECS.
//!
//! Responsibilities:
//! - Hold the domain `Game` as a Bevy `Resource` (`GameState`).
//! - Mirror the snapshot and allowed actions as resources (`CurrentSnapshot`, `PlayableCards`).
//! - Register `GameActionMessage` for UI systems to send actions.
//! - Register `SnapshotChangedMessage` so UI systems know when to rebuild.
//! - Set up a 2D camera on startup.
//!
//! ## Module layout
//!
//! | Module       | Contents |
//! |--------------|----------|
//! | `mod.rs`     | Resource / type / message definitions, `CatalogRegistry` |
//! | `snapshot`   | Pure snapshot-computation helpers (`compute_*`, `humanize_error`, etc.) |
//! | `systems`    | Bevy systems (`setup_game`, `handle_game_actions`, …) and `GamePlugin` |

pub(crate) mod snapshot;
pub(crate) mod systems;

pub(crate) use systems::GamePlugin;

use bevy::prelude::*;
use echomancy_core::prelude::*;

// ============================================================================
// Resources
// ============================================================================

/// Holds the live domain `Game` aggregate.
///
/// All state mutations go through `GameActionMessage` → `handle_game_actions`.
/// UI systems read `CurrentSnapshot` instead of this directly.
#[derive(Resource)]
pub(crate) struct GameState {
    pub(crate) game: Game,
}

/// The most recent player-relative snapshot, recomputed after every mutation.
///
/// UI plugins (Phase 8.2+) read `snapshot` to rebuild rendered card state.
#[derive(Resource)]
#[allow(dead_code)]
pub(crate) struct CurrentSnapshot {
    pub(crate) snapshot: GameSnapshot,
}

/// The most recent allowed-actions result, recomputed after every mutation.
///
/// UI plugins (Phase 8.2+) read `result` to highlight playable cards.
#[derive(Resource)]
#[allow(dead_code)]
pub(crate) struct PlayableCards {
    pub(crate) result: AllowedActionsResult,
}

/// The player whose perspective drives the UI (updated whenever priority changes).
#[derive(Resource)]
pub(crate) struct ActivePlayerId {
    pub(crate) player_id: String,
}

/// A single player's identity info: their ID and display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlayerInfo {
    pub(crate) id: String,
    pub(crate) name: String,
}

/// Stores both players' IDs and display names so the HUD can label whose
/// perspective is currently shown.
#[derive(Resource)]
pub(crate) struct PlayerIds {
    pub(crate) p1: PlayerInfo,
    pub(crate) p2: PlayerInfo,
}

impl PlayerIds {
    /// Return the display name of the player with the given ID, or `"Unknown"`.
    pub(crate) fn name_for(&self, player_id: &str) -> &str {
        if self.p1.id == player_id {
            &self.p1.name
        } else if self.p2.id == player_id {
            &self.p2.name
        } else {
            "Unknown"
        }
    }
}

/// Stores the most recent domain error message to display in the HUD.
///
/// Set to `Some(msg)` when an action is rejected; cleared to `None` on the
/// next successful action. The HUD reads this resource to render a red alert.
#[derive(Resource, Default)]
pub(crate) struct ErrorMessage {
    pub(crate) message: Option<String>,
}

/// Describes a spell that is waiting for the player to choose a target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingSpell {
    /// The instance ID of the card in hand.
    pub(crate) card_instance_id: String,
    /// The definition ID of the card (e.g. `"lightning-strike"`).
    pub(crate) card_definition_id: String,
}

/// Active when the player is in target-selection mode.
///
/// Set by the hand click handler when the player clicks a spell that requires
/// a target. Cleared after a target is chosen (dispatching `CastSpell`) or
/// when the player cancels target selection.
#[derive(Resource, Default)]
pub(crate) struct TargetSelectionState {
    /// The spell waiting to be cast. `None` means target-selection mode is inactive.
    pub(crate) pending_spell: Option<PendingSpell>,
}

// ============================================================================
// Messages
// ============================================================================

/// Sent by UI systems when the local player performs a game action.
///
/// `handle_game_actions` reads this, applies it to `GameState`, and
/// recomputes the snapshot.
#[derive(Message, Clone)]
pub(crate) struct GameActionMessage(pub(crate) Action);

/// Sent after the snapshot is recomputed.
///
/// UI systems should listen for this message to trigger a full rebuild of
/// any rendered card state.
#[derive(Message)]
pub(crate) struct SnapshotChangedMessage;

// ============================================================================
// Card registry for snapshot creation
// ============================================================================

/// Simple card registry that resolves definition IDs to human-readable names.
///
/// In the MVP the catalog is a small static set. This delegates to the
/// catalog's naming convention: the definition ID is the canonical name source.
pub(crate) struct CatalogRegistry;

impl CardRegistry for CatalogRegistry {
    fn card_name(&self, definition_id: &str) -> String {
        // Map known definition IDs to display names.
        match definition_id {
            "forest" => "Forest".to_owned(),
            "mountain" => "Mountain".to_owned(),
            "plains" => "Plains".to_owned(),
            "island" => "Island".to_owned(),
            "swamp" => "Swamp".to_owned(),
            "bear" => "Bear".to_owned(),
            "elite-vanguard" => "Elite Vanguard".to_owned(),
            "goblin" => "Goblin".to_owned(),
            "giant-growth" => "Giant Growth".to_owned(),
            "lightning-strike" => "Lightning Strike".to_owned(),
            other => other.to_owned(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- PlayerIds ---------------------------------------------------------

    #[test]
    fn player_ids_name_for_returns_p1_name() {
        let ids = PlayerIds {
            p1: PlayerInfo { id: "id-1".to_owned(), name: "Alice".to_owned() },
            p2: PlayerInfo { id: "id-2".to_owned(), name: "Bob".to_owned() },
        };
        assert_eq!(ids.name_for("id-1"), "Alice");
    }

    #[test]
    fn player_ids_name_for_returns_p2_name() {
        let ids = PlayerIds {
            p1: PlayerInfo { id: "id-1".to_owned(), name: "Alice".to_owned() },
            p2: PlayerInfo { id: "id-2".to_owned(), name: "Bob".to_owned() },
        };
        assert_eq!(ids.name_for("id-2"), "Bob");
    }

    #[test]
    fn player_ids_name_for_returns_unknown_for_bad_id() {
        let ids = PlayerIds {
            p1: PlayerInfo { id: "id-1".to_owned(), name: "Alice".to_owned() },
            p2: PlayerInfo { id: "id-2".to_owned(), name: "Bob".to_owned() },
        };
        assert_eq!(ids.name_for("bad-id"), "Unknown");
    }

    // ---- CatalogRegistry --------------------------------------------------

    #[test]
    fn catalog_registry_resolves_known_cards() {
        let registry = CatalogRegistry;
        assert_eq!(registry.card_name("forest"), "Forest");
        assert_eq!(registry.card_name("mountain"), "Mountain");
        assert_eq!(registry.card_name("bear"), "Bear");
        assert_eq!(registry.card_name("elite-vanguard"), "Elite Vanguard");
        assert_eq!(registry.card_name("giant-growth"), "Giant Growth");
        assert_eq!(registry.card_name("lightning-strike"), "Lightning Strike");
    }

    #[test]
    fn catalog_registry_returns_raw_id_for_unknown_cards() {
        let registry = CatalogRegistry;
        assert_eq!(registry.card_name("some-unknown-card"), "some-unknown-card");
    }
}
