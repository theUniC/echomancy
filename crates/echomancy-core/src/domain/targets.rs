/// A target for an effect — currently only players are supported.
///
/// Mirrors the TypeScript `Target` type from `targets/Target.ts`.
/// MVP scope: only player targets exist.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    /// A player is targeted.
    Player {
        /// The targeted player's ID.
        player_id: String,
    },
}

impl Target {
    /// Convenience constructor for a player target.
    pub fn player(player_id: impl Into<String>) -> Self {
        Target::Player {
            player_id: player_id.into(),
        }
    }

    /// Returns the player ID if this target is a player target, otherwise `None`.
    pub fn player_id(&self) -> Option<&str> {
        match self {
            Target::Player { player_id } => Some(player_id.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_target_constructor() {
        let t = Target::player("player-1");
        assert_eq!(t.player_id(), Some("player-1"));
    }

    #[test]
    fn player_target_equality() {
        let a = Target::player("player-1");
        let b = Target::player("player-1");
        let c = Target::player("player-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn player_id_accessor_on_player_variant() {
        // Verify that player_id() returns the inner ID when constructed via the Player variant.
        let t = Target::Player {
            player_id: "p1".to_owned(),
        };
        assert_eq!(t.player_id(), Some("p1"));
    }
}
