//! Cost system — what must be paid before an effect resolves.
//!
//! Costs are represented as a closed-set enum for the same reason as effects:
//! the MVP has a finite, known set of cost types.
//!

use crate::domain::value_objects::mana::ManaCost as ManaValue;

/// Context provided when validating or paying a cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostContext {
    /// The player who is paying the cost.
    pub player_id: String,
    /// The card or ability that requires the cost (its instance ID).
    pub source_id: String,
}

impl CostContext {
    /// Create a new cost context.
    pub fn new(player_id: impl Into<String>, source_id: impl Into<String>) -> Self {
        CostContext {
            player_id: player_id.into(),
            source_id: source_id.into(),
        }
    }
}

/// All cost variants supported by the MVP rules engine.
///
/// Using an enum (closed set) over `Box<dyn Cost>` (open set) avoids
/// trait-object complexity and enables exhaustive matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cost {
    /// Pay mana from the player's pool.
    Mana { cost: ManaValue },

    /// Tap the permanent that has this ability.
    TapSelf,

    /// Sacrifice the permanent that has this ability.
    SacrificeSelf,
}

impl Cost {
    /// Convenience constructor for a mana cost.
    pub fn mana(cost: ManaValue) -> Self {
        Cost::Mana { cost }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::mana::ManaCost;

    #[test]
    fn cost_context_new() {
        let ctx = CostContext::new("player-1", "card-abc");
        assert_eq!(ctx.player_id, "player-1");
        assert_eq!(ctx.source_id, "card-abc");
    }

    #[test]
    fn tap_self_variant() {
        let cost = Cost::TapSelf;
        assert_eq!(cost, Cost::TapSelf);
    }

    #[test]
    fn sacrifice_self_variant() {
        let cost = Cost::SacrificeSelf;
        assert_eq!(cost, Cost::SacrificeSelf);
    }

    #[test]
    fn mana_cost_variant() {
        let mc = ManaCost::parse("2G").unwrap();
        let cost = Cost::mana(mc.clone());
        assert_eq!(cost, Cost::Mana { cost: mc });
    }

    #[test]
    fn mana_zero_cost() {
        let mc = ManaCost::zero();
        let cost = Cost::mana(mc.clone());
        assert_eq!(cost, Cost::Mana { cost: mc });
    }
}
