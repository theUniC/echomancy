//! Effect system — executable part of spells and abilities.
//!
//! Effects are represented as a closed-set enum rather than `Box<dyn Trait>`
//! because the MVP has a finite, known set of effects. This avoids trait-object
//! complexity and makes exhaustive matching possible.
//!
//! Mirrors the TypeScript `Effect` interface and its implementations from
//! `effects/Effect.ts`, `effects/impl/DrawCardsEffect.ts`, etc.

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
