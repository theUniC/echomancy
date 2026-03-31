//! Card definition — the template/blueprint for a card.
//!
//! Mirrors the TypeScript `CardDefinition` type from `cards/CardDefinition.ts`.

use serde::{Deserialize, Serialize};

use crate::domain::abilities::ActivatedAbility;
use crate::domain::enums::{CardType, StaticAbility};
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
    /// A single activated ability on this card, if any.
    #[serde(skip)]
    activated_ability: Option<ActivatedAbility>,
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
            mana_cost: None,
            power: None,
            toughness: None,
            static_abilities: Vec::new(),
            activated_ability: None,
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

    /// The single activated ability on this card, if any.
    pub fn activated_ability(&self) -> Option<&ActivatedAbility> {
        self.activated_ability.as_ref()
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

    // -------------------------------------------------------------------------
    // Builder methods (consume `self`)
    // -------------------------------------------------------------------------

    /// Attach a mana cost.
    pub fn with_mana_cost(mut self, cost: ManaCost) -> Self {
        self.mana_cost = Some(cost);
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

    /// Set the activated ability.
    pub fn with_activated_ability(mut self, ability: ActivatedAbility) -> Self {
        self.activated_ability = Some(ability);
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
        assert!(forest().activated_ability().is_none());
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
}
