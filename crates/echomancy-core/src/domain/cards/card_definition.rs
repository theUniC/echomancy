//! Card definition — the template/blueprint for a card.
//!
//! Mirrors the TypeScript `CardDefinition` type from `cards/CardDefinition.ts`.

use serde::{Deserialize, Serialize};

use crate::domain::abilities::ActivatedAbility;
use crate::domain::enums::{CardType, ManaColor, StaticAbility};
use crate::domain::targets::TargetRequirement;
use crate::domain::triggers::Trigger;
use crate::domain::value_objects::mana::ManaCost;

/// Static immutable description of a card.
///
/// `CardDefinition` is value-object style: it describes _what_ a card is,
/// not a specific copy of it in a game. Two cards with the same `id` are
/// the same card.
///
/// Mirrors the TypeScript `CardDefinition` type from `cards/CardDefinition.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardDefinition {
    /// Unique identifier for the card type (e.g. `"forest"`, `"lightning-bolt"`).
    id: String,
    /// Human-readable card name.
    name: String,
    /// One or more card types (most cards have exactly one).
    types: Vec<CardType>,
    /// Whether this card has the Legendary supertype (CR 205.4).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    is_legendary: bool,
    /// Subtypes (creature types, land types, etc.) per CR 205.3.
    /// Examples: "Human", "Warrior", "Forest", "Plains", "Equipment".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    subtypes: Vec<String>,
    /// Mana cost, if any. Lands have no mana cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    mana_cost: Option<ManaCost>,
    /// Base power for creatures. Ignored for non-creatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    power: Option<u32>,
    /// Base toughness for creatures. Ignored for non-creatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    toughness: Option<u32>,
    /// Static keyword abilities (Flying, Reach, Vigilance, etc.).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    static_abilities: Vec<StaticAbility>,
    /// All activated abilities on this card (CR 602).
    ///
    /// Most cards have zero or one activated ability, but some cards (e.g.
    /// multi-ability permanents) can have more than one.
    #[serde(skip)]
    activated_abilities: Vec<ActivatedAbility>,
    /// Triggered abilities on this card.
    #[serde(skip)]
    triggers: Vec<Trigger>,
    /// What kind of target this spell requires at cast time.
    #[serde(skip)]
    target_requirement: TargetRequirement,
    /// Human-readable rules text shown on the card (oracle text).
    ///
    /// `None` for basic lands and other cards without printed text.
    #[serde(skip_serializing_if = "Option::is_none")]
    oracle_text: Option<String>,
}

impl CardDefinition {
    /// Create a new card definition with mandatory fields.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        types: Vec<CardType>,
    ) -> Self {
        CardDefinition {
            id: id.into(),
            name: name.into(),
            types,
            is_legendary: false,
            subtypes: Vec::new(),
            mana_cost: None,
            power: None,
            toughness: None,
            static_abilities: Vec::new(),
            activated_abilities: Vec::new(),
            triggers: Vec::new(),
            target_requirement: TargetRequirement::None,
            oracle_text: None,
        }
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

    /// The card's unique definition ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The card's display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The card's types.
    pub fn types(&self) -> &[CardType] {
        &self.types
    }

    /// Whether this card has the Legendary supertype (CR 205.4).
    pub fn is_legendary(&self) -> bool {
        self.is_legendary
    }

    /// The card's subtypes (creature types, land types, etc.).
    pub fn subtypes(&self) -> &[String] {
        &self.subtypes
    }

    /// Returns `true` if this card has the given subtype (case-insensitive).
    ///
    /// CR 702.73: A card with Changeling has all creature types, so this method
    /// returns `true` for any subtype query if the card has Changeling.
    pub fn has_subtype(&self, subtype: &str) -> bool {
        // CR 702.73: Changeling means this card has all creature types.
        if self.has_static_ability(crate::domain::enums::StaticAbility::Changeling) {
            return true;
        }
        self.subtypes.iter().any(|s| s.eq_ignore_ascii_case(subtype))
    }

    /// The mana cost, if any.
    pub fn mana_cost(&self) -> Option<&ManaCost> {
        self.mana_cost.as_ref()
    }

    /// Base power for creature cards.
    pub fn power(&self) -> Option<u32> {
        self.power
    }

    /// Base toughness for creature cards.
    pub fn toughness(&self) -> Option<u32> {
        self.toughness
    }

    /// Static keyword abilities.
    pub fn static_abilities(&self) -> &[StaticAbility] {
        &self.static_abilities
    }

    /// All activated abilities on this card (CR 602).
    ///
    /// Returns a slice — most cards have zero or one ability, but some have more.
    pub fn activated_abilities(&self) -> &[ActivatedAbility] {
        &self.activated_abilities
    }

    /// Returns the first activated ability on this card, if any.
    ///
    /// Convenience method for the common single-ability case (lands, Sol Ring,
    /// etc.). Use `activated_abilities()` when you need all abilities or need
    /// to index by position.
    pub fn first_activated_ability(&self) -> Option<&ActivatedAbility> {
        self.activated_abilities.first()
    }

    /// Returns the activated ability at the given index, or `None` if out of bounds.
    ///
    /// Used by the `ActivateAbility` action handler to dispatch to the correct
    /// ability when a permanent has more than one (CR 602.1).
    pub fn activated_ability_at(&self, index: usize) -> Option<&ActivatedAbility> {
        self.activated_abilities.get(index)
    }

    /// Triggered abilities on this card.
    pub fn triggers(&self) -> &[Trigger] {
        &self.triggers
    }

    /// What kind of target this spell requires at cast time.
    pub fn target_requirement(&self) -> TargetRequirement {
        self.target_requirement
    }

    /// Human-readable rules text (oracle text), if any.
    pub fn oracle_text(&self) -> Option<&str> {
        self.oracle_text.as_deref()
    }

    // -------------------------------------------------------------------------
    // Type predicates
    // -------------------------------------------------------------------------

    /// Returns `true` if this card is a land.
    pub fn is_land(&self) -> bool {
        self.types.contains(&CardType::Land)
    }

    /// Returns `true` if this card is a creature.
    pub fn is_creature(&self) -> bool {
        self.types.contains(&CardType::Creature)
    }

    /// Returns `true` if this card is an instant.
    pub fn is_instant(&self) -> bool {
        self.types.contains(&CardType::Instant)
    }

    /// Returns `true` if this card is an artifact.
    pub fn is_artifact(&self) -> bool {
        self.types.contains(&CardType::Artifact)
    }

    /// Returns `true` if this card is an enchantment.
    pub fn is_enchantment(&self) -> bool {
        self.types.contains(&CardType::Enchantment)
    }

    /// Returns `true` if this card has the given static ability.
    pub fn has_static_ability(&self, ability: StaticAbility) -> bool {
        self.static_abilities.contains(&ability)
    }

    /// Returns the colors of this card, derived from its mana cost (CR 105.2).
    ///
    /// A card's color identity is determined by the colored mana symbols in its
    /// cost. Cards with no colored mana (e.g. Sol Ring {1}) are colorless.
    /// Lands have no mana cost and are colorless.
    ///
    /// CR 702.114: A card with Devoid is colorless regardless of its mana cost.
    pub fn colors(&self) -> Vec<ManaColor> {
        // CR 702.114: Devoid overrides color from mana cost.
        if self.has_static_ability(crate::domain::enums::StaticAbility::Devoid) {
            return Vec::new();
        }
        let Some(cost) = &self.mana_cost else {
            return Vec::new();
        };
        let mut colors = Vec::new();
        if cost.white > 0 { colors.push(ManaColor::White); }
        if cost.blue > 0 { colors.push(ManaColor::Blue); }
        if cost.black > 0 { colors.push(ManaColor::Black); }
        if cost.red > 0 { colors.push(ManaColor::Red); }
        if cost.green > 0 { colors.push(ManaColor::Green); }
        colors
    }

    /// Returns `true` if this card is colorless (no colored mana in cost).
    ///
    /// CR 702.114: A card with Devoid is always colorless.
    pub fn is_colorless(&self) -> bool {
        self.colors().is_empty()
    }

    // -------------------------------------------------------------------------
    // Builder methods (consume `self`)
    // -------------------------------------------------------------------------

    /// Attach a mana cost.
    pub fn with_mana_cost(mut self, cost: ManaCost) -> Self {
        self.mana_cost = Some(cost);
        self
    }

    /// Mark this card as Legendary (CR 205.4).
    pub fn with_legendary(mut self) -> Self {
        self.is_legendary = true;
        self
    }

    /// Add a subtype (creature type, land type, etc.).
    pub fn with_subtype(mut self, subtype: impl Into<String>) -> Self {
        self.subtypes.push(subtype.into());
        self
    }

    /// Set power and toughness (creature stats).
    pub fn with_power_toughness(mut self, power: u32, toughness: u32) -> Self {
        self.power = Some(power);
        self.toughness = Some(toughness);
        self
    }

    /// Add a static keyword ability.
    pub fn with_static_ability(mut self, ability: StaticAbility) -> Self {
        self.static_abilities.push(ability);
        self
    }

    /// Add an activated ability to this card.
    ///
    /// Multiple calls append to the list, allowing cards with more than one
    /// activated ability (CR 602). The first call behaves like the old
    /// `with_activated_ability()` — all existing callers are unaffected.
    pub fn with_activated_ability(mut self, ability: ActivatedAbility) -> Self {
        self.activated_abilities.push(ability);
        self
    }

    /// Add a triggered ability.
    pub fn with_trigger(mut self, trigger: Trigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Set the target requirement for this spell.
    pub fn with_target_requirement(mut self, req: TargetRequirement) -> Self {
        self.target_requirement = req;
        self
    }

    /// Set the oracle text (printed rules text) for this card.
    pub fn with_oracle_text(mut self, text: impl Into<String>) -> Self {
        self.oracle_text = Some(text.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forest() -> CardDefinition {
        CardDefinition::new("forest", "Forest", vec![CardType::Land])
    }

    fn bear() -> CardDefinition {
        CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2)
    }

    #[test]
    fn forest_is_land() {
        assert!(forest().is_land());
        assert!(!forest().is_creature());
    }

    #[test]
    fn bear_is_creature_with_stats() {
        let b = bear();
        assert!(b.is_creature());
        assert_eq!(b.power(), Some(2));
        assert_eq!(b.toughness(), Some(2));
    }

    #[test]
    fn land_has_no_mana_cost() {
        assert!(forest().mana_cost().is_none());
    }

    #[test]
    fn accessors_return_correct_values() {
        let def = forest();
        assert_eq!(def.id(), "forest");
        assert_eq!(def.name(), "Forest");
        assert_eq!(def.types(), &[CardType::Land]);
    }

    #[test]
    fn static_ability_detection() {
        let flyer = CardDefinition::new("x", "X", vec![CardType::Creature])
            .with_static_ability(StaticAbility::Flying);
        assert!(flyer.has_static_ability(StaticAbility::Flying));
        assert!(!flyer.has_static_ability(StaticAbility::Reach));
    }

    #[test]
    fn no_static_abilities_by_default() {
        assert!(bear().static_abilities().is_empty());
    }

    #[test]
    fn no_activated_ability_by_default() {
        assert!(forest().activated_abilities().is_empty());
    }

    // ---- Multiple activated abilities (R15) ---------------------------------

    #[test]
    fn card_can_have_multiple_activated_abilities() {
        use crate::domain::abilities::{ActivatedAbility, ActivationCost};
        use crate::domain::effects::Effect;
        use crate::domain::enums::ManaColor;
        use crate::domain::value_objects::mana::ManaCost;

        let card = CardDefinition::new("dual-ability", "Dual Ability", vec![CardType::Land])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
            })
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Mana(ManaCost::parse("1").unwrap()),
                effect: Effect::DrawCards { amount: 1 },
            });

        assert_eq!(card.activated_abilities().len(), 2);
    }

    #[test]
    fn first_activated_ability_returns_first_in_vec() {
        use crate::domain::abilities::{ActivatedAbility, ActivationCost};
        use crate::domain::effects::Effect;
        use crate::domain::enums::ManaColor;

        let card = CardDefinition::new("tapper", "Tapper", vec![CardType::Land])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
            });

        assert!(card.first_activated_ability().is_some());
        assert_eq!(
            card.first_activated_ability().unwrap().effect,
            Effect::AddMana { color: ManaColor::Green, amount: 1 }
        );
    }

    #[test]
    fn first_activated_ability_returns_none_when_no_abilities() {
        assert!(forest().first_activated_ability().is_none());
    }

    #[test]
    fn activated_ability_at_index_returns_correct_ability() {
        use crate::domain::abilities::{ActivatedAbility, ActivationCost};
        use crate::domain::effects::Effect;
        use crate::domain::enums::ManaColor;
        use crate::domain::value_objects::mana::ManaCost;

        let card = CardDefinition::new("multi", "Multi", vec![CardType::Land])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
            })
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Mana(ManaCost::parse("2").unwrap()),
                effect: Effect::DrawCards { amount: 1 },
            });

        assert!(card.activated_ability_at(0).is_some());
        assert!(card.activated_ability_at(1).is_some());
        assert!(card.activated_ability_at(2).is_none());
        assert_eq!(
            card.activated_ability_at(1).unwrap().effect,
            Effect::DrawCards { amount: 1 }
        );
    }

    #[test]
    fn no_triggers_by_default() {
        assert!(bear().triggers().is_empty());
    }

    #[test]
    fn oracle_text_is_none_by_default() {
        assert!(forest().oracle_text().is_none());
    }

    #[test]
    fn with_oracle_text_sets_text() {
        let card = bear().with_oracle_text("Some rules text.");
        assert_eq!(card.oracle_text(), Some("Some rules text."));
    }

    #[test]
    fn oracle_text_accessor_returns_str() {
        let card = CardDefinition::new("x", "X", vec![CardType::Creature])
            .with_oracle_text("Flying");
        assert_eq!(card.oracle_text(), Some("Flying"));
    }

    #[test]
    fn is_artifact_returns_true_for_artifact() {
        let art = CardDefinition::new("sol-ring", "Sol Ring", vec![CardType::Artifact]);
        assert!(art.is_artifact());
        assert!(!art.is_creature());
        assert!(!art.is_enchantment());
    }

    #[test]
    fn is_enchantment_returns_true_for_enchantment() {
        let enc = CardDefinition::new("test-enc", "Test Enc", vec![CardType::Enchantment]);
        assert!(enc.is_enchantment());
        assert!(!enc.is_creature());
        assert!(!enc.is_artifact());
    }

    #[test]
    fn is_artifact_false_for_non_artifact() {
        let land = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        assert!(!land.is_artifact());
    }

    #[test]
    fn is_enchantment_false_for_non_enchantment() {
        let land = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        assert!(!land.is_enchantment());
    }

    // ---- Color identity (CR 105.2) ------------------------------------------

    #[test]
    fn green_creature_has_green_color() {
        let bear = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_mana_cost(ManaCost::parse("1G").unwrap());
        assert_eq!(bear.colors(), vec![ManaColor::Green]);
        assert!(!bear.is_colorless());
    }

    #[test]
    fn red_instant_has_red_color() {
        let bolt = CardDefinition::new("bolt", "Bolt", vec![CardType::Instant])
            .with_mana_cost(ManaCost::parse("1R").unwrap());
        assert_eq!(bolt.colors(), vec![ManaColor::Red]);
    }

    #[test]
    fn multicolor_card_has_multiple_colors() {
        let gold = CardDefinition::new("gold", "Gold Card", vec![CardType::Creature])
            .with_mana_cost(ManaCost::parse("WG").unwrap());
        let colors = gold.colors();
        assert!(colors.contains(&ManaColor::White));
        assert!(colors.contains(&ManaColor::Green));
        assert_eq!(colors.len(), 2);
    }

    #[test]
    fn artifact_with_generic_cost_is_colorless() {
        let ring = CardDefinition::new("ring", "Ring", vec![CardType::Artifact])
            .with_mana_cost(ManaCost::parse("1").unwrap());
        assert!(ring.is_colorless());
        assert!(ring.colors().is_empty());
    }

    #[test]
    fn land_is_colorless() {
        let land = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        assert!(land.is_colorless());
    }

    // ---- Subtypes (CR 205.3) --------------------------------------------

    #[test]
    fn no_subtypes_by_default() {
        assert!(bear().subtypes().is_empty());
    }

    #[test]
    fn with_subtype_adds_subtype() {
        let card = CardDefinition::new("elf", "Elf", vec![CardType::Creature])
            .with_subtype("Elf");
        assert_eq!(card.subtypes(), &["Elf"]);
        assert!(card.has_subtype("Elf"));
    }

    #[test]
    fn has_subtype_is_case_insensitive() {
        let card = CardDefinition::new("elf", "Elf", vec![CardType::Creature])
            .with_subtype("Elf");
        assert!(card.has_subtype("elf"));
        assert!(card.has_subtype("ELF"));
    }

    #[test]
    fn multiple_subtypes() {
        let card = CardDefinition::new("ev", "Elite Vanguard", vec![CardType::Creature])
            .with_subtype("Human")
            .with_subtype("Soldier");
        assert_eq!(card.subtypes().len(), 2);
        assert!(card.has_subtype("Human"));
        assert!(card.has_subtype("Soldier"));
        assert!(!card.has_subtype("Elf"));
    }

    // ---- Changeling (CR 702.73) --------------------------------------------

    #[test]
    fn changeling_has_all_subtypes() {
        let card = CardDefinition::new("shapeshifter", "Shapeshifter", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_static_ability(StaticAbility::Changeling);
        // Changeling means it has ALL creature types
        assert!(card.has_subtype("Elf"), "Changeling should have Elf subtype");
        assert!(card.has_subtype("Wizard"), "Changeling should have Wizard subtype");
        assert!(card.has_subtype("Dragon"), "Changeling should have Dragon subtype");
        assert!(card.has_subtype("Human"), "Changeling should have Human subtype");
    }

    // ---- Devoid (CR 702.114) -----------------------------------------------

    #[test]
    fn devoid_card_is_colorless_despite_colored_mana_cost() {
        let card = CardDefinition::new("eldrazi", "Eldrazi", vec![CardType::Creature])
            .with_mana_cost(ManaCost::parse("1R").unwrap())
            .with_power_toughness(2, 2)
            .with_static_ability(StaticAbility::Devoid);
        assert!(card.is_colorless(), "Devoid card should be colorless");
        assert!(card.colors().is_empty(), "Devoid card should have no colors");
    }

    #[test]
    fn non_devoid_red_card_has_red_color() {
        let card = CardDefinition::new("goblin", "Goblin", vec![CardType::Creature])
            .with_mana_cost(ManaCost::parse("R").unwrap())
            .with_power_toughness(1, 1);
        assert!(!card.is_colorless());
        assert!(card.colors().contains(&ManaColor::Red));
    }
}
