use serde::{Deserialize, Serialize};

/// Describes what kind of target a spell requires.
///
/// Used on `CardDefinition` to declare targeting requirements at cast time.
/// The cast-spell handler validates that the player provided a legal target
/// matching this requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TargetRequirement {
    /// Spell requires no targets (creatures, enchantments, sorceries without "target").
    #[default]
    None,
    /// "Any target" — player or creature. Used by Lightning Strike.
    AnyTarget,
    /// Must target a creature on the battlefield.
    Creature,
}

/// A target for an effect.
///
/// Mirrors the TypeScript `Target` type from `targets/Target.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Target {
    /// A player is targeted.
    Player {
        /// The targeted player's ID.
        player_id: String,
    },
    /// A creature permanent on the battlefield is targeted.
    Creature {
        /// The targeted permanent's instance ID.
        permanent_id: String,
    },
}

impl Target {
    /// Convenience constructor for a player target.
    pub fn player(player_id: impl Into<String>) -> Self {
        Target::Player {
            player_id: player_id.into(),
        }
    }

    /// Convenience constructor for a creature target.
    pub fn creature(permanent_id: impl Into<String>) -> Self {
        Target::Creature {
            permanent_id: permanent_id.into(),
        }
    }

    /// Returns the player ID if this target is a player target, otherwise `None`.
    pub fn player_id(&self) -> Option<&str> {
        match self {
            Target::Player { player_id } => Some(player_id.as_str()),
            Target::Creature { .. } => None,
        }
    }

    /// Returns the permanent ID if this target is a creature target, otherwise `None`.
    pub fn permanent_id(&self) -> Option<&str> {
        match self {
            Target::Creature { permanent_id } => Some(permanent_id.as_str()),
            Target::Player { .. } => None,
        }
    }

    /// Returns a string ID that identifies the target object (player ID or permanent ID).
    pub fn target_id(&self) -> &str {
        match self {
            Target::Player { player_id } => player_id.as_str(),
            Target::Creature { permanent_id } => permanent_id.as_str(),
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

    #[test]
    fn creature_target_constructor() {
        let t = Target::creature("perm-1");
        assert_eq!(t.permanent_id(), Some("perm-1"));
        assert_eq!(t.player_id(), None);
    }

    #[test]
    fn creature_target_equality() {
        let a = Target::creature("perm-1");
        let b = Target::creature("perm-1");
        let c = Target::creature("perm-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn player_permanent_id_returns_none() {
        let t = Target::player("p1");
        assert_eq!(t.permanent_id(), None);
    }

    #[test]
    fn target_id_for_player() {
        let t = Target::player("p1");
        assert_eq!(t.target_id(), "p1");
    }

    #[test]
    fn target_id_for_creature() {
        let t = Target::creature("perm-42");
        assert_eq!(t.target_id(), "perm-42");
    }

    #[test]
    fn target_requirement_default_is_none() {
        assert_eq!(TargetRequirement::default(), TargetRequirement::None);
    }

    #[test]
    fn target_requirement_variants_are_distinct() {
        assert_ne!(TargetRequirement::None, TargetRequirement::AnyTarget);
        assert_ne!(TargetRequirement::None, TargetRequirement::Creature);
        assert_ne!(TargetRequirement::AnyTarget, TargetRequirement::Creature);
    }
}
