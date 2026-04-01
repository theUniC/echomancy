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
use crate::domain::enums::{CardType, ManaColor, StaticAbility};
use crate::domain::targets::TargetRequirement;
use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};
use crate::domain::value_objects::mana::ManaCost;

// ============================================================================
// Basic Lands
// ============================================================================

/// Return the `Forest` basic land definition.
///
/// Activated ability: {T} → Add {G}.
pub fn forest() -> CardDefinition {
    CardDefinition::new("forest", "Forest", vec![CardType::Land])
        .with_subtype("Forest")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
        })
        .with_oracle_text("{T}: Add {G}.")
}

/// Return the `Mountain` basic land definition.
///
/// Activated ability: {T} → Add {R}.
pub fn mountain() -> CardDefinition {
    CardDefinition::new("mountain", "Mountain", vec![CardType::Land])
        .with_subtype("Mountain")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Red, amount: 1 },
        })
        .with_oracle_text("{T}: Add {R}.")
}

/// Return the `Plains` basic land definition.
///
/// Activated ability: {T} → Add {W}.
pub fn plains() -> CardDefinition {
    CardDefinition::new("plains", "Plains", vec![CardType::Land])
        .with_subtype("Plains")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::White, amount: 1 },
        })
        .with_oracle_text("{T}: Add {W}.")
}

/// Return the `Island` basic land definition.
///
/// Activated ability: {T} → Add {U}.
pub fn island() -> CardDefinition {
    CardDefinition::new("island", "Island", vec![CardType::Land])
        .with_subtype("Island")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Blue, amount: 1 },
        })
        .with_oracle_text("{T}: Add {U}.")
}

/// Return the `Swamp` basic land definition.
///
/// Activated ability: {T} → Add {B}.
pub fn swamp() -> CardDefinition {
    CardDefinition::new("swamp", "Swamp", vec![CardType::Land])
        .with_subtype("Swamp")
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Black, amount: 1 },
        })
        .with_oracle_text("{T}: Add {B}.")
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
        .with_subtype("Bear")
        .with_power_toughness(2, 2)
        .with_mana_cost(cost)
    // Bears have no rules text.
}

/// Return the `Elite Vanguard` (2/1 creature) definition.
///
/// Mana cost: {W} (1 white), matching the original card.
pub fn elite_vanguard() -> CardDefinition {
    let cost = ManaCost::parse("W").expect("elite vanguard mana cost is valid");
    CardDefinition::new("elite-vanguard", "Elite Vanguard", vec![CardType::Creature])
        .with_subtype("Human")
        .with_subtype("Soldier")
        .with_power_toughness(2, 1)
        .with_mana_cost(cost)
}

/// Return the `Goblin` (1/1 creature) definition.
///
/// Mana cost: {R} (1 red). A simple red creature for the red starter deck.
pub fn goblin() -> CardDefinition {
    let cost = ManaCost::parse("R").expect("goblin mana cost is valid");
    CardDefinition::new("goblin", "Goblin", vec![CardType::Creature])
        .with_subtype("Goblin")
        .with_power_toughness(1, 1)
        .with_mana_cost(cost)
    // Goblins have no rules text.
}

// ============================================================================
// Spells
// ============================================================================

/// Return the `Cancel` instant definition (simplified counterspell).
///
/// Mana cost: {1} (1 generic — simplified from {1}{U}{U} to be deck-agnostic).
/// Targets: a spell on the stack.
/// Effect: Counter target spell.
pub fn cancel() -> CardDefinition {
    let cost = ManaCost::parse("1").expect("cancel mana cost is valid");
    CardDefinition::new("cancel", "Cancel", vec![CardType::Instant])
        .with_mana_cost(cost)
        .with_target_requirement(TargetRequirement::Spell)
        .with_oracle_text("Counter target spell.")
}

/// Return the `Giant Growth` instant definition.
///
/// Mana cost: {G} (1 green).
pub fn giant_growth() -> CardDefinition {
    let cost = ManaCost::parse("G").expect("giant growth mana cost is valid");
    CardDefinition::new("giant-growth", "Giant Growth", vec![CardType::Instant])
        .with_mana_cost(cost)
        .with_target_requirement(TargetRequirement::Creature)
        .with_oracle_text("Target creature gets +3/+3 until end of turn.")
}

/// Return the `Lightning Strike` instant definition.
///
/// Mana cost: {1}{R} (1 generic + 1 red).
/// Targets: any target (player or creature) — CR 115.6.
pub fn lightning_strike() -> CardDefinition {
    let cost = ManaCost::parse("1R").expect("lightning strike mana cost is valid");
    CardDefinition::new("lightning-strike", "Lightning Strike", vec![CardType::Instant])
        .with_mana_cost(cost)
        .with_target_requirement(TargetRequirement::AnyTarget)
        .with_oracle_text("Lightning Strike deals 3 damage to any target.")
}

// ============================================================================
// Artifacts
// ============================================================================

/// Return the `Sol Ring` artifact definition.
///
/// Mana cost: {1} (1 generic).
/// Activated ability: {T} → Add {C}{C}.
///
/// Sol Ring is a mana ability (CR 605) — it resolves immediately without
/// using the stack, and the activating player retains priority.
pub fn sol_ring() -> CardDefinition {
    let cost = ManaCost::parse("1").expect("sol ring mana cost is valid");
    CardDefinition::new("sol-ring", "Sol Ring", vec![CardType::Artifact])
        .with_mana_cost(cost)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Colorless, amount: 2 },
        })
        .with_oracle_text("{T}: Add {C}{C}.")
}

// ============================================================================
// Enchantments
// ============================================================================

/// Return the `Wild Bounty` enchantment definition.
///
/// Mana cost: {1}{G} (1 generic + 1 green).
/// Effect: When Wild Bounty enters the battlefield, draw a card.
///
/// This is a non-aura enchantment. It resolves to the battlefield and stays
/// there (unlike instants/sorceries which go to the graveyard). The ETB draw
/// is modelled as a Rust `Trigger` (CR 603.3) that places an `AbilityOnStack`
/// item on the stack. When the ability resolves, the CLIPS rule
/// `wild-bounty-etb-draw` fires on `TRIGGERED_ABILITY_FIRES`.
pub fn wild_bounty() -> CardDefinition {
    let cost = ManaCost::parse("1G").expect("wild bounty mana cost is valid");
    let etb_trigger = Trigger::new(
        TriggerEventType::ZoneChanged,
        TriggerCondition::SourceEntersBattlefield,
        Effect::DrawCards { amount: 1 },
    );
    CardDefinition::new("wild-bounty", "Wild Bounty", vec![CardType::Enchantment])
        .with_mana_cost(cost)
        .with_oracle_text("When Wild Bounty enters the battlefield, draw a card.")
        .with_trigger(etb_trigger)
}

/// Return the `Divination` sorcery definition.
///
/// Mana cost: {2}{U} (2 generic + 1 blue).
pub fn divination() -> CardDefinition {
    let cost = ManaCost::parse("2U").expect("divination mana cost is valid");
    CardDefinition::new("divination", "Divination", vec![CardType::Sorcery])
        .with_mana_cost(cost)
        .with_oracle_text("Draw two cards.")
}

// ============================================================================
// Showcase cards — testing keyword abilities
// ============================================================================

/// Return the `Oakshield Troll` creature definition.
///
/// Mana cost: {1}{G} (1 generic + 1 green).
/// A 3/3 Troll with Hexproof — can't be targeted by opponents' spells
/// or abilities (CR 702.11). Showcases the K4 Hexproof implementation.
pub fn oakshield_troll() -> CardDefinition {
    let cost = ManaCost::parse("1G").expect("oakshield troll mana cost is valid");
    CardDefinition::new("oakshield-troll", "Oakshield Troll", vec![CardType::Creature])
        .with_subtype("Troll")
        .with_power_toughness(3, 3)
        .with_mana_cost(cost)
        .with_static_ability(StaticAbility::Hexproof)
        .with_oracle_text("Hexproof (This creature can't be the target of spells or abilities your opponents control.)")
}

/// Return the `Ancient Guardian` creature definition.
///
/// Mana cost: {2}{G} (2 generic + 1 green).
/// A 4/5 Elemental with Indestructible — can't be destroyed by damage
/// or effects (CR 702.12). Showcases the K3 Indestructible implementation.
pub fn ancient_guardian() -> CardDefinition {
    let cost = ManaCost::parse("2G").expect("ancient guardian mana cost is valid");
    CardDefinition::new("ancient-guardian", "Ancient Guardian", vec![CardType::Creature])
        .with_subtype("Elemental")
        .with_power_toughness(4, 5)
        .with_mana_cost(cost)
        .with_static_ability(StaticAbility::Indestructible)
        .with_oracle_text("Indestructible (Damage and effects that say \"destroy\" don't destroy this creature.)")
}

/// Return the `Ironbark Wall` creature definition.
///
/// Mana cost: {G} (1 green).
/// A 0/4 Plant Wall with CannotAttack — this creature can't attack
/// (CR 508.1d). Showcases the K9 CannotAttack implementation.
pub fn ironbark_wall() -> CardDefinition {
    let cost = ManaCost::parse("G").expect("ironbark wall mana cost is valid");
    CardDefinition::new("ironbark-wall", "Ironbark Wall", vec![CardType::Creature])
        .with_subtype("Plant")
        .with_subtype("Wall")
        .with_power_toughness(0, 4)
        .with_mana_cost(cost)
        .with_static_ability(StaticAbility::CannotAttack)
        .with_oracle_text("This creature can't attack.")
}

/// Return the `Thalia, Forest Keeper` legendary creature definition.
///
/// Mana cost: {G} (1 green).
/// A 2/2 Legendary Human Druid with First Strike. Showcases the R3
/// Legendary supertype implementation (CR 205.4, CR 704.5j).
pub fn thalia_forest_keeper() -> CardDefinition {
    let cost = ManaCost::parse("G").expect("thalia forest keeper mana cost is valid");
    CardDefinition::new("thalia-forest-keeper", "Thalia, Forest Keeper", vec![CardType::Creature])
        .with_legendary()
        .with_subtype("Human")
        .with_subtype("Druid")
        .with_power_toughness(2, 2)
        .with_mana_cost(cost)
        .with_static_ability(StaticAbility::FirstStrike)
        .with_oracle_text("Legendary. First strike (This creature deals combat damage before creatures without first strike.)")
}

/// Return the `Thornwood Tapland` land definition.
///
/// No mana cost (land).
/// Enters the battlefield tapped (CR 614.12). Activated ability: {T} → Add {G}.
/// Showcases the K8 EntersTapped implementation.
pub fn thornwood_tapland() -> CardDefinition {
    CardDefinition::new("thornwood-tapland", "Thornwood Tapland", vec![CardType::Land])
        .with_static_ability(StaticAbility::EntersTapped)
        .with_activated_ability(ActivatedAbility {
            cost: ActivationCost::Tap,
            effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
        })
        .with_oracle_text("Thornwood Tapland enters the battlefield tapped.\n{T}: Add {G}.")
}

/// Return the `Reckless Berserker` creature definition.
///
/// Mana cost: {R} (1 red).
/// A 2/1 Berserker with Menace and MustAttack — must attack each combat
/// if able (CR 508.1d) and can only be blocked by two or more creatures
/// (CR 702.110). Showcases the K2 Menace and K10 MustAttack implementations.
pub fn reckless_berserker() -> CardDefinition {
    let cost = ManaCost::parse("R").expect("reckless berserker mana cost is valid");
    CardDefinition::new("reckless-berserker", "Reckless Berserker", vec![CardType::Creature])
        .with_subtype("Berserker")
        .with_power_toughness(2, 1)
        .with_mana_cost(cost)
        .with_static_ability(StaticAbility::Menace)
        .with_static_ability(StaticAbility::MustAttack)
        .with_oracle_text("Menace (This creature can't be blocked except by two or more creatures.)\nThis creature attacks each combat if able.")
}

/// Return the `Frozen Sentinel` creature definition.
///
/// Mana cost: {1}{R} (1 generic + 1 red).
/// A 3/3 Golem that doesn't untap during its controller's untap step
/// (CR 302.6). Once tapped — e.g. to attack — it stays tapped permanently.
/// Showcases the K7 DoesNotUntap implementation.
pub fn frozen_sentinel() -> CardDefinition {
    let cost = ManaCost::parse("1R").expect("frozen sentinel mana cost is valid");
    CardDefinition::new("frozen-sentinel", "Frozen Sentinel", vec![CardType::Creature])
        .with_subtype("Golem")
        .with_power_toughness(3, 3)
        .with_mana_cost(cost)
        .with_static_ability(StaticAbility::Haste)
        .with_static_ability(StaticAbility::DoesNotUntap)
        .with_oracle_text("Haste.\nFrozen Sentinel doesn't untap during your untap step.")
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
        let ability = f.first_activated_ability().expect("Forest should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Green, amount: 1 });
    }

    #[test]
    fn mountain_has_red_mana_ability() {
        let m = mountain();
        let ability = m.first_activated_ability().expect("Mountain should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Red, amount: 1 });
    }

    #[test]
    fn plains_has_white_mana_ability() {
        let p = plains();
        let ability = p.first_activated_ability().expect("Plains should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::White, amount: 1 });
    }

    #[test]
    fn island_has_blue_mana_ability() {
        let i = island();
        let ability = i.first_activated_ability().expect("Island should have an activated ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Blue, amount: 1 });
    }

    #[test]
    fn swamp_has_black_mana_ability() {
        let s = swamp();
        let ability = s.first_activated_ability().expect("Swamp should have an activated ability");
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
            "sol-ring",
            "wild-bounty",
            // Showcase cards
            "oakshield-troll",
            "ancient-guardian",
            "ironbark-wall",
            "thalia-forest-keeper",
            "thornwood-tapland",
            "reckless-berserker",
            "frozen-sentinel",
        ];
        let mut seen = std::collections::HashSet::new();
        for id in &ids {
            assert!(seen.insert(*id), "Duplicate catalog ID: {id}");
        }
    }

    // =========================================================================
    // Showcase cards — K4 Hexproof
    // =========================================================================

    #[test]
    fn oakshield_troll_is_creature_with_hexproof() {
        let t = oakshield_troll();
        assert!(t.is_creature());
        assert_eq!(t.id(), "oakshield-troll");
        assert_eq!(t.name(), "Oakshield Troll");
        assert_eq!(t.power(), Some(3));
        assert_eq!(t.toughness(), Some(3));
        assert!(t.has_static_ability(StaticAbility::Hexproof), "Oakshield Troll must have Hexproof");
        assert!(t.mana_cost().is_some());
        assert!(t.oracle_text().is_some());
    }

    // =========================================================================
    // Showcase cards — K3 Indestructible
    // =========================================================================

    #[test]
    fn ancient_guardian_is_creature_with_indestructible() {
        let g = ancient_guardian();
        assert!(g.is_creature());
        assert_eq!(g.id(), "ancient-guardian");
        assert_eq!(g.name(), "Ancient Guardian");
        assert_eq!(g.power(), Some(4));
        assert_eq!(g.toughness(), Some(5));
        assert!(
            g.has_static_ability(StaticAbility::Indestructible),
            "Ancient Guardian must have Indestructible"
        );
        assert!(g.oracle_text().is_some());
    }

    // =========================================================================
    // Showcase cards — K9 CannotAttack
    // =========================================================================

    #[test]
    fn ironbark_wall_is_creature_with_cannot_attack() {
        let w = ironbark_wall();
        assert!(w.is_creature());
        assert_eq!(w.id(), "ironbark-wall");
        assert_eq!(w.name(), "Ironbark Wall");
        assert_eq!(w.power(), Some(0));
        assert_eq!(w.toughness(), Some(4));
        assert!(
            w.has_static_ability(StaticAbility::CannotAttack),
            "Ironbark Wall must have CannotAttack"
        );
        assert!(w.oracle_text().is_some());
    }

    // =========================================================================
    // Showcase cards — R3 Legendary
    // =========================================================================

    #[test]
    fn thalia_forest_keeper_is_legendary_creature() {
        let t = thalia_forest_keeper();
        assert!(t.is_creature());
        assert!(t.is_legendary(), "Thalia, Forest Keeper must be Legendary");
        assert_eq!(t.id(), "thalia-forest-keeper");
        assert_eq!(t.name(), "Thalia, Forest Keeper");
        assert_eq!(t.power(), Some(2));
        assert_eq!(t.toughness(), Some(2));
        assert!(
            t.has_static_ability(StaticAbility::FirstStrike),
            "Thalia, Forest Keeper must have First Strike"
        );
        assert!(t.oracle_text().is_some());
    }

    // =========================================================================
    // Showcase cards — K8 EntersTapped
    // =========================================================================

    #[test]
    fn thornwood_tapland_is_land_with_enters_tapped() {
        let l = thornwood_tapland();
        assert!(l.is_land());
        assert_eq!(l.id(), "thornwood-tapland");
        assert_eq!(l.name(), "Thornwood Tapland");
        assert!(
            l.has_static_ability(StaticAbility::EntersTapped),
            "Thornwood Tapland must have EntersTapped"
        );
        // Should produce green mana
        let ability = l.first_activated_ability().expect("Thornwood Tapland must have a mana ability");
        assert_eq!(ability.effect, Effect::AddMana { color: ManaColor::Green, amount: 1 });
        assert!(l.oracle_text().is_some());
    }

    // =========================================================================
    // Showcase cards — K2 Menace + K10 MustAttack
    // =========================================================================

    #[test]
    fn reckless_berserker_has_menace_and_must_attack() {
        let b = reckless_berserker();
        assert!(b.is_creature());
        assert_eq!(b.id(), "reckless-berserker");
        assert_eq!(b.name(), "Reckless Berserker");
        assert_eq!(b.power(), Some(2));
        assert_eq!(b.toughness(), Some(1));
        assert!(b.has_static_ability(StaticAbility::Menace), "Reckless Berserker must have Menace");
        assert!(
            b.has_static_ability(StaticAbility::MustAttack),
            "Reckless Berserker must have MustAttack"
        );
        assert!(b.oracle_text().is_some());
    }

    // =========================================================================
    // Showcase cards — K7 DoesNotUntap
    // =========================================================================

    #[test]
    fn frozen_sentinel_does_not_untap() {
        let s = frozen_sentinel();
        assert!(s.is_creature());
        assert_eq!(s.id(), "frozen-sentinel");
        assert_eq!(s.name(), "Frozen Sentinel");
        assert_eq!(s.power(), Some(3));
        assert_eq!(s.toughness(), Some(3));
        assert!(
            s.has_static_ability(StaticAbility::DoesNotUntap),
            "Frozen Sentinel must have DoesNotUntap"
        );
        assert!(s.oracle_text().is_some());
    }

    // =========================================================================
    // Sol Ring (Artifact) — P3
    // =========================================================================

    #[test]
    fn sol_ring_is_artifact() {
        let sr = sol_ring();
        assert!(sr.is_artifact(), "Sol Ring should be an artifact");
        assert!(!sr.is_creature(), "Sol Ring should not be a creature");
        assert!(!sr.is_land(), "Sol Ring should not be a land");
    }

    #[test]
    fn sol_ring_has_id_and_name() {
        let sr = sol_ring();
        assert_eq!(sr.id(), "sol-ring");
        assert_eq!(sr.name(), "Sol Ring");
    }

    #[test]
    fn sol_ring_has_cost_1() {
        let sr = sol_ring();
        let cost = sr.mana_cost().expect("Sol Ring must have a mana cost");
        let expected = ManaCost::parse("1").unwrap();
        assert_eq!(*cost, expected, "Sol Ring mana cost should be {{1}}");
    }

    #[test]
    fn sol_ring_has_tap_for_2_colorless_ability() {
        let sr = sol_ring();
        let ability = sr
            .first_activated_ability()
            .expect("Sol Ring should have a tap mana ability");
        assert_eq!(
            ability.effect,
            Effect::AddMana { color: ManaColor::Colorless, amount: 2 },
            "Sol Ring should add 2 colorless mana"
        );
    }

    #[test]
    fn sol_ring_has_oracle_text() {
        let sr = sol_ring();
        assert_eq!(sr.oracle_text(), Some("{T}: Add {C}{C}."));
    }

    // =========================================================================
    // Arcane Sanctum (Enchantment) — P3
    // =========================================================================

    #[test]
    fn wild_bounty_is_enchantment() {
        let ae = wild_bounty();
        assert!(ae.is_enchantment(), "Arcane Sanctum should be an enchantment");
        assert!(!ae.is_creature(), "Arcane Sanctum should not be a creature");
        assert!(!ae.is_land(), "Arcane Sanctum should not be a land");
    }

    #[test]
    fn wild_bounty_has_id_and_name() {
        let ae = wild_bounty();
        assert_eq!(ae.id(), "wild-bounty");
        assert_eq!(ae.name(), "Wild Bounty");
    }

    #[test]
    fn wild_bounty_has_cost_1g() {
        let wb = wild_bounty();
        let cost = wb.mana_cost().expect("Wild Bounty must have a mana cost");
        let expected = ManaCost::parse("1G").unwrap();
        assert_eq!(*cost, expected, "Wild Bounty mana cost should be {{1}}{{G}}");
    }

    #[test]
    fn wild_bounty_has_oracle_text() {
        let ae = wild_bounty();
        assert!(ae.oracle_text().is_some(), "Wild Bounty should have oracle text");
    }
}
