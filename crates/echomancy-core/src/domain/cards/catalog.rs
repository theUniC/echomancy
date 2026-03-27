//! Built-in card definitions for the MVP card pool.
//!
//! All cards are read-only functions returning a `CardDefinition`.
//! Using functions rather than `static` items avoids `lazy_static` or
//! `once_cell` dependencies while still being cheap (definitions are small).
//!
//! Mirrors the TypeScript `CardCatalog` from `cards/CardCatalog.ts`.

use crate::domain::cards::card_definition::CardDefinition;
use crate::domain::enums::CardType;

// ============================================================================
// Basic Lands
// ============================================================================

/// Return the `Forest` basic land definition.
pub fn forest() -> CardDefinition {
    CardDefinition::new("forest", "Forest", vec![CardType::Land])
}

/// Return the `Mountain` basic land definition.
pub fn mountain() -> CardDefinition {
    CardDefinition::new("mountain", "Mountain", vec![CardType::Land])
}

/// Return the `Plains` basic land definition.
pub fn plains() -> CardDefinition {
    CardDefinition::new("plains", "Plains", vec![CardType::Land])
}

/// Return the `Island` basic land definition.
pub fn island() -> CardDefinition {
    CardDefinition::new("island", "Island", vec![CardType::Land])
}

/// Return the `Swamp` basic land definition.
pub fn swamp() -> CardDefinition {
    CardDefinition::new("swamp", "Swamp", vec![CardType::Land])
}

// ============================================================================
// Creatures
// ============================================================================

/// Return the `Bear` (2/2 creature) definition.
pub fn bear() -> CardDefinition {
    CardDefinition::new("bear", "Bear", vec![CardType::Creature])
        .with_power_toughness(2, 2)
}

/// Return the `Elite Vanguard` (2/1 creature) definition.
pub fn elite_vanguard() -> CardDefinition {
    CardDefinition::new("elite-vanguard", "Elite Vanguard", vec![CardType::Creature])
        .with_power_toughness(2, 1)
}

// ============================================================================
// Spells
// ============================================================================

/// Return the `Giant Growth` instant definition.
pub fn giant_growth() -> CardDefinition {
    CardDefinition::new("giant-growth", "Giant Growth", vec![CardType::Instant])
}

/// Return the `Lightning Strike` instant definition.
pub fn lightning_strike() -> CardDefinition {
    CardDefinition::new("lightning-strike", "Lightning Strike", vec![CardType::Instant])
}

/// Return the `Divination` sorcery definition.
pub fn divination() -> CardDefinition {
    CardDefinition::new("divination", "Divination", vec![CardType::Sorcery])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forest_is_land() {
        let f = forest();
        assert_eq!(f.id(), "forest");
        assert_eq!(f.name(), "Forest");
        assert!(f.is_land());
        assert!(f.mana_cost().is_none());
    }

    #[test]
    fn mountain_is_land() {
        assert!(mountain().is_land());
        assert_eq!(mountain().id(), "mountain");
    }

    #[test]
    fn bear_has_correct_stats() {
        let b = bear();
        assert!(b.is_creature());
        assert_eq!(b.power(), Some(2));
        assert_eq!(b.toughness(), Some(2));
        assert_eq!(b.id(), "bear");
    }

    #[test]
    fn elite_vanguard_has_correct_stats() {
        let ev = elite_vanguard();
        assert!(ev.is_creature());
        assert_eq!(ev.power(), Some(2));
        assert_eq!(ev.toughness(), Some(1));
        assert_eq!(ev.id(), "elite-vanguard");
    }

    #[test]
    fn giant_growth_is_instant() {
        let gg = giant_growth();
        assert!(gg.is_instant());
        assert_eq!(gg.id(), "giant-growth");
    }

    #[test]
    fn divination_is_sorcery() {
        let d = divination();
        assert!(d.types().contains(&CardType::Sorcery));
        assert_eq!(d.id(), "divination");
    }

    #[test]
    fn all_catalog_ids_are_unique() {
        let ids: Vec<&str> = vec![
            "forest",
            "mountain",
            "plains",
            "island",
            "swamp",
            "bear",
            "elite-vanguard",
            "giant-growth",
            "lightning-strike",
            "divination",
        ];
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(*id), "Duplicate catalog ID: {id}");
        }
    }
}
