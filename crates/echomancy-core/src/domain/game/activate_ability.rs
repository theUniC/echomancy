//! ActivateAbility handler — activate a permanent's activated ability.
//!
//! Mana abilities (per MTG CR 605) bypass the stack entirely: when the ability's
//! effect is `AddMana`, the permanent is tapped, mana is added immediately, and
//! priority is NOT passed to the opponent (the activating player retains priority).

use crate::domain::abilities::ActivationCost;
use crate::domain::effects::Effect;
use crate::domain::entities::the_stack::{AbilityKind, AbilityOnStack, StackItem};
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::types::{CardInstanceId, PlayerId};

use super::Game;

/// Handle the `ActivateAbility` action.
///
/// # Rules
///
/// 1. Player must have priority.
/// 2. The permanent must be on the player's battlefield.
/// 3. The permanent must have an activated ability.
/// 4. The cost must be payable (MVP: only `{T}` supported).
///    - For `{T}`: permanent must not be tapped.
///    - For `{T}` on a creature: no summoning sickness (unless Haste).
///
/// After validation, the ability is placed on the stack and priority passes
/// to the opponent.
///
/// # Errors
///
/// Various `GameError` variants for each validation failure.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    permanent_id: &str,
    ability_index: usize,
) -> Result<Vec<GameEvent>, GameError> {
    // 1. Player must have priority
    if !game.has_priority(player_id) {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "ACTIVATE_ABILITY".to_owned(),
        });
    }

    // 2. Permanent must be on player's battlefield
    let (permanent_card, ability) = {
        let player = game.player_state(player_id)?;
        let card = player
            .battlefield
            .iter()
            .find(|c| c.instance_id() == permanent_id)
            .cloned()
            .ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            })?;

        // 3. Must have an activated ability at the requested index
        let ability = card
            .definition()
            .activated_ability_at(ability_index)
            .cloned()
            .ok_or_else(|| GameError::PermanentHasNoActivatedAbility {
                permanent_id: CardInstanceId::new(permanent_id),
            })?;

        (card, ability)
    };

    // 4. Pay the activation cost
    pay_activation_cost(game, player_id, permanent_id, &permanent_card, &ability.cost)?;

    // Per MTG CR 605, mana abilities resolve immediately without using the stack.
    // The activating player retains priority after a mana ability resolves.
    if ability.effect.is_mana_ability() {
        let events = resolve_mana_ability_immediately(game, player_id, &ability.effect)?;
        return Ok(events);
    }

    // Non-mana abilities go on the stack; priority passes to the opponent.
    game.push_stack(StackItem::Ability(AbilityOnStack {
        source_id: permanent_id.to_owned(),
        effect: ability.effect,
        controller_id: player_id.to_owned(),
        targets: Vec::new(),
        kind: AbilityKind::Activated,
    }));

    // Give priority to opponent
    let events = game.give_priority_to_opponent_of(player_id);
    Ok(events)
}

/// Resolve a mana ability immediately, bypassing the stack (MTG CR 605).
///
/// Adds mana to the controller's pool and emits a `ManaAdded` event.
/// The activating player retains priority after a mana ability.
fn resolve_mana_ability_immediately(
    game: &mut Game,
    player_id: &str,
    effect: &Effect,
) -> Result<Vec<GameEvent>, GameError> {
    let Effect::AddMana { color, amount } = effect else {
        // Only AddMana is a mana ability; other effects should not reach here.
        return Ok(Vec::new());
    };

    game.add_mana_to_pool(player_id, *color, *amount)?;

    Ok(vec![GameEvent::ManaAdded {
        player_id: PlayerId::new(player_id),
        color: *color,
        amount: *amount,
    }])
}

/// Pay the activation cost for an activated ability (CR 602.2).
///
/// Handles all three cost variants:
/// - `Tap`: tap the permanent (checks summoning sickness for creatures).
/// - `TapAndMana`: tap the permanent and deduct mana from the player's pool.
/// - `Mana`: deduct mana from the player's pool (no tap).
fn pay_activation_cost(
    game: &mut Game,
    player_id: &str,
    permanent_id: &str,
    permanent_card: &crate::domain::cards::card_instance::CardInstance,
    cost: &ActivationCost,
) -> Result<(), GameError> {
    match cost {
        ActivationCost::Tap => {
            pay_tap_cost(game, permanent_id, permanent_card)?;
        }
        ActivationCost::TapAndMana(mana_cost) => {
            // Tap first, then pay mana. If either fails, the whole activation fails.
            pay_tap_cost(game, permanent_id, permanent_card)?;
            pay_mana_activation_cost(game, player_id, mana_cost)?;
        }
        ActivationCost::Mana(mana_cost) => {
            pay_mana_activation_cost(game, player_id, mana_cost)?;
        }
    }
    Ok(())
}

/// Pay the tap portion of an activation cost.
fn pay_tap_cost(
    game: &mut Game,
    permanent_id: &str,
    permanent_card: &crate::domain::cards::card_instance::CardInstance,
) -> Result<(), GameError> {
    let state = game
        .permanent_state(permanent_id)
        .ok_or_else(|| GameError::PermanentNotFound {
            permanent_id: CardInstanceId::new(permanent_id),
        })?
        .clone();

    if state.is_tapped() {
        return Err(GameError::PermanentAlreadyTapped {
            permanent_id: CardInstanceId::new(permanent_id),
        });
    }

    // Check summoning sickness for creatures (CR 302.6).
    if let Some(cs) = state.creature_state() {
        if cs.has_summoning_sickness()
            && !permanent_card
                .definition()
                .has_static_ability(crate::domain::enums::StaticAbility::Haste)
        {
            return Err(GameError::CreatureHasSummoningSickness {
                creature_id: CardInstanceId::new(permanent_id),
            });
        }
    }

    game.tap_permanent(permanent_id)?;
    Ok(())
}

/// Pay the mana portion of an activation cost using the auto-pay algorithm.
fn pay_mana_activation_cost(
    game: &mut Game,
    player_id: &str,
    mana_cost: &crate::domain::value_objects::mana::ManaCost,
) -> Result<(), GameError> {
    use crate::domain::services::mana_payment::pay_cost;

    let player = game.player_state_mut(player_id)?;
    let new_pool = pay_cost(player.mana_pool.clone(), mana_cost)
        .map_err(|e| GameError::InsufficientManaForSpell {
            message: e.to_string(),
        })?;
    player.mana_pool = new_pool;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::abilities::{ActivatedAbility, ActivationCost};
    use crate::domain::actions::Action;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::effects::Effect;
    use crate::domain::enums::{CardType, ManaColor};
    use crate::domain::game::test_helpers::{
        add_permanent_to_battlefield, clear_summoning_sickness, make_game_in_first_main,
    };
    use crate::domain::types::{CardInstanceId, PlayerId};

    fn make_tap_ability_land(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("sol-ring", "Sol Ring", vec![CardType::Land])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::NoOp,
            });
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_mana_land(instance_id: &str, owner_id: &str, color: ManaColor) -> CardInstance {
        let def = CardDefinition::new("forest", "Forest", vec![CardType::Land])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::AddMana { color, amount: 1 },
            });
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_tap_ability_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("tapper", "Tapper", vec![CardType::Creature])
            .with_power_toughness(1, 1)
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::NoOp,
            });
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn activate_tap_ability_taps_the_permanent() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_tap_ability_land("land-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("land-1"),
            ability_index: 0,
        })
        .unwrap();

        let state = game.permanent_state("land-1").unwrap();
        assert!(state.is_tapped());
    }

    #[test]
    fn activate_tap_ability_puts_ability_on_stack() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_tap_ability_land("land-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("land-1"),
            ability_index: 0,
        })
        .unwrap();

        assert_eq!(game.stack().len(), 1);
        assert!(matches!(game.stack()[0], crate::domain::entities::the_stack::StackItem::Ability(_)));
    }

    #[test]
    fn already_tapped_permanent_cannot_activate_tap_ability() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_tap_ability_land("land-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);
        game.tap_permanent("land-1").unwrap();

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("land-1"),
            ability_index: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::PermanentAlreadyTapped { .. }));
    }

    #[test]
    fn creature_with_summoning_sickness_cannot_activate_tap_ability() {
        let (mut game, p1, _) = make_game_in_first_main();
        let creature = make_tap_ability_creature("tapper-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        // Has summoning sickness (just added to battlefield)

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("tapper-1"),
            ability_index: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CreatureHasSummoningSickness { .. }));
    }

    #[test]
    fn creature_without_summoning_sickness_can_activate_tap_ability() {
        let (mut game, p1, _) = make_game_in_first_main();
        let creature = make_tap_ability_creature("tapper-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, "tapper-1");

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("tapper-1"),
            ability_index: 0,
        })
        .unwrap();

        let state = game.permanent_state("tapper-1").unwrap();
        assert!(state.is_tapped());
    }

    #[test]
    fn permanent_not_on_battlefield_returns_error() {
        let (mut game, p1, _) = make_game_in_first_main();

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("nonexistent"),
            ability_index: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::PermanentNotFound { .. }));
    }

    #[test]
    fn permanent_without_activated_ability_returns_error() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = {
            let def =
                CardDefinition::new("forest", "Forest", vec![CardType::Land]);
            CardInstance::new("land-1", def, &p1)
        };
        add_permanent_to_battlefield(&mut game, &p1, land);

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("land-1"),
            ability_index: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::PermanentHasNoActivatedAbility { .. }));
    }

    #[test]
    fn player_without_priority_cannot_activate_ability() {
        let (mut game, _, p2) = make_game_in_first_main();
        let land = make_tap_ability_land("land-1", &p2);
        add_permanent_to_battlefield(&mut game, &p2, land);

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p2),
                permanent_id: CardInstanceId::new("land-1"),
            ability_index: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }

    // ---- Mana ability (CR 605) tests ----------------------------------------

    #[test]
    fn tapping_forest_adds_green_mana_to_pool() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_mana_land("forest-1", &p1, ManaColor::Green);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("forest-1"),
            ability_index: 0,
        })
        .unwrap();

        assert_eq!(game.mana_pool(&p1).unwrap().get(ManaColor::Green), 1);
    }

    #[test]
    fn mana_ability_does_not_use_the_stack() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_mana_land("forest-1", &p1, ManaColor::Green);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("forest-1"),
            ability_index: 0,
        })
        .unwrap();

        assert!(
            game.stack().is_empty(),
            "Mana abilities should resolve immediately and not use the stack"
        );
    }

    #[test]
    fn mana_ability_activating_player_retains_priority() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_mana_land("forest-1", &p1, ManaColor::Green);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("forest-1"),
            ability_index: 0,
        })
        .unwrap();

        assert_eq!(
            game.priority_player_id(),
            Some(p1.as_str()),
            "Activating player should retain priority after a mana ability"
        );
    }

    #[test]
    fn tapping_land_with_mana_ability_taps_the_permanent() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_mana_land("forest-1", &p1, ManaColor::Green);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("forest-1"),
            ability_index: 0,
        })
        .unwrap();

        let state = game.permanent_state("forest-1").unwrap();
        assert!(state.is_tapped(), "Land should be tapped after activating mana ability");
    }

    #[test]
    fn mana_ability_emits_mana_added_event() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_mana_land("forest-1", &p1, ManaColor::Green);
        add_permanent_to_battlefield(&mut game, &p1, land);

        let events = game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("forest-1"),
            ability_index: 0,
        })
        .unwrap();

        let mana_event = events.iter().find(|e| {
            matches!(e, GameEvent::ManaAdded { color: ManaColor::Green, amount: 1, .. })
        });
        assert!(mana_event.is_some(), "Should emit a ManaAdded event");
    }

    #[test]
    fn tapping_mountain_adds_red_mana() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_mana_land("mountain-1", &p1, ManaColor::Red);
        add_permanent_to_battlefield(&mut game, &p1, land);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("mountain-1"),
            ability_index: 0,
        })
        .unwrap();

        assert_eq!(game.mana_pool(&p1).unwrap().get(ManaColor::Red), 1);
    }

    // ---- Mana-only cost tests -----------------------------------------------

    fn make_mana_cost_ability(
        instance_id: &str,
        owner_id: &str,
        cost_str: &str,
    ) -> CardInstance {
        use crate::domain::value_objects::mana::ManaCost;
        let def = CardDefinition::new("pump-artifact", "Pump Artifact", vec![CardType::Artifact])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Mana(ManaCost::parse(cost_str).unwrap()),
                effect: Effect::NoOp,
            });
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_tap_and_mana_ability(
        instance_id: &str,
        owner_id: &str,
        cost_str: &str,
    ) -> CardInstance {
        use crate::domain::value_objects::mana::ManaCost;
        let def =
            CardDefinition::new("equipment", "Equipment", vec![CardType::Artifact])
                .with_activated_ability(ActivatedAbility {
                    cost: ActivationCost::TapAndMana(ManaCost::parse(cost_str).unwrap()),
                    effect: Effect::NoOp,
                });
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn activate_mana_cost_ability_succeeds_when_pool_has_enough() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Give player 2 generic mana.
        game.add_mana_to_pool(&p1, ManaColor::Colorless, 2).unwrap();
        let artifact = make_mana_cost_ability("artifact-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, artifact);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("artifact-1"),
            ability_index: 0,
        })
        .unwrap();

        // Mana was deducted.
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
    }

    #[test]
    fn activate_mana_cost_ability_fails_when_not_enough_mana() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Only 1 mana in pool but cost is 2.
        game.add_mana_to_pool(&p1, ManaColor::Colorless, 1).unwrap();
        let artifact = make_mana_cost_ability("artifact-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, artifact);

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("artifact-1"),
            ability_index: 0,
            })
            .unwrap_err();

        assert!(
            matches!(err, GameError::InsufficientManaForSpell { .. }),
            "expected InsufficientManaForSpell, got {err:?}"
        );
    }

    #[test]
    fn mana_is_deducted_from_pool_after_mana_cost_activation() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Give 3 mana, cost is 2.
        game.add_mana_to_pool(&p1, ManaColor::Green, 3).unwrap();
        let artifact = make_mana_cost_ability("artifact-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, artifact);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("artifact-1"),
            ability_index: 0,
        })
        .unwrap();

        // 3 - 2 = 1 remaining.
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 1);
    }

    #[test]
    fn mana_cost_ability_does_not_tap_the_permanent() {
        let (mut game, p1, _) = make_game_in_first_main();
        game.add_mana_to_pool(&p1, ManaColor::Colorless, 2).unwrap();
        let artifact = make_mana_cost_ability("artifact-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, artifact);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("artifact-1"),
            ability_index: 0,
        })
        .unwrap();

        let state = game.permanent_state("artifact-1").unwrap();
        assert!(!state.is_tapped(), "Mana-only cost should not tap the permanent");
    }

    // ---- TapAndMana cost tests ----------------------------------------------

    #[test]
    fn activate_tap_and_mana_ability_taps_permanent_and_deducts_mana() {
        let (mut game, p1, _) = make_game_in_first_main();
        game.add_mana_to_pool(&p1, ManaColor::Colorless, 2).unwrap();
        let equipment = make_tap_and_mana_ability("equipment-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, equipment);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("equipment-1"),
            ability_index: 0,
        })
        .unwrap();

        let state = game.permanent_state("equipment-1").unwrap();
        assert!(state.is_tapped(), "TapAndMana cost should tap the permanent");
        assert_eq!(
            game.mana_pool(&p1).unwrap().total(),
            0,
            "Mana should be deducted"
        );
    }

    #[test]
    fn tap_and_mana_ability_fails_when_already_tapped() {
        let (mut game, p1, _) = make_game_in_first_main();
        game.add_mana_to_pool(&p1, ManaColor::Colorless, 2).unwrap();
        let equipment = make_tap_and_mana_ability("equipment-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, equipment);
        game.tap_permanent("equipment-1").unwrap();

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("equipment-1"),
            ability_index: 0,
            })
            .unwrap_err();

        assert!(matches!(err, GameError::PermanentAlreadyTapped { .. }));
    }

    #[test]
    fn tap_and_mana_ability_fails_when_not_enough_mana() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Only 1 mana but cost is 2.
        game.add_mana_to_pool(&p1, ManaColor::Colorless, 1).unwrap();
        let equipment = make_tap_and_mana_ability("equipment-1", &p1, "2");
        add_permanent_to_battlefield(&mut game, &p1, equipment);

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("equipment-1"),
            ability_index: 0,
            })
            .unwrap_err();

        assert!(matches!(err, GameError::InsufficientManaForSpell { .. }));
    }

    // ---- Multiple activated abilities (R15 / CR 602) -------------------------

    fn make_dual_ability_card(instance_id: &str, owner_id: &str) -> CardInstance {
        // Ability 0: {T}: Add {G}  (mana ability, resolves immediately)
        // Ability 1: {T}: Put ability on stack (non-mana ability, goes on stack)
        let def = CardDefinition::new("dual", "Dual Ability Land", vec![CardType::Land])
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::AddMana { color: ManaColor::Green, amount: 1 },
            })
            .with_activated_ability(ActivatedAbility {
                cost: ActivationCost::Tap,
                effect: Effect::NoOp,
            });
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn activating_ability_0_on_multi_ability_card_adds_mana() {
        let (mut game, p1, _) = make_game_in_first_main();
        let card = make_dual_ability_card("dual-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("dual-1"),
            ability_index: 0,
        })
        .unwrap();

        // Ability 0 is a mana ability — adds mana immediately, no stack.
        assert_eq!(game.mana_pool(&p1).unwrap().get(ManaColor::Green), 1);
        assert!(game.stack().is_empty(), "Mana ability should bypass the stack");
    }

    #[test]
    fn activating_ability_1_on_multi_ability_card_puts_it_on_stack() {
        let (mut game, p1, _) = make_game_in_first_main();
        let card = make_dual_ability_card("dual-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        game.apply(Action::ActivateAbility {
            player_id: PlayerId::new(&p1),
            permanent_id: CardInstanceId::new("dual-1"),
            ability_index: 1,
        })
        .unwrap();

        // Ability 1 is a non-mana ability — goes on the stack.
        assert_eq!(game.stack().len(), 1);
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
    }

    #[test]
    fn activating_out_of_range_ability_index_returns_error() {
        let (mut game, p1, _) = make_game_in_first_main();
        // Card has only 1 ability (index 0 is valid, index 1 is not).
        let land = make_mana_land("forest-1", &p1, ManaColor::Green);
        add_permanent_to_battlefield(&mut game, &p1, land);

        let err = game
            .apply(Action::ActivateAbility {
                player_id: PlayerId::new(&p1),
                permanent_id: CardInstanceId::new("forest-1"),
                ability_index: 1,
            })
            .unwrap_err();

        assert!(
            matches!(err, GameError::PermanentHasNoActivatedAbility { .. }),
            "Out-of-range index should produce PermanentHasNoActivatedAbility, got: {err:?}"
        );
    }
}
