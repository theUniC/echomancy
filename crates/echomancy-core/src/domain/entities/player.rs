// NOTE: Not currently used by the Game aggregate. Kept for potential future direct-zone APIs.

//! Player entity and player state.
//!

use crate::domain::entities::battlefield::Battlefield;
use crate::domain::entities::graveyard::Graveyard;
use crate::domain::entities::hand::Hand;
use crate::domain::entities::library::Library;

const DEFAULT_LIFE_TOTAL: i32 = 20;

/// A player in the game.
///
/// The `Player` entity holds the player's identity and current life total.
/// Zone state is tracked separately in `PlayerState`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    id: String,
    name: String,
    life_total: i32,
}

impl Player {
    /// Create a new player with the default starting life total (20).
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Player {
            id: id.into(),
            name: name.into(),
            life_total: DEFAULT_LIFE_TOTAL,
        }
    }

    /// Create a new player with a custom life total.
    pub fn with_life(
        id: impl Into<String>,
        name: impl Into<String>,
        life_total: i32,
    ) -> Self {
        Player {
            id: id.into(),
            name: name.into(),
            life_total,
        }
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// The player's unique ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The player's display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The player's current life total.
    pub fn life_total(&self) -> i32 {
        self.life_total
    }

    // -------------------------------------------------------------------------
    // Mutations (return new Player)
    // -------------------------------------------------------------------------

    /// Adjust the life total by `delta` (positive = gain, negative = loss).
    ///
    /// Life total can go negative (player is still in the game until state-based
    /// actions are checked).
    pub fn adjust_life_total(&self, delta: i32) -> Player {
        Player {
            id: self.id.clone(),
            name: self.name.clone(),
            life_total: self.life_total + delta,
        }
    }
}

/// All zone state for a single player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    /// Cards in the player's hand.
    hand: Hand,
    /// Permanents the player controls on the battlefield.
    battlefield: Battlefield,
    /// The player's graveyard.
    graveyard: Graveyard,
    /// The player's library (deck).
    library: Library,
}

impl PlayerState {
    /// Create a fresh `PlayerState` with all zones empty.
    pub fn empty() -> Self {
        PlayerState {
            hand: Hand::empty(),
            battlefield: Battlefield::empty(),
            graveyard: Graveyard::empty(),
            library: Library::empty(),
        }
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// The player's hand zone.
    pub fn hand(&self) -> &Hand {
        &self.hand
    }

    /// The player's battlefield zone.
    pub fn battlefield(&self) -> &Battlefield {
        &self.battlefield
    }

    /// The player's graveyard zone.
    pub fn graveyard(&self) -> &Graveyard {
        &self.graveyard
    }

    /// The player's library (deck) zone.
    pub fn library(&self) -> &Library {
        &self.library
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_has_20_life() {
        let p = Player::new("p1", "Alice");
        assert_eq!(p.life_total(), 20);
    }

    #[test]
    fn player_with_custom_life() {
        let p = Player::with_life("p1", "Alice", 30);
        assert_eq!(p.life_total(), 30);
    }

    #[test]
    fn adjust_life_total_positive() {
        let p = Player::new("p1", "Alice");
        let p2 = p.adjust_life_total(5);
        assert_eq!(p2.life_total(), 25);
        // original unchanged
        assert_eq!(p.life_total(), 20);
    }

    #[test]
    fn adjust_life_total_negative() {
        let p = Player::new("p1", "Alice");
        let p2 = p.adjust_life_total(-7);
        assert_eq!(p2.life_total(), 13);
    }

    #[test]
    fn life_total_can_go_negative() {
        let p = Player::new("p1", "Alice");
        let p2 = p.adjust_life_total(-25);
        assert_eq!(p2.life_total(), -5);
    }

    #[test]
    fn player_id_and_name() {
        let p = Player::new("player-1", "Bob");
        assert_eq!(p.id(), "player-1");
        assert_eq!(p.name(), "Bob");
    }

    #[test]
    fn player_state_starts_empty() {
        let state = PlayerState::empty();
        assert!(state.hand().is_empty());
        assert!(state.battlefield().is_empty());
        assert!(state.graveyard().is_empty());
        assert!(state.library().is_empty());
    }
}
