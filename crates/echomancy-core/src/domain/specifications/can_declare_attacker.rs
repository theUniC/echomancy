//! CanDeclareAttacker specification — checks if a player has at least one
//! creature eligible to be declared as an attacker.
//!
//! This is a thin wrapper that delegates per-creature validation to the
//! `combat_declarations::validate_declare_attacker` service.  The spec
//! answers the action-window question: "does the player have *any* creature
//! that *could* attack?", not "is a specific creature allowed to attack?".
//!
//! Conditions (all must hold):
//! 1. The current step is `DeclareAttackers`.
//! 2. The player is the active (current) player.
//! 3. At least one creature they control can legally be declared as attacker.
//!
//! Mirrors the TypeScript `CanDeclareAttacker` class from
//! `game/specifications/CanDeclareAttacker.ts`.

use crate::domain::enums::Step;
use crate::domain::errors::GameError;
use crate::domain::services::combat_declarations::{CombatValidationContext, validate_declare_attacker};
use crate::domain::types::PlayerId;

/// Context required to evaluate the `CanDeclareAttacker` specification.
///
/// Any type that implements `CombatValidationContext` may be used here,
/// which allows the Game aggregate to pass itself directly without wrapping.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CanDeclareAttackerCtx<'a, C: CombatValidationContext> {
    /// The validation context used to look up battlefield cards and states.
    pub(crate) ctx: &'a C,
    /// The ID of the player whose action window is being evaluated.
    pub(crate) player_id: &'a str,
}

/// Returns `Ok(())` if the player has at least one creature that can legally
/// be declared as an attacker in the current game state.
///
/// # Errors
///
/// - `GameError::InvalidPlayerAction` — wrong step or not the active player.
/// - `GameError::InvalidPlayerAction` — active player has no attackable creature.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_satisfied<C: CombatValidationContext>(
    ctx: &CanDeclareAttackerCtx<'_, C>,
) -> Result<(), GameError> {
    // 1. Must be the DECLARE_ATTACKERS step.
    if ctx.ctx.current_step() != Step::DeclareAttackers {
        return Err(GameError::InvalidPlayerAction {
            player_id: PlayerId::new(ctx.player_id),
            action: "DECLARE_ATTACKER".to_owned(),
        });
    }

    // 2. Must be the active player.
    if ctx.player_id != ctx.ctx.current_player_id() {
        return Err(GameError::InvalidPlayerAction {
            player_id: PlayerId::new(ctx.player_id),
            action: "DECLARE_ATTACKER".to_owned(),
        });
    }

    // 3. Must have at least one creature that can legally attack.
    let battlefield = ctx.ctx.battlefield_cards(ctx.player_id);
    let has_attackable = battlefield.iter().any(|card| {
        // Only consider creature cards.
        if !ctx.ctx.is_creature(card) {
            return false;
        }
        // Delegate to the full validation service for this creature.
        validate_declare_attacker(ctx.ctx, ctx.player_id, card.instance_id()).is_ok()
    });

    if has_attackable {
        Ok(())
    } else {
        Err(GameError::InvalidPlayerAction {
            player_id: PlayerId::new(ctx.player_id),
            action: "DECLARE_ATTACKER".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::{CardType, StaticAbility, Step};
    use crate::domain::services::combat_declarations::CombatValidationContext;
    use crate::domain::value_objects::permanent_state::PermanentState;
    use std::collections::HashMap;

    // ---- minimal test context (mirrors the one in combat_declarations tests) --

    struct TestCtx {
        step: Step,
        current_player: String,
        players: HashMap<String, Vec<CardInstance>>,
        permanent_states: HashMap<String, PermanentState>,
    }

    impl TestCtx {
        fn new(step: Step, current_player: &str) -> Self {
            TestCtx {
                step,
                current_player: current_player.to_owned(),
                players: HashMap::new(),
                permanent_states: HashMap::new(),
            }
        }

        fn add_permanent(mut self, player: &str, card: CardInstance, state: PermanentState) -> Self {
            self.players
                .entry(player.to_owned())
                .or_default()
                .push(card.clone());
            self.permanent_states
                .insert(card.instance_id().to_owned(), state);
            self
        }
    }

    impl CombatValidationContext for TestCtx {
        fn current_step(&self) -> Step {
            self.step
        }

        fn current_player_id(&self) -> &str {
            &self.current_player
        }

        fn opponent_of(&self, player_id: &str) -> &str {
            for key in self.players.keys() {
                if key != player_id {
                    return key.as_str();
                }
            }
            if player_id == "p1" { "p2" } else { "p1" }
        }

        fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
            self.players
                .get(player_id)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        }

        fn is_creature(&self, card: &CardInstance) -> bool {
            card.definition().is_creature()
        }

        fn has_static_ability(&self, card: &CardInstance, ability: StaticAbility) -> bool {
            card.definition().has_static_ability(ability)
        }

        fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
            self.permanent_states.get(instance_id)
        }
    }

    fn make_creature(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(2, 2);
        CardInstance::new(id, def, owner)
    }

    fn make_land(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Land]);
        CardInstance::new(id, def, owner)
    }

    fn ready_creature_state() -> PermanentState {
        PermanentState::for_creature(2, 2)
            .with_summoning_sickness(false)
            .unwrap()
    }

    // ---- happy path --------------------------------------------------------

    #[test]
    fn active_player_with_untapped_creature_can_attack() {
        let creature = make_creature("c1", "p1");
        let state = ready_creature_state();
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", creature, state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_ok());
    }

    // ---- wrong step --------------------------------------------------------

    #[test]
    fn wrong_step_returns_invalid_player_action() {
        let creature = make_creature("c1", "p1");
        let state = ready_creature_state();
        let inner_ctx = TestCtx::new(Step::FirstMain, "p1")
            .add_permanent("p1", creature, state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        let err = is_satisfied(&spec_ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    // ---- not active player -------------------------------------------------

    #[test]
    fn non_active_player_cannot_declare_attackers() {
        let creature = make_creature("c1", "p1");
        let state = ready_creature_state();
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p2")
            .add_permanent("p1", creature, state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        let err = is_satisfied(&spec_ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    // ---- no attackable creatures -------------------------------------------

    #[test]
    fn no_creatures_on_battlefield_returns_err() {
        let land = make_land("l1", "p1");
        let land_state = PermanentState::for_non_creature();
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", land, land_state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_err());
    }

    #[test]
    fn empty_battlefield_returns_err() {
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1");
        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_err());
    }

    #[test]
    fn tapped_creature_cannot_attack() {
        let creature = make_creature("c1", "p1");
        // ready_creature_state with tapped=true
        let state = ready_creature_state().with_tapped(true);
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", creature, state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_err());
    }

    #[test]
    fn creature_with_summoning_sickness_cannot_attack() {
        let creature = make_creature("c1", "p1");
        // Default creature state has summoning sickness.
        let state = PermanentState::for_creature(2, 2);
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", creature, state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_err());
    }

    #[test]
    fn already_attacked_creature_cannot_attack_again() {
        let creature = make_creature("c1", "p1");
        let state = ready_creature_state()
            .with_has_attacked_this_turn(true)
            .unwrap();
        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", creature, state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_err());
    }

    // --- one attackable among unattackable -----------------------------------

    #[test]
    fn one_eligible_creature_among_ineligible_ones_returns_ok() {
        let tapped = make_creature("c1", "p1");
        let ready = make_creature("c2", "p1");

        let tapped_state = ready_creature_state().with_tapped(true);
        let ready_state = ready_creature_state();

        let inner_ctx = TestCtx::new(Step::DeclareAttackers, "p1")
            .add_permanent("p1", tapped, tapped_state)
            .add_permanent("p1", ready, ready_state);

        let spec_ctx = CanDeclareAttackerCtx {
            ctx: &inner_ctx,
            player_id: "p1",
        };
        assert!(is_satisfied(&spec_ctx).is_ok());
    }
}
