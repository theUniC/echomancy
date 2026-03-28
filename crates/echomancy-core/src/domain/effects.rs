//! Effect system — executable part of spells and abilities.
//!
//! Effects are represented as a closed-set enum rather than `Box<dyn Trait>`
//! because the MVP has a finite, known set of effects. This avoids trait-object
//! complexity and makes exhaustive matching possible.
//!
//! Mirrors the TypeScript `Effect` interface and its implementations from
//! `effects/Effect.ts`, `effects/impl/DrawCardsEffect.ts`, etc.

use crate::domain::enums::ManaColor;
use crate::domain::targets::Target;

/// Context carried with every effect resolution.
///
/// Mirrors the TypeScript `EffectContext` from `effects/EffectContext.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectContext {
    /// The ID of the card that sourced this effect (Last Known Information).
    pub source_id: Option<String>,
    /// The player who controls the effect.
    pub controller_id: String,
    /// MVP: always empty for most effects; player-targeted effects use index 0.
    pub targets: Vec<Target>,
}

impl EffectContext {
    /// Create a context for a controller with no source or targets.
    pub fn new(controller_id: impl Into<String>) -> Self {
        EffectContext {
            source_id: None,
            controller_id: controller_id.into(),
            targets: Vec::new(),
        }
    }

    /// Create a context with a specific source card.
    pub fn with_source(
        source_id: impl Into<String>,
        controller_id: impl Into<String>,
    ) -> Self {
        EffectContext {
            source_id: Some(source_id.into()),
            controller_id: controller_id.into(),
            targets: Vec::new(),
        }
    }

    /// Builder: attach targets to this context.
    pub fn with_targets(mut self, targets: Vec<Target>) -> Self {
        self.targets = targets;
        self
    }
}

/// All effect variants supported by the MVP rules engine.
///
/// Choosing an enum (closed set) over `Box<dyn Effect>` (open set) is an
/// intentional design decision: every effect in the MVP is known at compile
/// time, making exhaustive matching both possible and safe.
///
/// Mirrors:
/// - `DrawCardsEffect`   → `DrawCards { amount }`
/// - `DrawTargetPlayerEffect` → `DrawTargetPlayer { amount }`
/// - `NoOpEffect`        → `NoOp`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Draw `amount` cards for the controller of the ability.
    DrawCards { amount: u32 },

    /// Draw `amount` cards for the first `Target::Player` in the context.
    DrawTargetPlayer { amount: u32 },

    /// Add `amount` mana of the given color to the controller's pool.
    ///
    /// Per MTG CR 605, mana abilities resolve immediately without using the
    /// stack. The `activate_ability` handler checks for this variant and
    /// bypasses the stack entirely.
    AddMana { color: ManaColor, amount: u32 },

    /// No-op — does nothing on resolution.
    NoOp,
}

impl Effect {
    /// Convenience constructor for `DrawCards`.
    pub fn draw_cards(amount: u32) -> Self {
        Effect::DrawCards { amount }
    }

    /// Convenience constructor for `DrawTargetPlayer`.
    pub fn draw_target_player(amount: u32) -> Self {
        Effect::DrawTargetPlayer { amount }
    }

    /// Convenience constructor for `AddMana`.
    pub fn add_mana(color: ManaColor, amount: u32) -> Self {
        Effect::AddMana { color, amount }
    }

    /// Returns `true` if this effect is a mana ability.
    ///
    /// Per MTG CR 605, mana abilities resolve immediately without using the
    /// stack. They also don't use the stack when activated.
    pub fn is_mana_ability(&self) -> bool {
        matches!(self, Effect::AddMana { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_cards_constructor() {
        let e = Effect::draw_cards(3);
        assert_eq!(e, Effect::DrawCards { amount: 3 });
    }

    #[test]
    fn draw_target_player_constructor() {
        let e = Effect::draw_target_player(2);
        assert_eq!(e, Effect::DrawTargetPlayer { amount: 2 });
    }

    #[test]
    fn add_mana_constructor() {
        let e = Effect::add_mana(ManaColor::Green, 1);
        assert_eq!(e, Effect::AddMana { color: ManaColor::Green, amount: 1 });
    }

    #[test]
    fn add_mana_is_mana_ability() {
        let e = Effect::add_mana(ManaColor::Red, 1);
        assert!(e.is_mana_ability());
    }

    #[test]
    fn draw_cards_is_not_mana_ability() {
        let e = Effect::draw_cards(1);
        assert!(!e.is_mana_ability());
    }

    #[test]
    fn no_op_is_not_mana_ability() {
        assert!(!Effect::NoOp.is_mana_ability());
    }

    #[test]
    fn no_op_variant() {
        let e = Effect::NoOp;
        assert_eq!(e, Effect::NoOp);
    }

    #[test]
    fn effect_context_new() {
        let ctx = EffectContext::new("player-1");
        assert_eq!(ctx.controller_id, "player-1");
        assert!(ctx.source_id.is_none());
        assert!(ctx.targets.is_empty());
    }

    #[test]
    fn effect_context_with_source() {
        let ctx = EffectContext::with_source("card-1", "player-1");
        assert_eq!(ctx.source_id, Some("card-1".to_owned()));
        assert_eq!(ctx.controller_id, "player-1");
    }

    #[test]
    fn effect_context_with_targets() {
        use crate::domain::targets::Target;
        let ctx = EffectContext::new("player-1")
            .with_targets(vec![Target::player("player-2")]);
        assert_eq!(ctx.targets.len(), 1);
    }
}
