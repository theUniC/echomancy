//! Ability system — activated and triggered abilities.
//!
//! Mirrors the TypeScript `Ability`, `ActivatedAbility` from
//! `abilities/Ability.ts` and `abilities/ActivatedAbility.ts`.
//!
//! The TS `ActivationCost` was a struct `{ type: "TAP" }`.
//! In MVP only TAP cost is supported for activated abilities.
//! We express this as a unit struct `ActivationCost` rather than a full enum,
//! matching the TS scope.

use crate::domain::effects::Effect;
use crate::domain::triggers::Trigger;
use crate::domain::value_objects::mana::ManaCost;

/// The cost to activate an ability (CR 602.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationCost {
    /// The permanent must be tapped to activate the ability.
    /// Used for mana abilities on lands, Sol Ring, etc.
    Tap,
    /// The player must pay a mana cost and tap the permanent.
    /// Used for equipment equip costs and similar abilities.
    TapAndMana(ManaCost),
    /// The player must pay a mana cost (no tap required).
    /// Used for pump abilities, etc. e.g. `{2}: +1/+1 until end of turn`.
    Mana(ManaCost),
}

/// An ability that a player can activate by paying a cost.
///
/// Mirrors the TypeScript `ActivatedAbility` from `abilities/ActivatedAbility.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatedAbility {
    /// The cost to activate this ability.
    pub cost: ActivationCost,
    /// The effect that resolves when this ability is activated.
    pub effect: Effect,
}

impl ActivatedAbility {
    /// Create a new activated ability with a TAP cost.
    pub fn tap_ability(effect: Effect) -> Self {
        ActivatedAbility {
            cost: ActivationCost::Tap,
            effect,
        }
    }

    /// Create a new activated ability with a mana-only cost.
    pub fn mana_ability(cost: ManaCost, effect: Effect) -> Self {
        ActivatedAbility {
            cost: ActivationCost::Mana(cost),
            effect,
        }
    }

    /// Create a new activated ability with a tap-plus-mana cost.
    pub fn tap_and_mana_ability(cost: ManaCost, effect: Effect) -> Self {
        ActivatedAbility {
            cost: ActivationCost::TapAndMana(cost),
            effect,
        }
    }
}

/// Union type for all ability types on a card.
///
/// Mirrors the TypeScript `Ability = ActivatedAbility | Trigger`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ability {
    /// An ability the player explicitly activates.
    Activated(ActivatedAbility),
    /// An ability that fires automatically on an event.
    Triggered(Trigger),
}

impl Ability {
    /// Returns the activated ability if this is an `Activated` variant.
    pub fn as_activated(&self) -> Option<&ActivatedAbility> {
        match self {
            Ability::Activated(a) => Some(a),
            Ability::Triggered(_) => None,
        }
    }

    /// Returns the triggered ability if this is a `Triggered` variant.
    pub fn as_triggered(&self) -> Option<&Trigger> {
        match self {
            Ability::Activated(_) => None,
            Ability::Triggered(t) => Some(t),
        }
    }

    /// Returns `true` if this is an activated ability.
    pub fn is_activated(&self) -> bool {
        self.as_activated().is_some()
    }

    /// Returns `true` if this is a triggered ability.
    pub fn is_triggered(&self) -> bool {
        self.as_triggered().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::effects::Effect;
    use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};

    #[test]
    fn activated_ability_tap_cost() {
        let ab = ActivatedAbility::tap_ability(Effect::draw_cards(1));
        assert_eq!(ab.cost, ActivationCost::Tap);
        assert_eq!(ab.effect, Effect::DrawCards { amount: 1 });
    }

    #[test]
    fn ability_from_activated() {
        let ab = ActivatedAbility::tap_ability(Effect::NoOp);
        let ability = Ability::Activated(ab);
        assert!(ability.is_activated());
        assert!(!ability.is_triggered());
    }

    #[test]
    fn ability_from_trigger() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::NoOp,
        );
        let ability = Ability::Triggered(trigger);
        assert!(ability.is_triggered());
        assert!(!ability.is_activated());
    }

    #[test]
    fn as_activated_returns_none_for_trigger() {
        let trigger = Trigger::new(
            TriggerEventType::ZoneChanged,
            TriggerCondition::Always,
            Effect::NoOp,
        );
        let ability = Ability::Triggered(trigger);
        assert!(ability.as_activated().is_none());
    }

    #[test]
    fn as_triggered_returns_none_for_activated() {
        let ab = ActivatedAbility::tap_ability(Effect::NoOp);
        let ability = Ability::Activated(ab);
        assert!(ability.as_triggered().is_none());
    }
}
