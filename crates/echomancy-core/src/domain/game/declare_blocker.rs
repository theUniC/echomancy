//! DeclareBlocker handler — declare a creature as a blocker.

use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::services::combat_declarations::validate_declare_blocker;
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

use super::Game;

/// Handle the `DeclareBlocker` action.
///
/// Validates via `CombatDeclarations` service and applies state changes to
/// both the blocker and the attacker.
///
/// # Rules
///
/// 1. Must be the `DeclareBlockers` step.
/// 2. Defending player (opponent of the attacker) declares the blocker.
/// 3. Blocker must be on the declaring player's battlefield.
/// 4. Both blocker and attacker must be creatures.
/// 5. Blocker must not be tapped.
/// 6. Blocker must not already be blocking another creature.
/// 7. Attacker must actually be attacking.
/// 8. Attacker must not already be blocked (MVP: one blocker per attacker).
/// 9. Blocker must be able to block flyers (Flying or Reach if attacker flies).
///
/// # Errors
///
/// Various `GameError` variants from the combat validation service.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    blocker_id: &str,
    attacker_id: &str,
) -> Result<Vec<GameEvent>, GameError> {
    // The Game itself implements CombatValidationContext
    let result = validate_declare_blocker(game, player_id, blocker_id, attacker_id)?;

    // Apply state changes for both blocker and attacker
    game.set_permanent_state(blocker_id, result.new_blocker_state);
    game.set_permanent_state(attacker_id, result.new_attacker_state);

    // Build snapshots for the event — blocker is on the declaring player's battlefield;
    // attacker is on the opponent's battlefield (scanned across all players).
    let blocker_snapshot = {
        let card = game
            .players
            .iter()
            .flat_map(|p| p.battlefield.iter())
            .find(|c| c.instance_id() == blocker_id)
            .ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(blocker_id),
            })?;
        CardInstanceSnapshot {
            instance_id: CardInstanceId::new(card.instance_id()),
            definition_id: CardDefinitionId::new(card.definition().id()),
            owner_id: PlayerId::new(card.owner_id()),
        }
    };

    let attacker_snapshot = {
        let card = game
            .players
            .iter()
            .flat_map(|p| p.battlefield.iter())
            .find(|c| c.instance_id() == attacker_id)
            .ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(attacker_id),
            })?;
        CardInstanceSnapshot {
            instance_id: CardInstanceId::new(card.instance_id()),
            definition_id: CardDefinitionId::new(card.definition().id()),
            owner_id: PlayerId::new(card.owner_id()),
        }
    };

    let event = GameEvent::CreatureDeclaredBlocker {
        creature: blocker_snapshot,
        controller_id: PlayerId::new(player_id),
        blocking: attacker_snapshot,
    };

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

    fn setup_declare_blockers() -> (crate::domain::game::Game, String, String) {
        let (mut game, p1, p2) = make_started_game();
        // Advance to DeclareAttackers
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }
        assert_eq!(game.current_step(), Step::DeclareAttackers);

        // Add and declare an attacker for p1
        let attacker = make_creature_card("attacker-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, attacker);
        clear_summoning_sickness(&mut game, "attacker-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Advance to DeclareBlockers
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        (game, p1, p2)
    }

    #[test]
    fn declare_blocker_emits_creature_declared_blocker_event() {
        let (mut game, _, p2) = setup_declare_blockers();
        let blocker = make_creature_card("blocker-1", &p2, 3, 3);
        add_permanent_to_battlefield(&mut game, &p2, blocker);

        let events = game
            .apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("attacker-1"),
            })
            .unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDeclaredBlocker { .. })));
    }

    #[test]
    fn declare_blocker_sets_blocking_relationship() {
        let (mut game, _, p2) = setup_declare_blockers();
        let blocker = make_creature_card("blocker-1", &p2, 3, 3);
        add_permanent_to_battlefield(&mut game, &p2, blocker);
        // No summoning sickness check needed for blockers (only attackers)

        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        let blocker_state = game.permanent_state("blocker-1").unwrap();
        let blocker_cs = blocker_state.creature_state().unwrap();
        assert!(blocker_cs.blocking_creature_id().is_some());

        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let attacker_cs = attacker_state.creature_state().unwrap();
        assert!(attacker_cs.is_blocked());
    }

    #[test]
    fn tapped_creature_cannot_block() {
        let (mut game, _, p2) = setup_declare_blockers();
        let blocker = make_creature_card("blocker-1", &p2, 3, 3);
        add_permanent_to_battlefield(&mut game, &p2, blocker);
        game.tap_permanent("blocker-1").unwrap();

        let err = game
            .apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("attacker-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::TappedCreatureCannotBlock { .. }));
    }

    #[test]
    fn cannot_block_non_attacking_creature() {
        let (mut game, p1, p2) = setup_declare_blockers();
        let blocker = make_creature_card("blocker-1", &p2, 3, 3);
        let non_attacker = make_creature_card("non-attacker", &p1, 1, 1);
        add_permanent_to_battlefield(&mut game, &p2, blocker);
        add_permanent_to_battlefield(&mut game, &p1, non_attacker);

        let err = game
            .apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("blocker-1"),
                attacker_id: CardInstanceId::new("non-attacker"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CannotBlockNonAttackingCreature { .. }));
    }

    #[test]
    fn ground_creature_cannot_block_flying_attacker() {
        let (mut game, p1, p2) = make_started_game();
        // Set up with flying attacker
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        let flyer = make_creature_with_ability("flyer-1", &p1, 2, 2, StaticAbility::Flying);
        add_permanent_to_battlefield(&mut game, &p1, flyer);
        clear_summoning_sickness(&mut game, "flyer-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("flyer-1"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.current_step(), Step::DeclareBlockers);

        let ground_creature = make_creature_card("ground-1", &p2, 3, 3);
        add_permanent_to_battlefield(&mut game, &p2, ground_creature);

        let err = game
            .apply(Action::DeclareBlocker {
                player_id: PlayerId::new(&p2),
                blocker_id: CardInstanceId::new("ground-1"),
                attacker_id: CardInstanceId::new("flyer-1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CannotBlockFlyingCreature { .. }));
    }

    #[test]
    fn creature_with_reach_can_block_flying_attacker() {
        let (mut game, p1, p2) = make_started_game();
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        let flyer = make_creature_with_ability("flyer-1", &p1, 2, 2, StaticAbility::Flying);
        add_permanent_to_battlefield(&mut game, &p1, flyer);
        clear_summoning_sickness(&mut game, "flyer-1");

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("flyer-1"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        let reach_creature =
            make_creature_with_ability("reach-1", &p2, 3, 3, StaticAbility::Reach);
        add_permanent_to_battlefield(&mut game, &p2, reach_creature);

        // Should succeed
        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("reach-1"),
            attacker_id: CardInstanceId::new("flyer-1"),
        })
        .unwrap();

        let state = game.permanent_state("reach-1").unwrap();
        assert!(state.creature_state().unwrap().blocking_creature_id().is_some());
    }

    #[test]
    fn creature_can_be_blocked_by_multiple_creatures() {
        let (mut game, _, p2) = setup_declare_blockers();

        let blocker1 = make_creature_card("blocker-1", &p2, 1, 1);
        let blocker2 = make_creature_card("blocker-2", &p2, 2, 2);
        add_permanent_to_battlefield(&mut game, &p2, blocker1);
        add_permanent_to_battlefield(&mut game, &p2, blocker2);

        // First blocker declares.
        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-1"),
            attacker_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Second blocker should also be allowed — no error.
        game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker-2"),
            attacker_id: CardInstanceId::new("attacker-1"),
        })
        .unwrap();

        // Attacker should now have two blockers.
        let attacker_state = game.permanent_state("attacker-1").unwrap();
        let attacker_cs = attacker_state.creature_state().unwrap();
        assert_eq!(attacker_cs.blocked_by().len(), 2);

        // Both blockers should track which creature they are blocking.
        let b1_state = game.permanent_state("blocker-1").unwrap();
        assert_eq!(b1_state.creature_state().unwrap().blocking_creature_id(), Some("attacker-1"));

        let b2_state = game.permanent_state("blocker-2").unwrap();
        assert_eq!(b2_state.creature_state().unwrap().blocking_creature_id(), Some("attacker-1"));
    }
}
