//! Card instance — a unique copy of a card definition in a game.
//!
//! Mirrors the TypeScript `CardInstance` type from `cards/CardInstance.ts`.

use serde::{Deserialize, Serialize};

use crate::domain::cards::card_definition::CardDefinition;

/// A specific, uniquely-identified copy of a card in a game.
///
/// When two players each have a Forest in their decks, both are represented
/// by separate `CardInstance` values, each with a distinct `instance_id`,
/// both sharing the same underlying `CardDefinition`.
///
/// Equality is determined solely by `instance_id`, following MTG Rule 109.1
/// (each object has its own individual identity). Two instances with the same
/// definition but different IDs are never equal.
///
/// Mirrors the TypeScript `CardInstance` type from `cards/CardInstance.ts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    /// Globally unique ID for this copy of the card in the game.
    instance_id: String,
    /// The static definition this instance is based on.
    definition: CardDefinition,
    /// The player who owns this card (not necessarily who controls it).
    owner_id: String,
}

impl CardInstance {
    /// Create a new card instance.
    pub fn new(
        instance_id: impl Into<String>,
        definition: CardDefinition,
        owner_id: impl Into<String>,
    ) -> Self {
        CardInstance {
            instance_id: instance_id.into(),
            definition,
            owner_id: owner_id.into(),
        }
    }

    /// The unique instance ID for this copy.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// The static card definition.
    pub fn definition(&self) -> &CardDefinition {
        &self.definition
    }

    /// The owning player's ID.
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }
}

impl PartialEq for CardInstance {
    fn eq(&self, other: &Self) -> bool {
        self.instance_id == other.instance_id
    }
}

impl Eq for CardInstance {}

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use crate::domain::enums::CardType;

    /// Create a minimal creature `CardInstance` for tests.
    pub fn make_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::cards::card_definition::CardDefinition;
        let def = CardDefinition::new("test-creature", "Test Creature", vec![CardType::Creature])
            .with_power_toughness(1, 1);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Create a minimal land `CardInstance` for tests.
    pub fn make_land(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::cards::card_definition::CardDefinition;
        let def = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        CardInstance::new(instance_id, def, owner_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::CardType;
    use test_helpers::{make_creature, make_land};

    #[test]
    fn instance_has_correct_id_and_owner() {
        let inst = make_creature("inst-1", "player-1");
        assert_eq!(inst.instance_id(), "inst-1");
        assert_eq!(inst.owner_id(), "player-1");
    }

    #[test]
    fn definition_is_accessible() {
        let inst = make_creature("inst-1", "player-1");
        assert!(inst.definition().is_creature());
        assert_eq!(inst.definition().power(), Some(1));
    }

    #[test]
    fn land_instance() {
        let inst = make_land("land-1", "player-1");
        assert!(inst.definition().is_land());
        assert_eq!(inst.definition().types(), &[CardType::Land]);
    }

    #[test]
    fn equality_by_instance_id_only() {
        // Two instances with different IDs are never equal, even if the same definition.
        let a = make_creature("inst-a", "player-1");
        let b = make_creature("inst-b", "player-1");
        assert_ne!(a, b);
    }

    #[test]
    fn equality_ignores_definition_and_owner() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::enums::CardType;
        // Two instances with the same ID but different owners/definitions are equal.
        let def_a = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        let def_b = CardDefinition::new("island", "Island", vec![CardType::Land]);
        let a = CardInstance::new("shared-id", def_a, "player-1");
        let b = CardInstance::new("shared-id", def_b, "player-2");
        assert_eq!(a, b);
    }

    #[test]
    fn cloned_instance_equals_original() {
        let a = make_creature("inst-1", "player-1");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
