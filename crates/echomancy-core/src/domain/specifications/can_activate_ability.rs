//! CanActivateAbility specification — checks if a player has at least one
//! permanent whose activated ability can currently be activated.
//!
//! Supports all three cost variants (CR 602.1):
//! - `Tap`: permanent must be untapped, and no summoning sickness for creatures.
//! - `TapAndMana`: same tap checks plus the mana pool must be able to pay the cost.
//! - `Mana`: mana pool must be able to pay the cost (no tap required).

use crate::domain::abilities::{ActivatedAbility, ActivationCost};
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::StaticAbility;
use crate::domain::errors::GameError;
use crate::domain::services::mana_payment::can_pay_cost;
use crate::domain::types::PlayerId;
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;

/// Read-only context interface for ability-activation validation.
///
/// The Game aggregate will implement this; in tests we use a minimal struct.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) trait ActivateAbilityContext {
    /// Returns all permanents the given player controls on the battlefield.
    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance];

    /// Returns the `PermanentState` for the given instance ID, if present.
    fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState>;

    /// Returns `true` if the permanent with the given instance ID has summoning
    /// sickness (CR 302.6: creature has been under your control since the start
    /// of your most recent turn).
    fn has_summoning_sickness(&self, instance_id: &str) -> bool;

    /// Returns `true` if the permanent with the given instance ID has the given
    /// static ability on its card definition.
    fn has_static_ability(&self, instance_id: &str, ability: StaticAbility) -> bool;

    /// Returns the mana pool for the given player, if they exist.
    fn mana_pool(&self, player_id: &str) -> Option<&ManaPool>;
}

/// Context required to evaluate the `CanActivateAbility` specification.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CanActivateAbilityCtx<'a> {
    /// The player whose permanents are being checked.
    pub(crate) player_id: &'a str,
}

/// Returns `true` if the tap portion of an activation cost can be paid for the
/// given permanent: it must be untapped and not have summoning sickness (unless
/// it has Haste).
fn can_pay_tap_cost<C: ActivateAbilityContext>(
    player_id: &str,
    instance_id: &str,
    game_ctx: &C,
) -> bool {
    let not_tapped = match game_ctx.permanent_state(instance_id) {
        Some(state) => !state.is_tapped(),
        None => return false,
    };
    if !not_tapped {
        return false;
    }

    let has_sickness = game_ctx.has_summoning_sickness(instance_id);
    let has_haste = game_ctx.has_static_ability(instance_id, StaticAbility::Haste);
    if has_sickness && !has_haste {
        return false;
    }

    // Also verify the permanent belongs to the player (belt-and-suspenders check).
    game_ctx
        .battlefield_cards(player_id)
        .iter()
        .any(|c| c.instance_id() == instance_id)
}

/// Returns `true` if the given ability's cost can be paid right now for the
/// given permanent.
fn can_pay_ability_cost<C: ActivateAbilityContext>(
    player_id: &str,
    instance_id: &str,
    ability: &ActivatedAbility,
    game_ctx: &C,
) -> bool {
    match &ability.cost {
        ActivationCost::Tap => {
            can_pay_tap_cost(player_id, instance_id, game_ctx)
        }
        ActivationCost::TapAndMana(mana_cost) => {
            if !can_pay_tap_cost(player_id, instance_id, game_ctx) {
                return false;
            }
            match game_ctx.mana_pool(player_id) {
                Some(pool) => can_pay_cost(pool, mana_cost),
                None => false,
            }
        }
        ActivationCost::Mana(mana_cost) => {
            match game_ctx.mana_pool(player_id) {
                Some(pool) => can_pay_cost(pool, mana_cost),
                None => false,
            }
        }
    }
}

/// Returns `Ok(())` if the player has at least one permanent with an
/// activated ability that can currently be paid.
///
/// # Errors
///
/// Returns `GameError::NoActivatableAbility` if no permanent has an
/// activatable ability at this moment.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_satisfied<C: ActivateAbilityContext>(
    ctx: &CanActivateAbilityCtx<'_>,
    game_ctx: &C,
) -> Result<(), GameError> {
    let battlefield = game_ctx.battlefield_cards(ctx.player_id);

    let has_activatable = battlefield.iter().any(|card| {
        // Card must have at least one activatable ability.
        card.definition()
            .activated_abilities()
            .iter()
            .any(|ability| can_pay_ability_cost(ctx.player_id, card.instance_id(), ability, game_ctx))
    });

    if has_activatable {
        Ok(())
    } else {
        Err(GameError::NoActivatableAbility {
            player_id: PlayerId::new(ctx.player_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::abilities::ActivatedAbility;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::effects::Effect;
    use crate::domain::enums::CardType;
    use crate::domain::value_objects::mana::ManaPool;
    use crate::domain::value_objects::permanent_state::PermanentState;
    use std::collections::HashMap;

    // ---- minimal test context ---------------------------------------------

    struct TestCtx {
        cards: HashMap<String, Vec<CardInstance>>,
        states: HashMap<String, PermanentState>,
        /// instance_id → has summoning sickness
        summoning_sickness: HashMap<String, bool>,
        /// instance_id → list of static abilities
        static_abilities: HashMap<String, Vec<StaticAbility>>,
        /// player_id → mana pool
        mana_pools: HashMap<String, ManaPool>,
    }

    impl TestCtx {
        fn new() -> Self {
            TestCtx {
                cards: HashMap::new(),
                states: HashMap::new(),
                summoning_sickness: HashMap::new(),
                static_abilities: HashMap::new(),
                mana_pools: HashMap::new(),
            }
        }

        #[allow(dead_code)]
        fn with_mana_pool(mut self, player_id: &str, pool: ManaPool) -> Self {
            self.mana_pools.insert(player_id.to_owned(), pool);
            self
        }

        fn add_permanent(mut self, player: &str, card: CardInstance, state: PermanentState) -> Self {
            let id = card.instance_id().to_owned();
            self.cards
                .entry(player.to_owned())
                .or_default()
                .push(card.clone());
            self.states.insert(id.clone(), state);
            // Default: no summoning sickness, no extra static abilities
            self.summoning_sickness.entry(id).or_insert(false);
            self
        }

        fn with_summoning_sickness(mut self, instance_id: &str, value: bool) -> Self {
            self.summoning_sickness.insert(instance_id.to_owned(), value);
            self
        }

        #[allow(dead_code)]
        fn with_static_ability(mut self, instance_id: &str, ability: StaticAbility) -> Self {
            self.static_abilities
                .entry(instance_id.to_owned())
                .or_default()
                .push(ability);
            self
        }
    }

    impl ActivateAbilityContext for TestCtx {
        fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
            self.cards
                .get(player_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        }

        fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
            self.states.get(instance_id)
        }

        fn has_summoning_sickness(&self, instance_id: &str) -> bool {
            self.summoning_sickness
                .get(instance_id)
                .copied()
                .unwrap_or(false)
        }

        fn has_static_ability(&self, instance_id: &str, ability: StaticAbility) -> bool {
            // Also check the card definition for static abilities.
            let in_map = self
                .static_abilities
                .get(instance_id)
                .map(|v| v.contains(&ability))
                .unwrap_or(false);
            // Check card definition abilities too.
            let in_def = self
                .cards
                .values()
                .flat_map(|cards| cards.iter())
                .find(|c| c.instance_id() == instance_id)
                .map(|c| c.definition().has_static_ability(ability))
                .unwrap_or(false);
            in_map || in_def
        }

        fn mana_pool(&self, player_id: &str) -> Option<&ManaPool> {
            self.mana_pools.get(player_id)
        }
    }

    // ---- card helpers -------------------------------------------------------

    fn make_creature_with_tap_ability(id: &str, owner: &str) -> CardInstance {
        let ability = ActivatedAbility::tap_ability(Effect::draw_cards(1));
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_activated_ability(ability);
        CardInstance::new(id, def, owner)
    }

    fn make_creature_with_tap_ability_and_haste(id: &str, owner: &str) -> CardInstance {
        let ability = ActivatedAbility::tap_ability(Effect::draw_cards(1));
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_activated_ability(ability)
            .with_static_ability(StaticAbility::Haste);
        CardInstance::new(id, def, owner)
    }

    fn make_creature_without_ability(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(2, 2);
        CardInstance::new(id, def, owner)
    }

    fn untapped_creature_state() -> PermanentState {
        PermanentState::for_creature(1, 1)
            .with_summoning_sickness(false)
            .unwrap()
    }

    fn tapped_creature_state() -> PermanentState {
        untapped_creature_state().with_tapped(true)
    }

    fn spec_ctx(player: &str) -> CanActivateAbilityCtx<'_> {
        CanActivateAbilityCtx { player_id: player }
    }

    // ---- happy path --------------------------------------------------------

    #[test]
    fn untapped_permanent_with_tap_ability_is_activatable() {
        let card = make_creature_with_tap_ability("c1", "p1");
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, untapped_creature_state());

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_ok());
    }

    // ---- tapped permanent -------------------------------------------------

    #[test]
    fn tapped_permanent_with_tap_ability_is_not_activatable() {
        let card = make_creature_with_tap_ability("c1", "p1");
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, tapped_creature_state());

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_err());
    }

    // ---- no activated ability ---------------------------------------------

    #[test]
    fn permanent_without_activated_ability_returns_err() {
        let card = make_creature_without_ability("c1", "p1");
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, untapped_creature_state());

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_err());
    }

    // ---- empty battlefield ------------------------------------------------

    #[test]
    fn empty_battlefield_returns_err() {
        let game_ctx = TestCtx::new();
        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_err());
    }

    // ---- error variant ----------------------------------------------------

    #[test]
    fn error_variant_is_no_activatable_ability() {
        let game_ctx = TestCtx::new();
        let ctx = spec_ctx("p1");
        let err = is_satisfied(&ctx, &game_ctx).unwrap_err();
        assert!(matches!(
            err,
            GameError::NoActivatableAbility { player_id } if player_id.as_str() == "p1"
        ));
    }

    // ---- mix of activatable and non-activatable ---------------------------

    #[test]
    fn one_activatable_among_non_activatable_returns_ok() {
        let no_ability = make_creature_without_ability("c1", "p1");
        let tapped_with_ability = make_creature_with_tap_ability("c2", "p1");
        let untapped_with_ability = make_creature_with_tap_ability("c3", "p1");

        let game_ctx = TestCtx::new()
            .add_permanent("p1", no_ability, untapped_creature_state())
            .add_permanent("p1", tapped_with_ability, tapped_creature_state())
            .add_permanent("p1", untapped_with_ability, untapped_creature_state());

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_ok());
    }

    #[test]
    fn all_tap_ability_permanents_tapped_returns_err() {
        let c1 = make_creature_with_tap_ability("c1", "p1");
        let c2 = make_creature_with_tap_ability("c2", "p1");

        let game_ctx = TestCtx::new()
            .add_permanent("p1", c1, tapped_creature_state())
            .add_permanent("p1", c2, tapped_creature_state());

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_err());
    }

    // ---- summoning sickness (CR 302.6) ------------------------------------

    #[test]
    fn creature_with_summoning_sickness_and_tap_ability_is_not_activatable() {
        let card = make_creature_with_tap_ability("c1", "p1");
        let state = PermanentState::for_creature(1, 1); // has_summoning_sickness defaults to true
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, state)
            .with_summoning_sickness("c1", true);

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_err());
    }

    #[test]
    fn creature_with_summoning_sickness_but_haste_can_activate_tap_ability() {
        let card = make_creature_with_tap_ability_and_haste("c1", "p1");
        let state = PermanentState::for_creature(1, 1); // has summoning sickness
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, state)
            .with_summoning_sickness("c1", true);

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_ok());
    }

    #[test]
    fn creature_without_summoning_sickness_can_activate_tap_ability() {
        let card = make_creature_with_tap_ability("c1", "p1");
        let state = untapped_creature_state(); // summoning sickness cleared
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, state)
            .with_summoning_sickness("c1", false);

        let ctx = spec_ctx("p1");
        assert!(is_satisfied(&ctx, &game_ctx).is_ok());
    }

    #[test]
    fn error_contains_player_id_when_sickness_blocks_all() {
        let card = make_creature_with_tap_ability("c1", "p1");
        let state = PermanentState::for_creature(1, 1);
        let game_ctx = TestCtx::new()
            .add_permanent("p1", card, state)
            .with_summoning_sickness("c1", true);

        let ctx = spec_ctx("p1");
        let err = is_satisfied(&ctx, &game_ctx).unwrap_err();
        assert!(matches!(
            err,
            GameError::NoActivatableAbility { player_id } if player_id.as_str() == "p1"
        ));
    }
}
