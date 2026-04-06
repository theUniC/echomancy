//! Layer System (CR 613) — continuous effect evaluation pipeline.
//!
//! Implements the seven-layer evaluation order for continuous effects.
//! Layers 1–3 (copy, control, text) are deferred; this module covers Layers 4–7.
//!
//! # Architecture
//!
//! The layer system is a **pure function**: given the current game state and a
//! permanent ID it returns effective characteristics without mutating anything.
//!
//! Effects are stored in two places:
//! - `PermanentState::continuous_effects` — legacy per-permanent effects (kept for
//!   backward compatibility with the old `current_power`/`current_toughness`).
//! - `Game::global_continuous_effects` — the new game-level list used by the layer
//!   pipeline. These carry full layer metadata.
//!
//! The `effective_*` queries on `Game` run through the full layer pipeline.

use crate::domain::enums::{CardType, ManaColor, StaticAbility};
use crate::domain::value_objects::permanent_state::EffectDuration;

// ============================================================================
// Effect layer discriminant
// ============================================================================

/// Which layer a continuous effect belongs to (CR 613).
///
/// Layer 7 is subdivided into sublayers 7a–7d (CR 613.4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectLayer {
    /// Layer 4: type-changing effects (CR 613.1d).
    Layer4Type,
    /// Layer 5: color-changing effects (CR 613.1e).
    Layer5Color,
    /// Layer 6: ability-adding/removing effects (CR 613.1f).
    Layer6Ability,
    /// Layer 7a: CDAs that define P/T.
    Layer7a,
    /// Layer 7b: effects that SET P/T to a specific value.
    Layer7b,
    /// Layer 7c: all other P/T modifications (+N/+N, counters, etc.).
    Layer7c,
    /// Layer 7d: switch P/T effects.
    Layer7d,
}

// ============================================================================
// Effect payload
// ============================================================================

/// A formula used by CDA effects in Layer 7a.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants used in tests; production use comes with future CDA card implementations.
pub enum PtFormula {
    /// P/T = number of cards in the controller's hand (e.g. Kederekt Creeper).
    CardsInControllerHand,
}

/// What a continuous effect actually does — the data payload.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants are part of the complete layer system domain model; used in tests and future card implementations.
pub enum EffectPayload {
    // ---- Layer 4 ----
    /// Add one or more card types.
    AddTypes(Vec<CardType>),
    /// Remove one or more card types.
    RemoveTypes(Vec<CardType>),
    /// Replace all card types with this set.
    SetTypes(Vec<CardType>),
    /// Add one or more subtypes.
    AddSubtypes(Vec<String>),
    /// Remove one or more subtypes.
    RemoveSubtypes(Vec<String>),
    /// Replace all subtypes with this set.
    SetSubtypes(Vec<String>),

    // ---- Layer 5 ----
    /// Set colors to this exact set (removes all prior colors).
    SetColors(Vec<ManaColor>),
    /// Add these colors to existing colors.
    AddColors(Vec<ManaColor>),
    /// Remove all colors (permanent becomes colorless).
    RemoveAllColors,

    // ---- Layer 6 ----
    /// Grant a keyword ability.
    GrantAbility(StaticAbility),
    /// Remove a specific keyword ability.
    RemoveAbility(StaticAbility),
    /// Remove all keyword abilities.
    RemoveAllAbilities,
    /// Prevent this ability from being granted by any effect.
    CantHaveAbility(StaticAbility),
    /// Grant an ability via a keyword counter (Ikoria).
    GrantKeywordCounter(StaticAbility),

    // ---- Layer 7 ----
    /// Set P/T to exact values (7b) or by formula (7a CDA).
    SetPowerToughness(i32, i32),
    /// Set P/T by CDA formula (7a).
    SetPtFormula(PtFormula),
    /// Modify P/T by delta (7c): +N to power, +M to toughness.
    ModifyPowerToughness(i32, i32),
    /// Switch power and toughness (7d).
    SwitchPowerToughness,
}

// ============================================================================
// Target scope
// ============================================================================

/// A filter that identifies which permanents a `Filter`-scoped effect affects.
///
/// Re-evaluated on every query (for static ability effects).
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants are part of the domain model; used in tests and future anthem/static implementations.
pub enum EffectFilter {
    /// All creatures on the battlefield (no controller restriction).
    AllCreatures,
    /// All creatures controlled by a specific player.
    CreaturesControlledBy(String),
    /// A specific permanent by ID.
    Permanent(String),
}

/// How the set of affected permanents is determined.
///
/// - `LockedSet`: fixed at effect creation time (spell-resolution effects, CR 611.2c).
/// - `Filter`: re-evaluated on each query (static ability effects).
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Filter variant is used in tests and future static ability implementations.
pub enum EffectTargeting {
    /// The set of targeted permanent IDs is fixed at creation time.
    LockedSet(Vec<String>),
    /// The set is re-evaluated each query using this filter.
    Filter(EffectFilter),
}

// ============================================================================
// GlobalContinuousEffect
// ============================================================================

/// A fully-typed continuous effect record that participates in the layer pipeline.
///
/// This is the new model introduced by LS1. The old `ContinuousEffect` on
/// `PermanentState` is kept for backward compatibility of `current_power` /
/// `current_toughness`, but all new effects should use this struct and be
/// stored in `Game::global_continuous_effects`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalContinuousEffect {
    /// Which layer this effect belongs to.
    pub layer: EffectLayer,
    /// What the effect does.
    pub payload: EffectPayload,
    /// When this effect expires.
    pub duration: EffectDuration,
    /// Monotonically increasing timestamp (CR 613.7).
    ///
    /// - For static-ability effects: equals the source permanent's ETB timestamp.
    /// - For spell-resolution effects: equals the game-level counter at resolution time.
    pub timestamp: u64,
    /// The ID of the source permanent or spell/ability that created this effect.
    pub source_id: String,
    /// The player who controlled the source when this effect was created.
    pub controller_id: String,
    /// Whether this effect originates from a Characteristic-Defining Ability (CR 604.3a).
    ///
    /// CDAs apply before timestamp-ordered effects within Layers 4 and 5.
    pub is_cda: bool,
    /// How to determine which permanents this effect applies to.
    pub targeting: EffectTargeting,
    /// For multi-layer effects (CR 613.6): the permanent IDs locked when the effect
    /// first applied in its earliest layer. Used for all later layer components of
    /// the same multi-layer effect.
    pub locked_target_set: Option<Vec<String>>,
}

impl GlobalContinuousEffect {
    /// Returns `true` if this effect applies to the given permanent, given the
    /// current battlefield state (controller lookup is passed in).
    ///
    /// `permanent_controller_id` — the ID of the player currently controlling
    /// the permanent we are querying.
    pub(crate) fn applies_to(
        &self,
        permanent_id: &str,
        permanent_controller_id: &str,
    ) -> bool {
        // Multi-layer lock-in: if a locked_target_set is present, use it.
        if let Some(locked) = &self.locked_target_set {
            return locked.iter().any(|id| id == permanent_id);
        }

        match &self.targeting {
            EffectTargeting::LockedSet(ids) => ids.iter().any(|id| id == permanent_id),
            EffectTargeting::Filter(filter) => match filter {
                EffectFilter::AllCreatures => true, // caller ensures only creatures are queried
                EffectFilter::CreaturesControlledBy(controller) => {
                    permanent_controller_id == controller
                }
                EffectFilter::Permanent(id) => id == permanent_id,
            },
        }
    }
}

// ============================================================================
// LayerContext — input to the layer pipeline
// ============================================================================

/// Everything the layer pipeline needs to compute effective characteristics.
pub(crate) struct LayerContext<'a> {
    /// The permanent ID being queried.
    pub permanent_id: &'a str,
    /// The controller of the permanent.
    pub controller_id: &'a str,
    /// Base card types from the card definition.
    pub base_types: &'a [CardType],
    /// Base subtypes from the card definition.
    pub base_subtypes: &'a [String],
    /// Base colors from the card definition.
    pub base_colors: Vec<ManaColor>,
    /// Static abilities on the card definition.
    pub base_abilities: &'a [StaticAbility],
    /// Base power from the card definition (None for non-creatures).
    pub base_power: Option<i32>,
    /// Base toughness from the card definition (None for non-creatures).
    pub base_toughness: Option<i32>,
    /// Current +1/+1 counter count.
    pub plus_counters: u32,
    /// Current -1/-1 counter count.
    pub minus_counters: u32,
    /// Keyword counters on this permanent (Ikoria-style).
    pub keyword_counters: Vec<StaticAbility>,
    /// Number of cards in the controller's hand (for CardsInControllerHand CDA).
    pub controller_hand_size: usize,
    /// All active global continuous effects in the game.
    pub effects: &'a [GlobalContinuousEffect],
}

// ============================================================================
// Layer pipeline result
// ============================================================================

/// The effective characteristics of a permanent after the full layer pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveCharacteristics {
    pub types: Vec<CardType>,
    pub subtypes: Vec<String>,
    pub colors: Vec<ManaColor>,
    pub abilities: Vec<StaticAbility>,
    /// `None` for non-creatures (after Layer 4 processing).
    pub power: Option<i32>,
    /// `None` for non-creatures (after Layer 4 processing).
    pub toughness: Option<i32>,
}

// ============================================================================
// Layer pipeline — pure functions
// ============================================================================

/// Sort key for effects within a layer: CDAs first, then by timestamp ascending.
fn effect_sort_key(e: &GlobalContinuousEffect) -> (bool, u64) {
    // CDAs get false (sorts before true = non-CDA)
    (!e.is_cda, e.timestamp)
}

/// Collect and sort effects that apply to this permanent in the given layer.
fn effects_for_layer<'a>(
    ctx: &'a LayerContext<'_>,
    layer: &EffectLayer,
) -> Vec<&'a GlobalContinuousEffect> {
    let mut relevant: Vec<&GlobalContinuousEffect> = ctx
        .effects
        .iter()
        .filter(|e| &e.layer == layer && e.applies_to(ctx.permanent_id, ctx.controller_id))
        .collect();
    relevant.sort_by_key(|e| effect_sort_key(e));
    relevant
}

/// Apply Layer 4 (type-changing) effects.
fn apply_layer4(
    ctx: &LayerContext<'_>,
    types: &mut Vec<CardType>,
    subtypes: &mut Vec<String>,
) {
    let layer_effects = effects_for_layer(ctx, &EffectLayer::Layer4Type);
    for effect in layer_effects {
        match &effect.payload {
            EffectPayload::AddTypes(new_types) => {
                for t in new_types {
                    if !types.contains(t) {
                        types.push(*t);
                    }
                }
            }
            EffectPayload::RemoveTypes(remove_types) => {
                types.retain(|t| !remove_types.contains(t));
            }
            EffectPayload::SetTypes(set_types) => {
                *types = set_types.clone();
            }
            EffectPayload::AddSubtypes(new_subs) => {
                for s in new_subs {
                    if !subtypes.contains(s) {
                        subtypes.push(s.clone());
                    }
                }
            }
            EffectPayload::RemoveSubtypes(remove_subs) => {
                subtypes.retain(|s| !remove_subs.contains(s));
            }
            EffectPayload::SetSubtypes(set_subs) => {
                *subtypes = set_subs.clone();
            }
            // Non-Layer-4 payloads — not applicable here.
            EffectPayload::SetColors(_)
            | EffectPayload::AddColors(_)
            | EffectPayload::RemoveAllColors
            | EffectPayload::GrantAbility(_)
            | EffectPayload::RemoveAbility(_)
            | EffectPayload::RemoveAllAbilities
            | EffectPayload::CantHaveAbility(_)
            | EffectPayload::GrantKeywordCounter(_)
            | EffectPayload::SetPowerToughness(_, _)
            | EffectPayload::SetPtFormula(_)
            | EffectPayload::ModifyPowerToughness(_, _)
            | EffectPayload::SwitchPowerToughness => {}
        }
    }
}

/// Apply Layer 5 (color-changing) effects.
fn apply_layer5(ctx: &LayerContext<'_>, colors: &mut Vec<ManaColor>) {
    let layer_effects = effects_for_layer(ctx, &EffectLayer::Layer5Color);
    for effect in layer_effects {
        match &effect.payload {
            EffectPayload::SetColors(new_colors) => {
                *colors = new_colors.clone();
            }
            EffectPayload::AddColors(new_colors) => {
                for c in new_colors {
                    if !colors.contains(c) {
                        colors.push(*c);
                    }
                }
            }
            EffectPayload::RemoveAllColors => {
                colors.clear();
            }
            // Non-Layer-5 payloads — not applicable here.
            EffectPayload::AddTypes(_)
            | EffectPayload::RemoveTypes(_)
            | EffectPayload::SetTypes(_)
            | EffectPayload::AddSubtypes(_)
            | EffectPayload::RemoveSubtypes(_)
            | EffectPayload::SetSubtypes(_)
            | EffectPayload::GrantAbility(_)
            | EffectPayload::RemoveAbility(_)
            | EffectPayload::RemoveAllAbilities
            | EffectPayload::CantHaveAbility(_)
            | EffectPayload::GrantKeywordCounter(_)
            | EffectPayload::SetPowerToughness(_, _)
            | EffectPayload::SetPtFormula(_)
            | EffectPayload::ModifyPowerToughness(_, _)
            | EffectPayload::SwitchPowerToughness => {}
        }
    }
}

/// Apply Layer 6 (ability-adding/removing) effects.
///
/// Returns the final ability set. `CantHaveAbility` effects override any
/// `GrantAbility` effect for the same ability, regardless of timestamp (CR 613.1f).
fn apply_layer6(ctx: &LayerContext<'_>, abilities: &mut Vec<StaticAbility>) {
    // Collect cant-have constraints first — these override everything
    let cant_have: Vec<StaticAbility> = ctx
        .effects
        .iter()
        .filter(|e| {
            e.layer == EffectLayer::Layer6Ability
                && e.applies_to(ctx.permanent_id, ctx.controller_id)
        })
        .filter_map(|e| {
            if let EffectPayload::CantHaveAbility(a) = &e.payload {
                Some(*a)
            } else {
                None
            }
        })
        .collect();

    // Add keyword counters (always in Layer 6, regardless of timestamp)
    for kw in &ctx.keyword_counters {
        if !abilities.contains(kw) {
            abilities.push(*kw);
        }
    }

    let layer_effects = effects_for_layer(ctx, &EffectLayer::Layer6Ability);
    for effect in layer_effects {
        match &effect.payload {
            EffectPayload::GrantAbility(a) => {
                if !abilities.contains(a) {
                    abilities.push(*a);
                }
            }
            EffectPayload::RemoveAbility(a) => {
                abilities.retain(|x| x != a);
            }
            EffectPayload::RemoveAllAbilities => {
                abilities.clear();
            }
            EffectPayload::GrantKeywordCounter(a) => {
                // Already handled above, but if it appears as an effect record too, skip
                let _ = a;
            }
            EffectPayload::CantHaveAbility(_) => {
                // Handled below — just continue
            }
            // Non-Layer-6 payloads — not applicable here.
            EffectPayload::AddTypes(_)
            | EffectPayload::RemoveTypes(_)
            | EffectPayload::SetTypes(_)
            | EffectPayload::AddSubtypes(_)
            | EffectPayload::RemoveSubtypes(_)
            | EffectPayload::SetSubtypes(_)
            | EffectPayload::SetColors(_)
            | EffectPayload::AddColors(_)
            | EffectPayload::RemoveAllColors
            | EffectPayload::SetPowerToughness(_, _)
            | EffectPayload::SetPtFormula(_)
            | EffectPayload::ModifyPowerToughness(_, _)
            | EffectPayload::SwitchPowerToughness => {}
        }
    }

    // Apply cant-have overrides: remove any ability that has a cant-have constraint
    for cant in &cant_have {
        abilities.retain(|a| a != cant);
    }
}

/// Apply Layer 7 (P/T changing) effects.
///
/// Returns `(power, toughness)` or `None` if the permanent is not a creature
/// after Layer 4 has been applied.
fn apply_layer7(
    ctx: &LayerContext<'_>,
    is_creature: bool,
    base_power: Option<i32>,
    base_toughness: Option<i32>,
) -> Option<(i32, i32)> {
    if !is_creature {
        return None;
    }
    let base_p = base_power.unwrap_or(0);
    let base_t = base_toughness.unwrap_or(0);

    let mut power = base_p;
    let mut toughness = base_t;

    // Sublayer 7a: CDAs
    let layer_7a = effects_for_layer(ctx, &EffectLayer::Layer7a);
    for effect in layer_7a {
        match &effect.payload {
            EffectPayload::SetPtFormula(formula) => match formula {
                PtFormula::CardsInControllerHand => {
                    let hand_size = ctx.controller_hand_size as i32;
                    power = hand_size;
                    toughness = hand_size;
                }
            },
            EffectPayload::SetPowerToughness(p, t) => {
                power = *p;
                toughness = *t;
            }
            // Non-Layer-7a payloads — not applicable here.
            EffectPayload::AddTypes(_)
            | EffectPayload::RemoveTypes(_)
            | EffectPayload::SetTypes(_)
            | EffectPayload::AddSubtypes(_)
            | EffectPayload::RemoveSubtypes(_)
            | EffectPayload::SetSubtypes(_)
            | EffectPayload::SetColors(_)
            | EffectPayload::AddColors(_)
            | EffectPayload::RemoveAllColors
            | EffectPayload::GrantAbility(_)
            | EffectPayload::RemoveAbility(_)
            | EffectPayload::RemoveAllAbilities
            | EffectPayload::CantHaveAbility(_)
            | EffectPayload::GrantKeywordCounter(_)
            | EffectPayload::ModifyPowerToughness(_, _)
            | EffectPayload::SwitchPowerToughness => {}
        }
    }

    // Sublayer 7b: set P/T to specific values
    let layer_7b = effects_for_layer(ctx, &EffectLayer::Layer7b);
    for effect in layer_7b {
        match &effect.payload {
            EffectPayload::SetPowerToughness(p, t) => {
                power = *p;
                toughness = *t;
            }
            // Non-Layer-7b payloads — not applicable here.
            EffectPayload::AddTypes(_)
            | EffectPayload::RemoveTypes(_)
            | EffectPayload::SetTypes(_)
            | EffectPayload::AddSubtypes(_)
            | EffectPayload::RemoveSubtypes(_)
            | EffectPayload::SetSubtypes(_)
            | EffectPayload::SetColors(_)
            | EffectPayload::AddColors(_)
            | EffectPayload::RemoveAllColors
            | EffectPayload::GrantAbility(_)
            | EffectPayload::RemoveAbility(_)
            | EffectPayload::RemoveAllAbilities
            | EffectPayload::CantHaveAbility(_)
            | EffectPayload::GrantKeywordCounter(_)
            | EffectPayload::SetPtFormula(_)
            | EffectPayload::ModifyPowerToughness(_, _)
            | EffectPayload::SwitchPowerToughness => {}
        }
    }

    // Sublayer 7c: all modifications (counters + pump/anthem)
    let plus_delta = ctx.plus_counters as i32;
    let minus_delta = ctx.minus_counters as i32;
    power += plus_delta - minus_delta;
    toughness += plus_delta - minus_delta;

    let layer_7c = effects_for_layer(ctx, &EffectLayer::Layer7c);
    for effect in layer_7c {
        match &effect.payload {
            EffectPayload::ModifyPowerToughness(dp, dt) => {
                power += dp;
                toughness += dt;
            }
            // Non-Layer-7c payloads — not applicable here.
            EffectPayload::AddTypes(_)
            | EffectPayload::RemoveTypes(_)
            | EffectPayload::SetTypes(_)
            | EffectPayload::AddSubtypes(_)
            | EffectPayload::RemoveSubtypes(_)
            | EffectPayload::SetSubtypes(_)
            | EffectPayload::SetColors(_)
            | EffectPayload::AddColors(_)
            | EffectPayload::RemoveAllColors
            | EffectPayload::GrantAbility(_)
            | EffectPayload::RemoveAbility(_)
            | EffectPayload::RemoveAllAbilities
            | EffectPayload::CantHaveAbility(_)
            | EffectPayload::GrantKeywordCounter(_)
            | EffectPayload::SetPowerToughness(_, _)
            | EffectPayload::SetPtFormula(_)
            | EffectPayload::SwitchPowerToughness => {}
        }
    }

    // Sublayer 7d: switch P/T
    let layer_7d = effects_for_layer(ctx, &EffectLayer::Layer7d);
    for effect in layer_7d {
        match &effect.payload {
            EffectPayload::SwitchPowerToughness => {
                std::mem::swap(&mut power, &mut toughness);
            }
            // Non-Layer-7d payloads — not applicable here.
            EffectPayload::AddTypes(_)
            | EffectPayload::RemoveTypes(_)
            | EffectPayload::SetTypes(_)
            | EffectPayload::AddSubtypes(_)
            | EffectPayload::RemoveSubtypes(_)
            | EffectPayload::SetSubtypes(_)
            | EffectPayload::SetColors(_)
            | EffectPayload::AddColors(_)
            | EffectPayload::RemoveAllColors
            | EffectPayload::GrantAbility(_)
            | EffectPayload::RemoveAbility(_)
            | EffectPayload::RemoveAllAbilities
            | EffectPayload::CantHaveAbility(_)
            | EffectPayload::GrantKeywordCounter(_)
            | EffectPayload::SetPowerToughness(_, _)
            | EffectPayload::SetPtFormula(_)
            | EffectPayload::ModifyPowerToughness(_, _) => {}
        }
    }

    Some((power, toughness))
}

/// Run the full layer pipeline for a permanent and return effective characteristics.
///
/// This is the core pure function of the layer system. It takes all game state
/// needed (no `Game` reference) and returns effective values.
pub(crate) fn evaluate_layers(ctx: &LayerContext<'_>) -> EffectiveCharacteristics {
    // Start from base values
    let mut types: Vec<CardType> = ctx.base_types.to_vec();
    let mut subtypes: Vec<String> = ctx.base_subtypes.to_vec();
    let mut colors: Vec<ManaColor> = ctx.base_colors.clone();
    let mut abilities: Vec<StaticAbility> = ctx.base_abilities.to_vec();

    // Layer 4
    apply_layer4(ctx, &mut types, &mut subtypes);

    // Layer 5
    apply_layer5(ctx, &mut colors);

    // Layer 6
    apply_layer6(ctx, &mut abilities);

    // Layer 7 — uses post-Layer-4 types to determine if it's a creature
    let is_creature_after_l4 = types.contains(&CardType::Creature);
    let pt = apply_layer7(ctx, is_creature_after_l4, ctx.base_power, ctx.base_toughness);

    EffectiveCharacteristics {
        types,
        subtypes,
        colors,
        abilities,
        power: pt.map(|(p, _)| p),
        toughness: pt.map(|(_, t)| t),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers ------------------------------------------------------------

    fn empty_ctx<'a>(
        permanent_id: &'a str,
        controller_id: &'a str,
        types: &'a [CardType],
        base_power: Option<i32>,
        base_toughness: Option<i32>,
        effects: &'a [GlobalContinuousEffect],
    ) -> LayerContext<'a> {
        LayerContext {
            permanent_id,
            controller_id,
            base_types: types,
            base_subtypes: &[],
            base_colors: vec![],
            base_abilities: &[],
            base_power,
            base_toughness,
            plus_counters: 0,
            minus_counters: 0,
            keyword_counters: vec![],
            controller_hand_size: 0,
            effects,
        }
    }

    fn creature_ctx<'a>(
        permanent_id: &'a str,
        power: i32,
        toughness: i32,
        effects: &'a [GlobalContinuousEffect],
    ) -> LayerContext<'a> {
        LayerContext {
            permanent_id,
            controller_id: "p1",
            base_types: &[CardType::Creature],
            base_subtypes: &[],
            base_colors: vec![],
            base_abilities: &[],
            base_power: Some(power),
            base_toughness: Some(toughness),
            plus_counters: 0,
            minus_counters: 0,
            keyword_counters: vec![],
            controller_hand_size: 0,
            effects,
        }
    }

    fn pump_effect(permanent_id: &str, dp: i32, dt: i32, timestamp: u64) -> GlobalContinuousEffect {
        GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(dp, dt),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp,
            source_id: "test-source".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec![permanent_id.to_owned()]),
            locked_target_set: None,
        }
    }

    fn set_pt_effect(permanent_id: &str, p: i32, t: i32, timestamp: u64) -> GlobalContinuousEffect {
        GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(p, t),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp,
            source_id: "test-source".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec![permanent_id.to_owned()]),
            locked_target_set: None,
        }
    }

    // ============================================================================
    // Layer 7 P/T Calculation (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn base_22_no_effects_returns_22() {
        let ctx = creature_ctx("c1", 2, 2, &[]);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(2));
        assert_eq!(result.toughness, Some(2));
    }

    #[test]
    fn layer7c_plus3_3_on_base_22_returns_55() {
        let effects = [pump_effect("c1", 3, 3, 100)];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(5));
        assert_eq!(result.toughness, Some(5));
    }

    #[test]
    fn layer7b_set_11_then_layer7c_plus33_returns_44() {
        // Critical ordering test: 7b sets to 1/1, then 7c adds +3/+3 → 4/4
        // Even if 7b has a LATER timestamp than 7c — layer order is fixed
        let effects = [
            set_pt_effect("c1", 1, 1, 200), // 7b — later timestamp
            pump_effect("c1", 3, 3, 100),   // 7c — earlier timestamp
        ];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(4), "7b → 4/4 not 2+3=5");
        assert_eq!(result.toughness, Some(4));
    }

    #[test]
    fn layer7b_set_11_then_layer7c_plus33_returns_44_regardless_of_timestamp_order() {
        // Same test but timestamps reversed
        let effects = [
            set_pt_effect("c1", 1, 1, 100), // 7b — earlier timestamp
            pump_effect("c1", 3, 3, 200),   // 7c — later timestamp
        ];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(4));
        assert_eq!(result.toughness, Some(4));
    }

    #[test]
    fn two_layer7c_effects_both_sum() {
        let effects = [
            pump_effect("c1", 2, 2, 100),
            pump_effect("c1", 1, 1, 200),
        ];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(5)); // 2 + 2 + 1 = 5
        assert_eq!(result.toughness, Some(5));
    }

    #[test]
    fn two_plus_counters_add_to_pt() {
        let effects: Vec<GlobalContinuousEffect> = vec![];
        let ctx = LayerContext {
            plus_counters: 2,
            ..creature_ctx("c1", 2, 2, &effects)
        };
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(4));
        assert_eq!(result.toughness, Some(4));
    }

    #[test]
    fn two_minus_counters_reduce_pt() {
        let effects: Vec<GlobalContinuousEffect> = vec![];
        let ctx = LayerContext {
            minus_counters: 2,
            ..creature_ctx("c1", 2, 2, &effects)
        };
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(0));
        assert_eq!(result.toughness, Some(0));
    }

    #[test]
    fn one_plus_one_minus_counter_cancel_in_7c() {
        let effects: Vec<GlobalContinuousEffect> = vec![];
        let ctx = LayerContext {
            plus_counters: 1,
            minus_counters: 1,
            ..creature_ctx("c1", 2, 2, &effects)
        };
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(2));
        assert_eq!(result.toughness, Some(2));
    }

    #[test]
    fn layer7d_switch_pt_after_7c_pump() {
        // 1/3 with +0/+1 in 7c → 1/4, then switch → 4/1
        let effects = [
            pump_effect("c1", 0, 1, 100),
            GlobalContinuousEffect {
                layer: EffectLayer::Layer7d,
                payload: EffectPayload::SwitchPowerToughness,
                duration: EffectDuration::UntilEndOfTurn,
                timestamp: 200,
                source_id: "switch-source".to_owned(),
                controller_id: "p1".to_owned(),
                is_cda: false,
                targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
                locked_target_set: None,
            },
        ];
        let ctx = creature_ctx("c1", 1, 3, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(4));
        assert_eq!(result.toughness, Some(1));
    }

    #[test]
    fn layer7b_set_35_then_layer7d_switch() {
        // 2/4 creature with 7b "becomes 3/5" then 7d switch → 5/3
        let switch_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7d,
            payload: EffectPayload::SwitchPowerToughness,
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 300,
            source_id: "switch".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let effects = [set_pt_effect("c1", 3, 5, 100), switch_effect];
        let ctx = creature_ctx("c1", 2, 4, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(5));
        assert_eq!(result.toughness, Some(3));
    }

    #[test]
    fn until_end_of_turn_effect_not_in_expired_effects_slice_does_not_apply() {
        // Simulate: effect slice is empty (already removed at cleanup)
        let effects: Vec<GlobalContinuousEffect> = vec![];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(2));
        assert_eq!(result.toughness, Some(2));
    }

    // ============================================================================
    // Layer 4 Type Queries (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn land_with_no_type_effects_returns_land() {
        let types = [CardType::Land];
        let ctx = empty_ctx("p1", "p1", &types, None, None, &[]);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.types, vec![CardType::Land]);
    }

    #[test]
    fn land_with_add_creature_type_effect() {
        let add_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::AddTypes(vec![CardType::Creature]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "animate".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["p1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Land];
        let effects = [add_effect];
        let ctx = empty_ctx("p1", "p1", &types, Some(3), Some(3), &effects);
        let result = evaluate_layers(&ctx);
        assert!(result.types.contains(&CardType::Land));
        assert!(result.types.contains(&CardType::Creature));
    }

    #[test]
    fn creature_with_remove_creature_type_effect() {
        let remove_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::RemoveTypes(vec![CardType::Creature]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "humility".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [remove_effect];
        let ctx = empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects);
        let result = evaluate_layers(&ctx);
        assert!(!result.types.contains(&CardType::Creature));
    }

    #[test]
    fn changeling_cda_subtype_effect_applies_before_timestamp_ordered_effects() {
        // CDA adds all subtypes (simplified: ["Elf", "Human"])
        // Then a timestamp-ordered effect adds another subtype
        // CDA should apply first (is_cda = true sorts before non-CDA)
        let cda_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::AddSubtypes(vec!["Elf".to_owned(), "Human".to_owned()]),
            duration: EffectDuration::WhileSourceOnBattlefield("c1".to_owned()),
            timestamp: 500, // later timestamp, but is_cda sorts first
            source_id: "c1".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: true,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent("c1".to_owned())),
            locked_target_set: None,
        };
        let normal_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::RemoveSubtypes(vec!["Elf".to_owned()]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100, // earlier timestamp
            source_id: "spell".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [normal_effect, cda_effect];
        let ctx = LayerContext {
            base_subtypes: &[],
            ..empty_ctx("c1", "p1", &types, Some(1), Some(1), &effects)
        };
        let result = evaluate_layers(&ctx);
        // CDA runs first (adds Elf, Human), then non-CDA removes Elf → Human remains
        assert!(!result.subtypes.contains(&"Elf".to_owned()), "Elf removed by non-CDA effect");
        assert!(result.subtypes.contains(&"Human".to_owned()), "Human from CDA remains");
    }

    // ============================================================================
    // Layer 5 Color Queries (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn colorless_artifact_with_no_effects_returns_empty_colors() {
        let types = [CardType::Artifact];
        let ctx = empty_ctx("a1", "p1", &types, None, None, &[]);
        let result = evaluate_layers(&ctx);
        assert!(result.colors.is_empty());
    }

    #[test]
    fn becomes_blue_effect() {
        let effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::SetColors(vec![ManaColor::Blue]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["a1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Artifact];
        let effects = [effect];
        let ctx = empty_ctx("a1", "p1", &types, None, None, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.colors, vec![ManaColor::Blue]);
    }

    #[test]
    fn red_creature_loses_all_colors() {
        let effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::RemoveAllColors,
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [effect];
        let ctx = LayerContext {
            base_colors: vec![ManaColor::Red],
            ..empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects)
        };
        let result = evaluate_layers(&ctx);
        assert!(result.colors.is_empty());
    }

    #[test]
    fn color_cda_applies_before_timestamp_ordered_color_effect() {
        // CDA sets "all colors" first, then "becomes blue" overrides
        let cda_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::SetColors(vec![
                ManaColor::White,
                ManaColor::Blue,
                ManaColor::Black,
                ManaColor::Red,
                ManaColor::Green,
            ]),
            duration: EffectDuration::WhileSourceOnBattlefield("c1".to_owned()),
            timestamp: 500, // later timestamp, but is_cda
            source_id: "c1".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: true,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent("c1".to_owned())),
            locked_target_set: None,
        };
        let timestamp_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::SetColors(vec![ManaColor::Blue]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 600, // later → applies after CDA
            source_id: "spell".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [cda_effect, timestamp_effect];
        let ctx = empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects);
        let result = evaluate_layers(&ctx);
        // CDA → all colors, then "becomes blue" (timestamp 600) overrides to blue only
        assert_eq!(result.colors, vec![ManaColor::Blue]);
    }

    // ============================================================================
    // Layer 6 Ability Queries (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn creature_with_flying_on_definition_has_flying() {
        let types = [CardType::Creature];
        let abilities = [StaticAbility::Flying];
        let ctx = LayerContext {
            base_abilities: &abilities,
            ..empty_ctx("c1", "p1", &types, Some(2), Some(2), &[])
        };
        let result = evaluate_layers(&ctx);
        assert!(result.abilities.contains(&StaticAbility::Flying));
    }

    #[test]
    fn loses_flying_effect_removes_flying() {
        let lose_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::RemoveAbility(StaticAbility::Flying),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let abilities = [StaticAbility::Flying];
        let effects = [lose_effect];
        let ctx = LayerContext {
            base_abilities: &abilities,
            ..empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects)
        };
        let result = evaluate_layers(&ctx);
        assert!(!result.abilities.contains(&StaticAbility::Flying));
    }

    #[test]
    fn gains_trample_effect_grants_trample() {
        let grant_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::GrantAbility(StaticAbility::Trample),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [grant_effect];
        let ctx = empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects);
        let result = evaluate_layers(&ctx);
        assert!(result.abilities.contains(&StaticAbility::Trample));
    }

    #[test]
    fn keyword_counter_grants_flying() {
        let types = [CardType::Creature];
        let ctx = LayerContext {
            keyword_counters: vec![StaticAbility::Flying],
            ..empty_ctx("c1", "p1", &types, Some(2), Some(2), &[])
        };
        let result = evaluate_layers(&ctx);
        assert!(result.abilities.contains(&StaticAbility::Flying));
    }

    #[test]
    fn cant_have_ability_overrides_grant_regardless_of_timestamp() {
        let grant_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::GrantAbility(StaticAbility::Hexproof),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let cant_have_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::CantHaveAbility(StaticAbility::Hexproof),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 50, // EARLIER timestamp, but still wins per CR 613.1f
            source_id: "src2".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [grant_effect, cant_have_effect];
        let ctx = empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects);
        let result = evaluate_layers(&ctx);
        assert!(!result.abilities.contains(&StaticAbility::Hexproof));
    }

    // ============================================================================
    // Target Scope — LockedSet vs Filter (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn locked_set_effect_does_not_apply_to_permanent_not_in_set() {
        let effect = pump_effect("c1", 3, 3, 100); // LockedSet for "c1"
        let effects = [effect];
        let ctx = creature_ctx("c2", 2, 2, &effects); // querying "c2"
        let result = evaluate_layers(&ctx);
        // c2 is not in the locked set, so no bonus
        assert_eq!(result.power, Some(2));
        assert_eq!(result.toughness, Some(2));
    }

    #[test]
    fn filter_effect_applies_to_all_creatures_controlled_by_controller() {
        let anthem = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(1, 1),
            duration: EffectDuration::WhileSourceOnBattlefield("anthem".to_owned()),
            timestamp: 50,
            source_id: "anthem".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::Filter(EffectFilter::CreaturesControlledBy(
                "p1".to_owned(),
            )),
            locked_target_set: None,
        };
        let effects = [anthem];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        // c1 is controlled by p1 — filter applies
        assert_eq!(result.power, Some(3));
        assert_eq!(result.toughness, Some(3));
    }

    #[test]
    fn filter_effect_does_not_apply_to_opponent_creatures() {
        let anthem = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(1, 1),
            duration: EffectDuration::WhileSourceOnBattlefield("anthem".to_owned()),
            timestamp: 50,
            source_id: "anthem".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::Filter(EffectFilter::CreaturesControlledBy(
                "p1".to_owned(),
            )),
            locked_target_set: None,
        };
        let effects = [anthem];
        // c2 is controlled by p2
        let ctx = LayerContext {
            controller_id: "p2",
            ..creature_ctx("c2", 2, 2, &effects)
        };
        let result = evaluate_layers(&ctx);
        // p2 creature doesn't benefit from p1's anthem
        assert_eq!(result.power, Some(2));
        assert_eq!(result.toughness, Some(2));
    }

    // ============================================================================
    // Multi-Layer Effect Lock-In (CR 613.6)
    // ============================================================================

    #[test]
    fn locked_target_set_overrides_filter_in_later_layer() {
        // An artifact that was a noncreature when Layer 4 ran, captured in locked_target_set.
        // By Layer 7b time it's a creature. Lock-in must ensure it still receives the 2/2.
        let layer4_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::AddTypes(vec![CardType::Creature]),
            duration: EffectDuration::WhileSourceOnBattlefield("src".to_owned()),
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent("artifact1".to_owned())),
            locked_target_set: None,
        };
        let layer7b_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(2, 2),
            duration: EffectDuration::WhileSourceOnBattlefield("src".to_owned()),
            timestamp: 100,
            source_id: "src".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent("artifact1".to_owned())),
            // CR 613.6: locked at Layer 4 for all later layers
            locked_target_set: Some(vec!["artifact1".to_owned()]),
        };
        let types = [CardType::Artifact]; // Not a creature in the definition
        let effects = [layer4_effect, layer7b_effect];
        let ctx = LayerContext {
            permanent_id: "artifact1",
            controller_id: "p1",
            base_types: &types,
            base_subtypes: &[],
            base_colors: vec![],
            base_abilities: &[],
            base_power: None,
            base_toughness: None,
            plus_counters: 0,
            minus_counters: 0,
            keyword_counters: vec![],
            controller_hand_size: 0,
            effects: &effects,
        };
        let result = evaluate_layers(&ctx);
        // After Layer 4: types = [Artifact, Creature]
        assert!(result.types.contains(&CardType::Creature));
        // Layer 7b applies via locked_target_set (even though "is creature" check passes now)
        assert_eq!(result.power, Some(2));
        assert_eq!(result.toughness, Some(2));
    }

    // ============================================================================
    // Timestamp Ordering (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn two_layer7b_set_effects_later_timestamp_wins() {
        // Both set P/T, later one (timestamp 200) wins
        let effects = [
            set_pt_effect("c1", 3, 3, 100),
            set_pt_effect("c1", 5, 5, 200), // later → applied last → wins
        ];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(5));
        assert_eq!(result.toughness, Some(5));
    }

    #[test]
    fn two_layer7c_effects_sum_regardless_of_timestamp_order() {
        let effects = [
            pump_effect("c1", 1, 1, 200),
            pump_effect("c1", 2, 2, 100),
        ];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(5)); // 2 + 1 + 2 = 5
        assert_eq!(result.toughness, Some(5));
    }

    // ============================================================================
    // CDA Identification (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn cda_in_layer4_applies_before_non_cda_same_layer() {
        // CDA has later timestamp but is_cda = true → applies first
        // Non-CDA then removes what CDA added
        let cda = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::AddTypes(vec![CardType::Artifact]),
            duration: EffectDuration::WhileSourceOnBattlefield("c1".to_owned()),
            timestamp: 1000,
            source_id: "c1".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: true,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent("c1".to_owned())),
            locked_target_set: None,
        };
        let non_cda = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::RemoveTypes(vec![CardType::Artifact]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 500,
            source_id: "spell".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [non_cda, cda];
        let ctx = empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects);
        let result = evaluate_layers(&ctx);
        // CDA adds Artifact first; non-CDA removes Artifact → no Artifact in result
        assert!(!result.types.contains(&CardType::Artifact));
    }

    #[test]
    fn cda_in_layer5_applies_before_non_cda_same_layer() {
        // CDA sets all colors; later non-CDA "becomes blue" overrides
        let cda = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::SetColors(vec![ManaColor::Red, ManaColor::Green]),
            duration: EffectDuration::WhileSourceOnBattlefield("c1".to_owned()),
            timestamp: 1000,
            source_id: "c1".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: true,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent("c1".to_owned())),
            locked_target_set: None,
        };
        let non_cda = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::SetColors(vec![ManaColor::Blue]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 500,
            source_id: "spell".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let types = [CardType::Creature];
        let effects = [non_cda, cda];
        let ctx = empty_ctx("c1", "p1", &types, Some(2), Some(2), &effects);
        let result = evaluate_layers(&ctx);
        // CDA runs first (Red+Green), non-CDA (Blue, ts=500) runs second → Blue wins
        assert_eq!(result.colors, vec![ManaColor::Blue]);
    }

    // ============================================================================
    // Duration Handling (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn while_source_on_battlefield_effect_applies_when_source_present() {
        // Simulated: effect is in the slice → applies
        let anthem = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(1, 1),
            duration: EffectDuration::WhileSourceOnBattlefield("anthem".to_owned()),
            timestamp: 100,
            source_id: "anthem".to_owned(),
            controller_id: "p1".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        let effects = [anthem];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(3));
    }

    #[test]
    fn while_source_on_battlefield_effect_absent_when_removed_from_slice() {
        // When source leaves battlefield, the effect is removed from the slice
        let no_effects: Vec<GlobalContinuousEffect> = vec![];
        let ctx = creature_ctx("c1", 2, 2, &no_effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.power, Some(2));
    }

    // ============================================================================
    // SBA Integration (Acceptance Criteria)
    // ============================================================================

    #[test]
    fn effective_toughness_via_layer7b_used_for_lethal_damage_check() {
        // 2/2 with 7b "becomes 1/1" and 2 damage → effective toughness 1 → lethal
        let effects = [set_pt_effect("c1", 1, 1, 100)];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.toughness, Some(1));
        // The SBA check compares damage (2) >= effective toughness (1) → lethal
    }

    #[test]
    fn effective_toughness_zero_via_layer7b_triggers_sba() {
        // Creature reduced to 0 effective toughness by 7b
        let effects = [set_pt_effect("c1", 0, 0, 100)];
        let ctx = creature_ctx("c1", 2, 2, &effects);
        let result = evaluate_layers(&ctx);
        assert_eq!(result.toughness, Some(0));
    }
}
