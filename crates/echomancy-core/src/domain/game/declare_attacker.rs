//! DeclareAttacker handler — declare a creature as an attacker.

use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::services::combat_declarations::validate_declare_attacker;
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

use super::Game;

/// Handle the `DeclareAttacker` action.
///
/// Validates via `CombatDeclarations` service and applies state changes.
///
/// # Rules
///
/// 1. Must be the `DeclareAttackers` step.
/// 2. Must be the active (current) player.
/// 3. Creature must be on the player's battlefield.
/// 4. Must be a creature.
/// 5. No summoning sickness (unless Haste).
/// 6. Not tapped.
/// 7. Not already attacked this turn.
///
/// # Errors
///
/// Returns `GameError::InvalidPlayerAction` or more specific errors from the
/// combat validation service.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    creature_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // The Game itself implements CombatValidationContext
    let result = validate_declare_attacker(game, player_id, creature_id)?;

    // Apply the new state for the attacking creature
    game.set_permanent_state(creature_id, result.new_state);

    // Find the card for the event snapshot
    let card_snapshot = {
        let player = game.player_state(player_id)?;
        let card = player
            .battlefield
            .iter()
            .find(|c| c.instance_id() == creature_id)
            .ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(creature_id),
            })?;
        CardInstanceSnapshot {
            instance_id: CardInstanceId::new(card.instance_id()),
            definition_id: CardDefinitionId::new(card.definition().id()),
            owner_id: PlayerId::new(card.owner_id()),
        }
    };

    let event = GameEvent::CreatureDeclaredAttacker {
        creature: card_snapshot,
        controller_id: PlayerId::new(player_id),
    };

    // Evaluate triggers
    let triggered = game.collect_triggered_abilities(&event);
    game.execute_triggered_abilities(triggered);

    Ok(vec![event])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::enums::{StaticAbility, Step};
    use crate::domain::game::test_helpers::{
        add_permanent_to_battlefield, clear_summoning_sickness, make_creature_card,
        make_creature_with_ability, make_started_game,
    };
    use crate::domain::types::{CardInstanceId, PlayerId};

    fn setup_declare_attackers(player_id: &str) -> (crate::domain::game::Game, String, String) {
        let (mut game, p1, p2) = make_started_game();
        // Advance to DeclareAttackers step
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);
        (game, p1, p2)
    }

    #[test]
    fn declare_attacker_sets_is_attacking() {
        let (mut game, p1, _) = setup_declare_attackers(&"p1");
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, "bear-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("bear-1"),
        })
        .unwrap();

        let state = game.permanent_state("bear-1").unwrap();
        let cs = state.creature_state().unwrap();
        assert!(cs.is_attacking());
        assert!(cs.has_attacked_this_turn());
        assert!(state.is_tapped());
    }

    #[test]
    fn declare_attacker_emits_creature_declared_attacker_event() {
        let (mut game, p1, _) = setup_declare_attackers("p1");
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, "bear-1");

        let events = game
            .apply(Action::DeclareAttacker {
                player_id: PlayerId::new(&p1),
                creature_id: CardInstanceId::new("bear-1"),
            })
            .unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDeclaredAttacker { .. })));
    }

    #[test]
    fn creature_with_summoning_sickness_cannot_attack() {
        let (mut game, p1, _) = setup_declare_attackers("p1");
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        // Do NOT clear summoning sickness

        let err = game
            .apply(Action::DeclareAttacker {
                player_id: PlayerId::new(&p1),
                creature_id: CardInstanceId::new("bear-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CreatureHasSummoningSickness { .. }));
    }

    #[test]
    fn creature_with_haste_can_attack_with_summoning_sickness() {
        let (mut game, p1, _) = setup_declare_attackers("p1");
        let creature =
            make_creature_with_ability("bear-1", &p1, 2, 2, StaticAbility::Haste);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        // Has summoning sickness but also Haste

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("bear-1"),
        })
        .unwrap();

        let state = game.permanent_state("bear-1").unwrap();
        assert!(state.creature_state().unwrap().is_attacking());
    }

    #[test]
    fn tapped_creature_cannot_attack() {
        let (mut game, p1, _) = setup_declare_attackers("p1");
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, "bear-1");
        game.tap_permanent("bear-1").unwrap();

        let err = game
            .apply(Action::DeclareAttacker {
                player_id: PlayerId::new(&p1),
                creature_id: CardInstanceId::new("bear-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::TappedCreatureCannotAttack { .. }));
    }

    #[test]
    fn cannot_declare_attacker_outside_declare_attackers_step() {
        let (mut game, p1, _) = make_started_game();
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, "bear-1");

        let err = game
            .apply(Action::DeclareAttacker {
                player_id: PlayerId::new(&p1),
                creature_id: CardInstanceId::new("bear-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerAction { .. }));
    }
}
