//! Simple greedy bot for P2.
//!
//! This module provides a single entry point, `run_bot_turn`, which drives an
//! automated player through their priority windows without any look-ahead.
//!
//! # Strategy
//!
//! 1. If the bot does not currently hold priority, return immediately.
//! 2. Play a land if one is available (picks the first playable land).
//! 3. Tap all untapped lands to fill the mana pool.
//! 4. Cast the first castable spell (repeat until no spells can be cast).
//!    For spells that require a target, the bot targets the opponent.
//! 5. Declare all attackable creatures as attackers (DeclareAttackers step).
//! 6. Pass priority when nothing else can be done.
//! 7. Loop (max 50 iterations) until the bot no longer holds priority.
//!
//! The bot never blocks — it simply passes priority during the DeclareBlockers
//! step just like any other step where it has no castable spells.

use tracing::{debug, warn};

use crate::domain::actions::Action;
use crate::domain::targets::Target;
use crate::domain::types::{CardInstanceId, PlayerId};
use crate::infrastructure::legal_actions::compute_legal_actions;

use super::Game;

/// Maximum number of iterations the bot loop will run before giving up.
const MAX_BOT_ITERATIONS: u32 = 50;

/// Drive the bot player through all current priority windows.
///
/// Applies actions greedily to `game` on behalf of `bot_player_id`.  Returns
/// the number of actions the bot successfully applied.
///
/// # Panics
///
/// Never panics — all errors are logged as warnings and the loop continues.
pub fn run_bot_turn(game: &mut Game, bot_player_id: &str) -> u32 {
    let mut actions_taken: u32 = 0;

    for iteration in 0..MAX_BOT_ITERATIONS {
        // Only act when the bot holds priority.
        if game.priority_player_id() != Some(bot_player_id) {
            debug!(
                iteration,
                bot = %bot_player_id,
                priority = ?game.priority_player_id(),
                "bot: does not have priority, stopping"
            );
            break;
        }

        let allowed = compute_legal_actions(game, bot_player_id);

        // 1. Play a land if available.
        if let Some(land_id) = allowed.playable_lands.first().cloned() {
            debug!(iteration, bot = %bot_player_id, land = %land_id, "bot: playing land");
            if let Err(err) = game.apply(Action::PlayLand {
                player_id: PlayerId::new(bot_player_id),
                card_id: CardInstanceId::new(&land_id),
            }) {
                warn!(iteration, %err, "bot: PlayLand failed");
            } else {
                actions_taken += 1;
            }
            continue;
        }

        // 2. Tap all tappable lands for mana before trying to cast.
        if !allowed.tappable_lands.is_empty() {
            for land_id in &allowed.tappable_lands {
                debug!(iteration, bot = %bot_player_id, land = %land_id, "bot: tapping land for mana");
                if let Err(err) = game.apply(Action::ActivateAbility {
                    player_id: PlayerId::new(bot_player_id),
                    permanent_id: CardInstanceId::new(land_id),
            ability_index: 0,
                }) {
                    warn!(iteration, %err, "bot: ActivateAbility (tap land) failed");
                } else {
                    actions_taken += 1;
                }
            }
            // Re-compute actions after tapping.
            continue;
        }

        // 3. Cast first castable spell.
        if let Some(spell_id) = allowed.castable_spells.first().cloned() {
            let targets = if allowed.spells_needing_targets.contains(&spell_id) {
                // Target the opponent for spells that need a target.
                match game.opponent_of(bot_player_id) {
                    Ok(opp_id) => vec![Target::player(opp_id.to_owned())],
                    Err(err) => {
                        warn!(iteration, %err, "bot: cannot determine opponent for targeting");
                        vec![]
                    }
                }
            } else {
                vec![]
            };
            debug!(
                iteration,
                bot = %bot_player_id,
                spell = %spell_id,
                has_targets = !targets.is_empty(),
                "bot: casting spell"
            );
            if let Err(err) = game.apply(Action::CastSpell {
                player_id: PlayerId::new(bot_player_id),
                card_id: CardInstanceId::new(&spell_id),
                targets,
                x_value: 0, // Bot does not choose X values
            }) {
                warn!(iteration, %err, "bot: CastSpell failed");
            } else {
                actions_taken += 1;
            }
            continue;
        }

        // 4. Declare all attackable creatures during DeclareAttackers step.
        if !allowed.attackable_creatures.is_empty() {
            for creature_id in &allowed.attackable_creatures {
                debug!(
                    iteration,
                    bot = %bot_player_id,
                    creature = %creature_id,
                    "bot: declaring attacker"
                );
                if let Err(err) = game.apply(Action::DeclareAttacker {
                    player_id: PlayerId::new(bot_player_id),
                    creature_id: CardInstanceId::new(creature_id),
                }) {
                    warn!(iteration, %err, "bot: DeclareAttacker failed");
                } else {
                    actions_taken += 1;
                }
            }
            continue;
        }

        // 5. Nothing useful to do — pass priority.
        debug!(iteration, bot = %bot_player_id, "bot: passing priority");
        if let Err(err) = game.apply(Action::PassPriority {
            player_id: PlayerId::new(bot_player_id),
        }) {
            warn!(iteration, %err, "bot: PassPriority failed");
            break;
        }
        actions_taken += 1;
        // After passing, the bot may no longer hold priority — the loop
        // condition at the top will catch that and break naturally.
    }

    debug!(
        bot = %bot_player_id,
        actions_taken,
        step = ?game.current_step(),
        priority = ?game.priority_player_id(),
        "bot: run_bot_turn finished"
    );
    actions_taken
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::cards::prebuilt_decks;
    use crate::domain::cards::{card_definition::CardDefinition, card_instance::CardInstance};
    use crate::domain::enums::{CardType, ManaColor, Step};
    use crate::domain::game::automation::run_auto_pass_loop;
    use crate::domain::game::test_helpers::{
        add_card_to_hand, add_permanent_to_battlefield, make_land_card,
    };
    use crate::domain::types::PlayerId;
    use crate::domain::value_objects::mana::ManaCost;

    /// Build a standard test game with prebuilt decks, advanced to P1's FirstMain.
    fn make_bot_game() -> (Game, String, String) {
        let mut game = Game::create("bot-test");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bot").unwrap();
        game.assign_deck("p1", prebuilt_decks::green_deck("p1")).unwrap();
        game.assign_deck("p2", prebuilt_decks::red_deck("p2")).unwrap();
        game.start("p1", Some(42)).unwrap();
        run_auto_pass_loop(&mut game);
        assert_eq!(game.current_step(), Step::FirstMain);
        (game, "p1".to_owned(), "p2".to_owned())
    }

    // ---- RED: bot plays land when available ---------------------------------

    #[test]
    fn bot_plays_land_when_available() {
        let (mut game, p1, p2) = make_bot_game();

        // Advance to P2's turn so the bot has priority in FirstMain.
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();
        run_auto_pass_loop(&mut game);
        assert_eq!(game.current_player_id(), p2.as_str());
        assert_eq!(game.current_step(), Step::FirstMain);

        let initial_bf_size = game.battlefield(&p2).unwrap().len();

        let actions = run_bot_turn(&mut game, &p2);

        // Bot should have taken at least one action (playing a land).
        assert!(actions > 0, "bot should take at least one action");

        // Bot should have played a land — battlefield should have grown.
        let final_bf_size = game.battlefield(&p2).unwrap().len();
        assert!(
            final_bf_size > initial_bf_size,
            "bot should have played a land (bf before={initial_bf_size}, after={final_bf_size})"
        );
    }

    // ---- RED: bot casts creature when it has mana ---------------------------

    #[test]
    fn bot_casts_creature_when_mana_available() {
        let (mut game, p1, p2) = make_bot_game();

        // Advance to P2's turn.
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();
        run_auto_pass_loop(&mut game);
        assert_eq!(game.current_player_id(), p2.as_str());

        // Give P2 a Goblin in hand and enough mana to cast it.
        let goblin = CardInstance::new(
            "bot-goblin",
            CardDefinition::new("goblin", "Goblin", vec![CardType::Creature])
                .with_power_toughness(1, 1)
                .with_mana_cost(ManaCost::parse("R").unwrap()),
            &p2,
        );
        add_card_to_hand(&mut game, &p2, goblin);
        game.add_mana(&p2, ManaColor::Red, 1).unwrap();

        let initial_hand_size = game.hand(&p2).unwrap().len();

        run_bot_turn(&mut game, &p2);

        // The Goblin should have been cast (removed from hand), or resolved onto BF.
        let final_hand_size = game.hand(&p2).unwrap().len();
        assert!(
            final_hand_size < initial_hand_size,
            "bot should have cast the Goblin (hand before={initial_hand_size}, after={final_hand_size})"
        );
    }

    // ---- RED: bot passes when nothing to do --------------------------------

    #[test]
    fn bot_passes_when_nothing_to_do() {
        // Give the bot priority with an empty hand and no lands to play.
        let (mut game, p1, p2) = make_bot_game();

        // Give P2 priority with no cards in hand.
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();
        run_auto_pass_loop(&mut game);

        // Clear P2's hand to guarantee nothing to do.
        let player = game.player_state_mut(&p2).unwrap();
        player.hand.clear();

        assert_eq!(game.current_player_id(), p2.as_str());
        assert_eq!(game.priority_player_id(), Some(p2.as_str()));

        let actions = run_bot_turn(&mut game, &p2);

        // Bot should have passed priority (1 action = PassPriority).
        assert!(actions > 0, "bot should pass priority (actions={actions})");
        // After passing, the bot should no longer hold priority.
        assert_ne!(
            game.priority_player_id(),
            Some(p2.as_str()),
            "bot should no longer hold priority after passing"
        );
    }

    // ---- RED: bot doesn't infinite-loop ------------------------------------

    #[test]
    fn bot_does_not_infinite_loop() {
        let (mut game, p1, p2) = make_bot_game();

        // Advance to P2's turn.
        game.apply(Action::EndTurn { player_id: PlayerId::new(&p1) }).unwrap();
        run_auto_pass_loop(&mut game);

        // Run the bot — must terminate before MAX_BOT_ITERATIONS.
        let actions = run_bot_turn(&mut game, &p2);

        // We just verify it terminates and took a bounded number of actions.
        assert!(
            actions <= MAX_BOT_ITERATIONS,
            "bot took {actions} actions but max is {MAX_BOT_ITERATIONS}"
        );
    }
}
