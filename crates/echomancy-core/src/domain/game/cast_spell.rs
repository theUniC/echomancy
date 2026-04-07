//! CastSpell handler — cast a spell from hand onto the stack.

use crate::domain::entities::the_stack::{SpellOnStack, StackItem};
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::services::spell_timing::is_instant_speed;
use crate::domain::targets::{Target, TargetRequirement};
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

use super::Game;

/// Handle the `CastSpell` action.
///
/// # Rules
///
/// 1. Player must have priority.
/// 2. The card must be in the player's hand.
/// 3. The card must be castable (not a land).
/// 4. Timing rules apply:
///    - Instant-speed: can cast any time player has priority.
///    - Sorcery-speed: must be active player's turn, main phase, empty stack.
/// 5. Target validation (CR 601.2c):
///    - Spells with `AnyTarget` or `Creature` requirement must have exactly one target.
///    - `AnyTarget` accepts `Player` or `Creature` targets; both are validated.
///    - `Creature` requirement only accepts `Creature` targets.
///    - `None` requirement ignores provided targets.
/// 6. Mana cost must be payable.
///
/// After validation, the card is removed from hand and placed on the stack.
/// Per CR 117.3c, priority returns to the caster (not the opponent).
///
/// # Errors
///
/// Various `GameError` variants for each validation failure.
pub(crate) fn handle(
    game: &mut Game,
    player_id: &str,
    card_id: &str,
    targets: Vec<Target>,
    x_value: u32,
) -> Result<Vec<GameEvent>, GameError> {
    // 1. Player must have priority
    if !game.has_priority(player_id) {
        return Err(GameError::InvalidPlayerAction {
            player_id: player_id.into(),
            action: "CAST_SPELL".to_owned(),
        });
    }

    // 2. Card must be in hand
    let card = {
        let player = game.player_state(player_id)?;
        player
            .hand
            .iter()
            .find(|c| c.instance_id() == card_id)
            .cloned()
            .ok_or_else(|| GameError::CardNotFoundInHand {
                card_id: CardInstanceId::new(card_id),
                player_id: PlayerId::new(player_id),
            })?
    };

    // 3. Card must be castable (not a land)
    if card.definition().is_land() {
        return Err(GameError::CardIsNotSpell {
            card_id: CardInstanceId::new(card_id),
        });
    }

    // 4. Target validation (CR 601.2c)
    let (validated_targets, ward_payments) = validate_targets(game, player_id, card_id, card.definition().target_requirement(), &targets)?;

    // 5. Timing validation
    if !is_instant_speed(&card) {
        let is_creature = card.definition().is_creature();

        // Must be the active player's turn
        if game.current_player_id() != player_id {
            return Err(GameError::not_your_turn(is_creature));
        }

        // Must be in a main phase
        if !game.turn_state.is_main_phase() {
            return Err(GameError::not_main_phase(is_creature));
        }

        // Stack must be empty
        if game.stack_has_items() {
            return Err(GameError::stack_not_empty(is_creature));
        }
    }

    // 6. Pay mana cost (using effective cost with X resolved if applicable)
    game.pay_mana_cost_for_spell_with_x(player_id, &card, x_value)?;

    // 6b. Pay ward costs (CR 702.21) — deduct from caster's mana pool.
    if !ward_payments.is_empty() {
        use crate::domain::services::mana_payment::pay_cost;
        use crate::domain::value_objects::mana::ManaCost;
        for ward in ward_payments {
            let ward_cost = ManaCost {
                generic: ward.amount,
                white: 0,
                blue: 0,
                black: 0,
                red: 0,
                green: 0,
                colorless: 0,
                x: 0,
            };
            let player = game.player_state_mut(&ward.player_id)?;
            let new_pool = pay_cost(player.mana_pool.clone(), &ward_cost)
                .map_err(|_| GameError::InvalidTarget {
                    reason: format!("permanent has ward — must pay {{{}}}", ward.amount),
                })?;
            player.mana_pool = new_pool;
        }
    }

    // Remove from hand
    {
        let player = game.player_state_mut(player_id)?;
        player.hand.retain(|c| c.instance_id() != card_id);
    }

    // Capture card data for TR6 event before moving the card.
    let spell_cast_event = GameEvent::SpellCast {
        card_id: CardInstanceId::new(card.instance_id()),
        card_definition_id: CardDefinitionId::new(card.definition().id()),
        controller_id: PlayerId::new(player_id),
    };

    // Push onto stack
    game.push_stack(StackItem::Spell(SpellOnStack {
        card,
        controller_id: player_id.to_owned(),
        targets: validated_targets,
        x_value,
    }));

    // TR6: Evaluate "whenever a spell is cast" triggers AFTER the spell is on the stack.
    let spell_cast_triggered = game.collect_triggered_abilities(&spell_cast_event);
    game.execute_triggered_abilities(spell_cast_triggered);

    // CR 117.3c: after casting, priority returns to the caster.
    // Clear the "both passed" set since a new object was added to the stack.
    game.players_who_passed_priority.clear();
    let mut events = game.assign_priority_to(player_id);
    // Include the SpellCast event in the returned events list.
    events.push(spell_cast_event);
    Ok(events)
}

/// Validate the chosen targets against the card's requirement.
///
/// Returns the validated target list (identical to input when valid) along with
/// any pending ward cost payments, or a `GameError` describing the first violation.
fn validate_targets(
    game: &Game,
    caster_id: &str,
    card_id: &str,
    requirement: TargetRequirement,
    targets: &[Target],
) -> Result<(Vec<Target>, Vec<WardPayment>), GameError> {
    let mut ward_payments: Vec<WardPayment> = Vec::new();

    match requirement {
        TargetRequirement::None => {
            // Targets are silently ignored for spells that don't need them.
            Ok((Vec::new(), ward_payments))
        }
        TargetRequirement::AnyTarget => {
            if targets.is_empty() {
                return Err(GameError::TargetRequired {
                    card_id: card_id.to_owned(),
                });
            }
            // Validate the first target (MVP: exactly one target).
            let target = &targets[0];
            match target {
                Target::Player { player_id } => {
                    // Validate the player exists in the game.
                    if game.player_life_total(player_id).is_err() {
                        return Err(GameError::InvalidTarget {
                            reason: format!("player '{player_id}' is not in the game"),
                        });
                    }
                }
                Target::Creature { permanent_id } | Target::Permanent { permanent_id } => {
                    if let Some(ward) = validate_permanent_target_with_type(game, caster_id, permanent_id, None, |_| true)? {
                        ward_payments.push(ward);
                    }
                }
                Target::StackSpell { spell_id } => {
                    validate_stack_target(game, spell_id)?;
                }
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
        TargetRequirement::Creature => {
            require_one_target(targets, card_id)?;
            match &targets[0] {
                Target::Creature { permanent_id } => {
                    if let Some(ward) = validate_creature_target(game, caster_id, permanent_id)? {
                        ward_payments.push(ward);
                    }
                }
                _ => return Err(GameError::InvalidTarget {
                    reason: "target must be a creature".to_owned(),
                }),
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
        TargetRequirement::Artifact => {
            require_one_target(targets, card_id)?;
            let perm_id = require_permanent_target(&targets[0])?;
            if let Some(ward) = validate_permanent_target_with_type(game, caster_id, perm_id, Some("artifact"), |c| c.definition().is_artifact())? {
                ward_payments.push(ward);
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
        TargetRequirement::Enchantment => {
            require_one_target(targets, card_id)?;
            let perm_id = require_permanent_target(&targets[0])?;
            if let Some(ward) = validate_permanent_target_with_type(game, caster_id, perm_id, Some("enchantment"), |c| c.definition().is_enchantment())? {
                ward_payments.push(ward);
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
        TargetRequirement::ArtifactOrEnchantment => {
            require_one_target(targets, card_id)?;
            let perm_id = require_permanent_target(&targets[0])?;
            if let Some(ward) = validate_permanent_target_with_type(game, caster_id, perm_id, Some("artifact or enchantment"), |c| {
                c.definition().is_artifact() || c.definition().is_enchantment()
            })? {
                ward_payments.push(ward);
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
        TargetRequirement::Permanent => {
            require_one_target(targets, card_id)?;
            let perm_id = require_permanent_target(&targets[0])?;
            if let Some(ward) = validate_permanent_target_with_type(game, caster_id, perm_id, None, |_| true)? {
                ward_payments.push(ward);
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
        TargetRequirement::Spell => {
            require_one_target(targets, card_id)?;
            match &targets[0] {
                Target::StackSpell { spell_id } => {
                    validate_stack_target(game, spell_id)?;
                }
                _ => return Err(GameError::InvalidTarget {
                    reason: "target must be a spell on the stack".to_owned(),
                }),
            }
            Ok((targets[..1].to_vec(), ward_payments))
        }
    }
}

fn require_one_target(targets: &[Target], card_id: &str) -> Result<(), GameError> {
    if targets.is_empty() {
        return Err(GameError::TargetRequired {
            card_id: card_id.to_owned(),
        });
    }
    Ok(())
}

fn require_permanent_target(target: &Target) -> Result<&str, GameError> {
    match target {
        Target::Creature { permanent_id } | Target::Permanent { permanent_id } => {
            Ok(permanent_id.as_str())
        }
        _ => Err(GameError::InvalidTarget {
            reason: "target must be a permanent on the battlefield".to_owned(),
        }),
    }
}

/// Validate that a permanent ID refers to a creature on the battlefield.
fn validate_creature_target(game: &Game, caster_id: &str, permanent_id: &str) -> Result<Option<WardPayment>, GameError> {
    validate_permanent_target_with_type(game, caster_id, permanent_id, Some("creature"), |card| {
        card.definition().is_creature()
    })
}

/// Validate that a permanent exists on the battlefield, optionally checking a type predicate.
///
/// Also checks Hexproof (CR 702.11): a permanent with Hexproof can't be targeted
/// by spells or abilities controlled by an opponent.
/// Also checks Ward (CR 702.21): if the permanent has a ward cost and the caster
/// is an opponent, the ward cost is deducted from the caster's mana pool.
fn validate_permanent_target_with_type(
    game: &Game,
    caster_id: &str,
    permanent_id: &str,
    type_name: Option<&str>,
    type_check: impl Fn(&crate::domain::cards::card_instance::CardInstance) -> bool,
) -> Result<Option<WardPayment>, GameError> {
    use crate::domain::enums::StaticAbility;

    for pid in game.turn_order() {
        if let Ok(battlefield) = game.battlefield(pid) {
            if let Some(card) = battlefield.iter().find(|c| c.instance_id() == permanent_id) {
                if let Some(name) = type_name {
                    if !type_check(card) {
                        return Err(GameError::InvalidTarget {
                            reason: format!("permanent '{permanent_id}' is not a {name}"),
                        });
                    }
                }
                // CR 702.18: Shroud — can't be targeted by anyone.
                // Use the layer pipeline so Layer 6 effects (e.g. RemoveAllAbilities) are respected.
                let has_shroud = game
                    .effective_abilities(card.instance_id())
                    .map(|a| a.contains(&StaticAbility::Shroud))
                    .unwrap_or(false);
                if has_shroud {
                    return Err(GameError::InvalidTarget {
                        reason: format!(
                            "permanent '{permanent_id}' has shroud and can't be targeted"
                        ),
                    });
                }
                // CR 702.11: Hexproof — can't be targeted by opponents.
                // Use the layer pipeline so Layer 6 effects (e.g. RemoveAllAbilities) are respected.
                let has_hexproof = game
                    .effective_abilities(card.instance_id())
                    .map(|a| a.contains(&StaticAbility::Hexproof))
                    .unwrap_or(false);
                if has_hexproof && pid != caster_id
                {
                    return Err(GameError::InvalidTarget {
                        reason: format!(
                            "permanent '{permanent_id}' has hexproof and can't be targeted by opponents"
                        ),
                    });
                }
                // CR 702.21: Ward — opponent must pay ward cost to target this permanent.
                let ward = card.definition().ward_cost();
                if ward > 0 && pid != caster_id {
                    // Check if the caster has enough mana to pay the ward cost.
                    let caster_pool = game.mana_pool(caster_id).map_err(|_| GameError::InvalidTarget {
                        reason: format!("permanent '{permanent_id}' has ward — must pay {{{ward}}}"),
                    })?;
                    if caster_pool.total() < ward {
                        return Err(GameError::InvalidTarget {
                            reason: format!(
                                "permanent '{permanent_id}' has ward — must pay {{{ward}}}"
                            ),
                        });
                    }
                    // Return the ward payment to apply after validation
                    return Ok(Some(WardPayment {
                        player_id: caster_id.to_owned(),
                        amount: ward,
                    }));
                }
                return Ok(None);
            }
        }
    }
    Err(GameError::InvalidTarget {
        reason: format!("permanent '{permanent_id}' is not on the battlefield"),
    })
}

/// Deferred ward cost payment to apply after full target validation.
struct WardPayment {
    player_id: String,
    amount: u32,
}

/// Validate that a spell ID refers to a spell currently on the stack.
fn validate_stack_target(game: &Game, spell_id: &str) -> Result<(), GameError> {
    use crate::domain::entities::the_stack::StackItem;
    let found = game.stack().iter().any(|item| match item {
        StackItem::Spell(s) => s.card.instance_id() == spell_id,
        StackItem::Ability(a) => a.source_id == spell_id,
    });
    if found {
        Ok(())
    } else {
        Err(GameError::InvalidTarget {
            reason: format!("spell '{spell_id}' is not on the stack"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::{CardType, ManaColor, StaticAbility};
    use crate::domain::game::test_helpers::{
        add_card_to_hand, add_permanent_to_battlefield, make_creature_card, make_game_in_first_main, make_land_card,
        make_started_game,
    };
    use crate::domain::types::{CardInstanceId, PlayerId};
    use crate::domain::value_objects::mana::ManaCost;

    fn make_ward_creature(instance_id: &str, owner_id: &str, ward_cost: u32) -> CardInstance {
        let def = CardDefinition::new("ward-creature", "Ward Creature", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_ward(ward_cost);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_instant_targeting_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::targets::TargetRequirement;
        let def = CardDefinition::new("removal", "Removal", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::Creature);
        CardInstance::new(instance_id, def, owner_id)
    }

    // ---- Ward (K11.2) ---------------------------------------------------

    #[test]
    fn ward_creature_cannot_be_targeted_by_opponent_without_enough_mana() {
        let (mut game, p1, p2) = make_game_in_first_main();

        // p2 has a ward-2 creature on the battlefield
        let ward_creature = make_ward_creature("ward-1", &p2, 2);
        add_permanent_to_battlefield(&mut game, &p2, ward_creature);

        // p1 casts a removal spell targeting ward creature but has no mana to pay ward cost
        let removal = make_instant_targeting_creature("removal-1", &p1);
        add_card_to_hand(&mut game, &p1, removal);

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("removal-1"),
            targets: vec![Target::creature("ward-1")],
            x_value: 0,
        });

        assert!(result.is_err(), "should fail when opponent can't pay ward cost");
    }

    #[test]
    fn ward_creature_can_be_targeted_by_opponent_who_pays_ward_cost() {
        let (mut game, p1, p2) = make_game_in_first_main();

        // p2 has a ward-2 creature on the battlefield
        let ward_creature = make_ward_creature("ward-1", &p2, 2);
        add_permanent_to_battlefield(&mut game, &p2, ward_creature);

        // p1 casts a removal spell targeting ward creature and has enough mana
        let removal = make_instant_targeting_creature("removal-1", &p1);
        add_card_to_hand(&mut game, &p1, removal);
        game.add_mana(&p1, ManaColor::Colorless, 2).unwrap();

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("removal-1"),
            targets: vec![Target::creature("ward-1")],
            x_value: 0,
        });

        assert!(result.is_ok(), "should succeed when opponent pays ward cost");
        // Ward cost should be deducted from mana pool
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
    }

    #[test]
    fn ward_does_not_affect_controller_targeting_own_creature() {
        let (mut game, p1, _p2) = make_game_in_first_main();

        // p1 has a ward-2 creature on the battlefield
        let ward_creature = make_ward_creature("ward-1", &p1, 2);
        add_permanent_to_battlefield(&mut game, &p1, ward_creature);

        // p1 targets their own ward creature — no ward cost needed
        let removal = make_instant_targeting_creature("removal-1", &p1);
        add_card_to_hand(&mut game, &p1, removal);
        // No mana added — controller doesn't pay ward

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("removal-1"),
            targets: vec![Target::creature("ward-1")],
            x_value: 0,
        });

        assert!(result.is_ok(), "controller can target own ward creature without paying ward cost");
    }

    fn make_sorcery(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("shock", "Shock", vec![CardType::Sorcery]);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_sorcery_with_cost(instance_id: &str, owner_id: &str, cost: &str) -> CardInstance {
        let mana_cost = ManaCost::parse(cost).unwrap();
        let def = CardDefinition::new("shock", "Shock", vec![CardType::Sorcery])
            .with_mana_cost(mana_cost);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_instant(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("cancel", "Cancel", vec![CardType::Instant]);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_flash_creature(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("flash-bear", "Flash Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_static_ability(StaticAbility::Flash);
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn cast_spell_moves_card_from_hand_to_stack() {
        let (mut game, p1, _) = make_game_in_first_main();
        let spell = make_sorcery("spell-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("spell-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert!(game.hand(&p1).unwrap().is_empty());
        assert_eq!(game.stack().len(), 1);
    }

    /// CR 117.3c: after a player casts a spell, that same player receives priority.
    #[test]
    fn cast_spell_returns_priority_to_caster_per_cr_117_3c() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let spell = make_sorcery("spell-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("spell-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert_eq!(
            game.priority_player_id(),
            Some(p1.as_str()),
            "caster should retain priority after casting per CR 117.3c"
        );
    }

    /// After caster passes priority, opponent gets it.
    #[test]
    fn opponent_gets_priority_after_caster_passes() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let spell = make_sorcery("spell-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("spell-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        // Caster passes priority
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(
            game.priority_player_id(),
            Some(p2.as_str()),
            "opponent should have priority after caster passes"
        );
    }

    #[test]
    fn cannot_cast_land_as_spell() {
        let (mut game, p1, _) = make_game_in_first_main();
        let land = make_land_card("land-1", &p1);
        add_card_to_hand(&mut game, &p1, land);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("land-1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::CardIsNotSpell { .. }));
    }

    #[test]
    fn cannot_cast_sorcery_outside_main_phase() {
        let (mut game, p1, _) = make_started_game();
        let spell = make_sorcery("spell-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("spell-1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::NotMainPhase { .. }));
    }

    #[test]
    fn cannot_cast_sorcery_when_not_your_turn() {
        // p1 has priority in their first main. p2 tries to cast a sorcery
        // without having priority — the game should reject this.
        let (mut game, _, p2) = make_game_in_first_main();
        let spell = make_sorcery("spell-1", &p2);
        add_card_to_hand(&mut game, &p2, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p2),
                card_id: CardInstanceId::new("spell-1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        // p2 doesn't have priority, so the game returns InvalidPlayerAction
        // (priority check fires before the NotYourTurn timing check).
        assert!(matches!(
            err,
            GameError::InvalidPlayerAction { .. } | GameError::NotYourTurn { .. }
        ));
    }

    #[test]
    fn cannot_cast_sorcery_when_stack_not_empty() {
        // CR 117.3c: after casting, p1 retains priority. p1 has priority but
        // cannot cast a sorcery because the stack is not empty.
        let (mut game, p1, _p2) = make_game_in_first_main();
        let spell1 = make_sorcery("s1", &p1);
        let spell2 = make_sorcery("s2", &p1);
        add_card_to_hand(&mut game, &p1, spell1);
        add_card_to_hand(&mut game, &p1, spell2);

        // Cast first spell — p1 retains priority (CR 117.3c)
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("s1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        // p1 has priority but the stack is not empty — cannot cast a sorcery
        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("s2"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        assert!(matches!(
            err,
            GameError::StackNotEmpty { .. }
        ));
    }

    #[test]
    fn can_cast_instant_on_opponents_turn() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let instant = make_instant("instant-1", &p2);
        add_card_to_hand(&mut game, &p2, instant);

        // p1 has priority in first main, cast a spell (p1 retains priority per CR 117.3c)
        let spell = make_sorcery("sorcery-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("sorcery-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        // p1 passes priority — p2 now gets priority and can cast an instant
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p2),
            card_id: CardInstanceId::new("instant-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert_eq!(game.stack().len(), 2);
    }

    #[test]
    fn cast_creature_with_mana_cost_deducts_mana() {
        let (mut game, p1, _) = make_game_in_first_main();
        let creature = {
            use crate::domain::value_objects::mana::ManaCost;
            let cost = ManaCost::parse("2").unwrap();
            let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
                .with_power_toughness(2, 2)
                .with_mana_cost(cost);
            CardInstance::new("bear-1", def, &p1)
        };
        add_card_to_hand(&mut game, &p1, creature);

        // Add 2 colorless mana
        game.add_mana(&p1, ManaColor::Colorless, 2).unwrap();
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 2);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("bear-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
    }

    #[test]
    fn cast_spell_without_enough_mana_returns_error() {
        let (mut game, p1, _) = make_game_in_first_main();
        let spell = make_sorcery_with_cost("s1", &p1, "3");
        add_card_to_hand(&mut game, &p1, spell);

        // Only 1 mana available
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("s1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InsufficientManaForSpell { .. }));
    }

    #[test]
    fn cast_flash_creature_on_opponents_turn() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let flash_creature = make_flash_creature("flash-1", &p2);
        add_card_to_hand(&mut game, &p2, flash_creature);

        // p1 casts a sorcery (p1 retains priority per CR 117.3c)
        let spell = make_sorcery("s1", &p1);
        add_card_to_hand(&mut game, &p1, spell);
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("s1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        // p1 passes priority — p2 gets priority and can cast flash creature
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p2),
            card_id: CardInstanceId::new("flash-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();
        assert_eq!(game.stack().len(), 2);
    }

    #[test]
    fn spell_resolves_after_both_players_pass_priority() {
        let (mut game, p1, p2) = make_game_in_first_main();
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_card_to_hand(&mut game, &p1, creature);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("bear-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert_eq!(game.stack().len(), 1);

        // CR 117.3c: p1 (caster) retains priority
        assert_eq!(game.priority_player_id(), Some(p1.as_str()));

        // p1 passes priority — p2 gets it
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        assert_eq!(game.priority_player_id(), Some(p2.as_str()));

        // p2 passes priority — both have passed, stack resolves
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p2),
        })
        .unwrap();

        // Stack should be empty, creature on battlefield
        assert!(game.stack().is_empty());
        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
    }

    /// End-to-end test: cast a Bear ({1}{G}) with real mana, pass priority from
    /// both players, and verify the creature appears on the battlefield.
    ///
    /// This test pins the full engine chain:
    ///   play 2 Forests → tap both → cast Bear → pass priority × 2 → battlefield
    #[test]
    fn bear_with_mana_cost_resolves_onto_battlefield() {
        use crate::domain::cards::catalog;

        let (mut game, p1, p2) = make_game_in_first_main();

        // Build a Bear with cost {1}{G} — matching the catalog definition.
        let mana_cost = ManaCost::parse("1G").unwrap();
        let bear_def = catalog::bear().with_mana_cost(mana_cost);
        let bear = CardInstance::new("bear-1", bear_def, &p1);
        add_card_to_hand(&mut game, &p1, bear);

        // Give P1 exactly {1}{G} in their mana pool (1 generic + 1 green).
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();
        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();

        // Cast the Bear — mana should be consumed.
        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("bear-1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        // Bear is on the stack; mana pool is now empty.
        assert_eq!(game.stack().len(), 1);
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);

        // CR 117.3c: P1 (caster) retains priority.
        assert_eq!(game.priority_player_id(), Some(p1.as_str()));

        // P1 passes priority → P2 gets priority.
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();
        assert_eq!(game.priority_player_id(), Some(p2.as_str()));

        // P2 passes priority → both have passed → stack resolves.
        game.apply(Action::PassPriority {
            player_id: PlayerId::new(&p2),
        })
        .unwrap();

        // Bear must be on the battlefield, stack must be empty.
        assert!(game.stack().is_empty(), "Stack should be empty after resolution");
        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            1,
            "Bear should be on P1's battlefield"
        );
    }

    // -------------------------------------------------------------------------
    // Target validation tests
    // -------------------------------------------------------------------------

    fn make_instant_any_target(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::targets::TargetRequirement;
        let def = CardDefinition::new("lightning-strike", "Lightning Strike", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::AnyTarget);
        CardInstance::new(instance_id, def, owner_id)
    }

    fn make_instant_creature_target(instance_id: &str, owner_id: &str) -> CardInstance {
        use crate::domain::targets::TargetRequirement;
        let def = CardDefinition::new("doom-blade", "Doom Blade", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::Creature);
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn any_target_spell_with_no_targets_returns_target_required() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let spell = make_instant_any_target("strike-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("strike-1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        assert!(
            matches!(err, GameError::TargetRequired { .. }),
            "expected TargetRequired, got: {err:?}"
        );
    }

    #[test]
    fn creature_requirement_spell_with_no_targets_returns_target_required() {
        let (mut game, p1, _p2) = make_game_in_first_main();
        let spell = make_instant_creature_target("blade-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("blade-1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::TargetRequired { .. }));
    }

    #[test]
    fn any_target_spell_with_valid_player_target_succeeds() {
        use crate::domain::targets::Target;
        let (mut game, p1, p2) = make_game_in_first_main();
        let spell = make_instant_any_target("strike-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("strike-1"),
            targets: vec![Target::player(&p2)],
            x_value: 0,
        })
        .expect("casting with a valid player target should succeed");

        assert_eq!(game.stack().len(), 1);
        // Verify the target was stored on the stack item
        if let crate::domain::entities::the_stack::StackItem::Spell(spell) = &game.stack()[0] {
            assert_eq!(spell.targets.len(), 1);
            assert_eq!(spell.targets[0].player_id(), Some(p2.as_str()));
        } else {
            panic!("expected a spell on the stack");
        }
    }

    #[test]
    fn any_target_spell_with_valid_creature_target_succeeds() {
        use crate::domain::targets::Target;
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;
        let (mut game, p1, p2) = make_game_in_first_main();

        // Put a creature on p2's battlefield
        let creature = make_creature_card("bear-99", &p2, 2, 2);
        add_permanent_to_battlefield(&mut game, &p2, creature);

        let spell = make_instant_any_target("strike-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("strike-1"),
            targets: vec![Target::creature("bear-99")],
            x_value: 0,
        })
        .expect("casting with a valid creature target should succeed");

        assert_eq!(game.stack().len(), 1);
    }

    #[test]
    fn any_target_spell_with_nonexistent_player_returns_invalid_target() {
        use crate::domain::targets::Target;
        let (mut game, p1, _p2) = make_game_in_first_main();
        let spell = make_instant_any_target("strike-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("strike-1"),
                targets: vec![Target::player("ghost-player")],
                x_value: 0,
            })
            .unwrap_err();
        assert!(
            matches!(err, GameError::InvalidTarget { .. }),
            "expected InvalidTarget, got: {err:?}"
        );
    }

    #[test]
    fn any_target_spell_with_nonexistent_creature_returns_invalid_target() {
        use crate::domain::targets::Target;
        let (mut game, p1, _p2) = make_game_in_first_main();
        let spell = make_instant_any_target("strike-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("strike-1"),
                targets: vec![Target::creature("perm-doesnt-exist")],
                x_value: 0,
            })
            .unwrap_err();
        assert!(matches!(err, GameError::InvalidTarget { .. }));
    }

    #[test]
    fn creature_requirement_spell_with_player_target_returns_invalid_target() {
        use crate::domain::targets::Target;
        let (mut game, p1, p2) = make_game_in_first_main();
        let spell = make_instant_creature_target("blade-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        let err = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("blade-1"),
                targets: vec![Target::player(&p2)],
                x_value: 0,
            })
            .unwrap_err();
        assert!(
            matches!(err, GameError::InvalidTarget { .. }),
            "expected InvalidTarget for player target on creature-only spell, got: {err:?}"
        );
    }

    #[test]
    fn no_target_spell_with_extra_targets_ignores_them() {
        // A Bear (TargetRequirement::None) should succeed even if targets are provided,
        // and the stored spell on the stack should have an empty target list.
        use crate::domain::targets::Target;
        let (mut game, p1, p2) = make_game_in_first_main();
        let bear = make_creature_card("bear-1", &p1, 2, 2);
        add_card_to_hand(&mut game, &p1, bear);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("bear-1"),
            targets: vec![Target::player(&p2)], // ignored
            x_value: 0,
        })
        .expect("bear with no target requirement should accept extra targets");

        assert_eq!(game.stack().len(), 1);
        if let crate::domain::entities::the_stack::StackItem::Spell(spell) = &game.stack()[0] {
            assert!(spell.targets.is_empty(), "targets should be ignored for non-targeting spells");
        }
    }

    #[test]
    fn target_stored_on_stack_item() {
        use crate::domain::targets::Target;
        let (mut game, p1, p2) = make_game_in_first_main();
        let spell = make_instant_any_target("strike-1", &p1);
        add_card_to_hand(&mut game, &p1, spell);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("strike-1"),
            targets: vec![Target::player(&p2)],
            x_value: 0,
        })
        .unwrap();

        if let crate::domain::entities::the_stack::StackItem::Spell(spell) = &game.stack()[0] {
            assert_eq!(spell.targets.len(), 1, "should have one stored target");
            assert_eq!(spell.targets[0], Target::player(&p2));
        } else {
            panic!("expected spell on stack");
        }
    }

    // ---- Hexproof (CR 702.11) -------------------------------------------

    #[test]
    fn hexproof_creature_cannot_be_targeted_by_opponent() {
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;

        let (mut game, p1, p2) = make_game_in_first_main();

        // P2 has a Hexproof creature on battlefield
        let hexproof_bear = CardDefinition::new("hex-bear", "Hexproof Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_static_ability(StaticAbility::Hexproof);
        let card = CardInstance::new("hex-bear-1", hexproof_bear, &p2);
        add_permanent_to_battlefield(&mut game, &p2, card);

        // P1 casts a spell targeting P2's Hexproof creature
        let bolt = CardDefinition::new("bolt", "Lightning Bolt", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::Creature);
        let bolt_card = CardInstance::new("bolt-1", bolt, &p1);
        add_card_to_hand(&mut game, &p1, bolt_card);

        // Add mana
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("bolt-1"),
            targets: vec![Target::creature("hex-bear-1")],
            x_value: 0,
        });

        assert!(result.is_err(), "should not be able to target Hexproof creature");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("hexproof"), "error should mention hexproof: {err}");
    }

    #[test]
    fn hexproof_creature_can_be_targeted_by_controller() {
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;

        let (mut game, p1, _p2) = make_game_in_first_main();

        // P1 has a Hexproof creature on battlefield
        let hexproof_bear = CardDefinition::new("hex-bear", "Hexproof Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_static_ability(StaticAbility::Hexproof);
        let card = CardInstance::new("hex-bear-1", hexproof_bear, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        // P1 casts Giant Growth on own Hexproof creature — should succeed
        let growth = CardDefinition::new("growth", "Giant Growth", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::Creature);
        let growth_card = CardInstance::new("growth-1", growth, &p1);
        add_card_to_hand(&mut game, &p1, growth_card);

        game.add_mana(&p1, ManaColor::Green, 1).unwrap();

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("growth-1"),
            targets: vec![Target::creature("hex-bear-1")],
            x_value: 0,
        });

        assert!(result.is_ok(), "controller should be able to target own Hexproof creature");
    }

    // ---- Shroud (CR 702.18) ---------------------------------------------

    #[test]
    fn shroud_creature_cannot_be_targeted_by_opponent() {
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;

        let (mut game, p1, p2) = make_game_in_first_main();

        let shroud_bear = CardDefinition::new("sh-bear", "Shroud Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_static_ability(StaticAbility::Shroud);
        let card = CardInstance::new("sh-bear-1", shroud_bear, &p2);
        add_permanent_to_battlefield(&mut game, &p2, card);

        let bolt = CardDefinition::new("bolt", "Bolt", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::Creature);
        add_card_to_hand(&mut game, &p1, CardInstance::new("bolt-1", bolt, &p1));
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("bolt-1"),
            targets: vec![Target::creature("sh-bear-1")],
            x_value: 0,
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("shroud"));
    }

    #[test]
    fn shroud_creature_cannot_be_targeted_by_controller_either() {
        use crate::domain::game::test_helpers::add_permanent_to_battlefield;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let shroud_bear = CardDefinition::new("sh-bear", "Shroud Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2)
            .with_static_ability(StaticAbility::Shroud);
        let card = CardInstance::new("sh-bear-1", shroud_bear, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        let growth = CardDefinition::new("growth", "Giant Growth", vec![CardType::Instant])
            .with_target_requirement(TargetRequirement::Creature);
        add_card_to_hand(&mut game, &p1, CardInstance::new("growth-1", growth, &p1));
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("growth-1"),
            targets: vec![Target::creature("sh-bear-1")],
            x_value: 0,
        });

        assert!(result.is_err(), "controller should NOT be able to target Shroud creature");
    }

    // ---- X costs (MA1) ---------------------------------------------------

    fn make_x_cost_sorcery(instance_id: &str, owner_id: &str, cost: &str) -> CardInstance {
        let mana_cost = ManaCost::parse(cost).unwrap();
        let def = CardDefinition::new("fireball", "Fireball", vec![CardType::Sorcery])
            .with_mana_cost(mana_cost);
        CardInstance::new(instance_id, def, owner_id)
    }

    #[test]
    fn cast_x_spell_with_x_value_deducts_total_cost() {
        // XR with x_value=3: total cost = 3 (X) + 1 (R) = 4 mana
        let (mut game, p1, _) = make_game_in_first_main();
        let spell = make_x_cost_sorcery("fireball-1", &p1, "XR");
        add_card_to_hand(&mut game, &p1, spell);

        // Provide 4 mana: 3 colorless + 1 red
        game.add_mana(&p1, ManaColor::Colorless, 3).unwrap();
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 4);

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("fireball-1"),
            targets: vec![],
            x_value: 3,
        })
        .unwrap();

        // All mana spent
        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
        assert_eq!(game.stack().len(), 1);
    }

    #[test]
    fn cast_x_spell_fails_when_insufficient_mana_for_x() {
        // XR with x_value=3: total cost = 4, but player only has 2 mana
        let (mut game, p1, _) = make_game_in_first_main();
        let spell = make_x_cost_sorcery("fireball-1", &p1, "XR");
        add_card_to_hand(&mut game, &p1, spell);

        game.add_mana(&p1, ManaColor::Colorless, 1).unwrap();
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("fireball-1"),
            targets: vec![],
            x_value: 3,
        });

        assert!(result.is_err(), "should fail with insufficient mana");
    }

    #[test]
    fn cast_x_spell_stores_x_value_on_stack() {
        // XR with x_value=2: spell on stack should have x_value=2
        let (mut game, p1, _) = make_game_in_first_main();
        let spell = make_x_cost_sorcery("fireball-1", &p1, "XR");
        add_card_to_hand(&mut game, &p1, spell);

        game.add_mana(&p1, ManaColor::Colorless, 2).unwrap();
        game.add_mana(&p1, ManaColor::Red, 1).unwrap();

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("fireball-1"),
            targets: vec![],
            x_value: 2,
        })
        .unwrap();

        // Check x_value is stored on the stack item
        use crate::domain::entities::the_stack::StackItem;
        let top = game.stack().last().unwrap();
        let StackItem::Spell(spell_on_stack) = top else {
            panic!("expected spell on stack");
        };
        assert_eq!(spell_on_stack.x_value, 2);
    }

    #[test]
    fn cast_non_x_spell_with_default_x_value_zero_works() {
        // Regular spell: x_value=0 should behave normally (backward compat)
        let (mut game, p1, _) = make_game_in_first_main();
        let spell = make_sorcery_with_cost("s1", &p1, "2");
        add_card_to_hand(&mut game, &p1, spell);
        game.add_mana(&p1, ManaColor::Colorless, 2).unwrap();

        game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("s1"),
            targets: vec![],
            x_value: 0,
        })
        .unwrap();

        assert_eq!(game.mana_pool(&p1).unwrap().total(), 0);
    }

    // =========================================================================
    // Layer-system bypass tests (LS1 fixes) — targeting
    // =========================================================================

    /// A creature with Shroud that has all abilities removed via the layer system
    /// should be targetable (Shroud no longer applies).
    ///
    /// Verifies that the Shroud check in `validate_permanent_target_with_type`
    /// consults effective_abilities rather than the card definition.
    #[test]
    fn shroud_removed_by_layer_system_allows_targeting() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, p2) = make_game_in_first_main();

        // p2 has a Shroud creature on the battlefield.
        let shroud_creature = {
            let def = CardDefinition::new("shroud-creature", "Shroud Creature", vec![CardType::Creature])
                .with_power_toughness(2, 2)
                .with_static_ability(StaticAbility::Shroud);
            CardInstance::new("shroud-1", def, &p2)
        };
        add_permanent_to_battlefield(&mut game, &p2, shroud_creature);

        // Apply RemoveAllAbilities via layer system (simulates "Turn to Frog").
        let remove_abilities = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::RemoveAllAbilities,
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 200,
            source_id: "frog-effect".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["shroud-1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(remove_abilities);

        // Verify Shroud is no longer in effective abilities.
        let abilities = game.effective_abilities("shroud-1").expect("should find creature");
        assert!(
            !abilities.contains(&StaticAbility::Shroud),
            "effective_abilities must not include Shroud after RemoveAllAbilities"
        );

        // p1 casts a removal spell targeting p2's creature — should succeed now that Shroud is gone.
        let removal = make_instant_targeting_creature("removal-1", &p1);
        add_card_to_hand(&mut game, &p1, removal);

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("removal-1"),
            targets: vec![Target::creature("shroud-1")],
            x_value: 0,
        });

        assert!(
            result.is_ok(),
            "Should be able to target creature after Shroud is removed by the layer system"
        );
    }

    /// A creature with Hexproof that has all abilities removed via the layer system
    /// should be targetable by opponents.
    #[test]
    fn hexproof_removed_by_layer_system_allows_opponent_targeting() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, p2) = make_game_in_first_main();

        // p2 has a Hexproof creature on the battlefield.
        let hexproof_creature = {
            let def = CardDefinition::new("hexproof-creature", "Hexproof Creature", vec![CardType::Creature])
                .with_power_toughness(2, 2)
                .with_static_ability(StaticAbility::Hexproof);
            CardInstance::new("hexproof-1", def, &p2)
        };
        add_permanent_to_battlefield(&mut game, &p2, hexproof_creature);

        // Apply RemoveAllAbilities via layer system.
        let remove_abilities = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::RemoveAllAbilities,
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 200,
            source_id: "frog-effect".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["hexproof-1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(remove_abilities);

        // p1 (opponent) targets p2's creature — should succeed now Hexproof is gone.
        let removal = make_instant_targeting_creature("removal-1", &p1);
        add_card_to_hand(&mut game, &p1, removal);

        let result = game.apply(Action::CastSpell {
            player_id: PlayerId::new(&p1),
            card_id: CardInstanceId::new("removal-1"),
            targets: vec![Target::creature("hexproof-1")],
            x_value: 0,
        });

        assert!(
            result.is_ok(),
            "Should be able to target creature after Hexproof is removed by the layer system"
        );
    }

    // TR6: SpellCast event is emitted when a spell is cast
    #[test]
    fn cast_spell_emits_spell_cast_event() {
        use crate::domain::events::GameEvent;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let creature = make_creature_card("bear-1", &p1, 2, 2);
        let def_with_cost = CardInstance::new(
            "bear-1",
            CardDefinition::new("bear", "Bear", vec![CardType::Creature])
                .with_power_toughness(2, 2)
                .with_mana_cost(ManaCost::parse("G").unwrap()),
            &p1,
        );
        add_card_to_hand(&mut game, &p1, def_with_cost);
        game.add_mana(&p1, ManaColor::Green, 1).unwrap();

        let events = game
            .apply(Action::CastSpell {
                player_id: PlayerId::new(&p1),
                card_id: CardInstanceId::new("bear-1"),
                targets: vec![],
                x_value: 0,
            })
            .unwrap();

        let spell_cast_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, GameEvent::SpellCast { .. }))
            .collect();
        assert_eq!(
            spell_cast_events.len(),
            1,
            "Expected exactly 1 SpellCast event when casting a spell"
        );

        if let GameEvent::SpellCast { card_id, controller_id, .. } = &spell_cast_events[0] {
            assert_eq!(card_id.as_str(), "bear-1");
            assert_eq!(controller_id.as_str(), &p1);
        }
    }
}
