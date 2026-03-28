//! Built-in card definitions for the MVP card pool.
//!
//! All cards are read-only functions returning a `CardDefinition`.
//! Using functions rather than `static` items avoids `lazy_static` or
//! `once_cell` dependencies while still being cheap (definitions are small).
//!
//! Mirrors the TypeScript `CardCatalog` from `cards/CardCatalog.ts`.

use crate::domain::abilities::{ActivatedAbility, ActivationCost};
use crate::domain::cards::card_definition::CardDefinition;
use crate::domain::effects::Effect;
use crate::domain::enums::{CardType, ManaColor};
use crate::domain::value_objects::mana::ManaCost;

// ============================================================================
// Basic Lands
// ============================================================================

/// Return the `Forest` basic land definition.
///
/// Activated ability: {T} → Add {G}.
pub fn forest() -> CardDefinition {
    CardDefinition::new("forest", "Forest", vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
        })
}

/// Return the `Mountain` basic land definition.
///
/// Activated ability: {T} → Add {R}.
pub fn mountain() -> CardDefinition {
    CardDefinition::new("mountain", "Mountain", vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Red, amount: 1 },
        })
}

/// Return the `Plains` basic land definition.
///
/// Activated ability: {T} → Add {W}.
pub fn plains() -> CardDefinition {
    CardDefinition::new("plains", "Plains", vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::White, amount: 1 },
        })
}

/// Return the `Island` basic land definition.
///
/// Activated ability: {T} → Add {U}.
pub fn island() -> CardDefinition {
    CardDefinition::new("island", "Island", vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Blue, amount: 1 },
        })
}

/// Return the `Swamp` basic land definition.
///
/// Activated ability: {T} → Add {B}.
pub fn swamp() -> CardDefinition {
    CardDefinition::new("swamp", "Swamp", vec![CardType::Land])
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Black, amount: 1 },
        })
}

// ============================================================================
// Creatures
// ============================================================================

/// Return the `Bear` (2/2 creature) definition.
///
/// Mana cost: {1}{G} (1 generic + 1 green), matching Grizzly Bears.
pub fn bear() -> CardDefinition {
    // SAFETY: "1G" is a valid mana cost string; this cannot fail at runtime.
    let cost = ManaCost::parse("1G").expect("bear mana cost is valid");
    CardDefinition::new("bear", "Bear", vec![CardType::Creature])
        .with_power_toughness(2, 2)
        .with_mana_cost(cost)
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
    fn forest_has_green_mana_ability() {
        let f = forest();
        let ability = f.activated_ability().expect("Forest should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Green, amount: 1 });
    }

    #[test]
    fn mountain_has_red_mana_ability() {
        let m = mountain();
        let ability = m.activated_ability().expect("Mountain should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Red, amount: 1 });
    }

    #[test]
    fn plains_has_white_mana_ability() {
        let p = plains();
        let ability = p.activated_ability().expect("Plains should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::White, amount: 1 });
    }

    #[test]
    fn island_has_blue_mana_ability() {
        let i = island();
        let ability = i.activated_ability().expect("Island should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Blue, amount: 1 });
    }

    #[test]
    fn swamp_has_black_mana_ability() {
        let s = swamp();
        let ability = s.activated_ability().expect("Swamp should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Black, amount: 1 });
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
    fn bear_has_mana_cost_1g() {
        use crate::domain::value_objects::mana::ManaCost;
        let b = bear();
        let cost = b.mana_cost().expect("Bear must have a mana cost of {1}{G}");
        let expected = ManaCost::parse("1G").unwrap();
        assert_eq!(*cost, expected, "Bear mana cost should be {{1}}{{G}}");
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
