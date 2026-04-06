//! Game mechanic functions for the `Game` aggregate.
//!
//! Each function implements one keyword action from the MTG rules:
//! - Scry (CR 701.18)
//! - Surveil (CR 701.37)
//! - Mill (CR 701.13)
//! - Discard (CR 701.8)
//! - Fight (CR 701.14)
//! - Bolster (CR 701.39)
//! - Adapt (CR 701.46)

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;

use super::Game;

impl Game {
    /// Scry N — look at the top N cards and put any number on the bottom
    /// of the library in any order, and the rest on top in any order.
    ///
    /// Per CR 701.18. MVP: auto-scry keeps all cards on top (no-op).
    /// The `to_bottom` parameter lists instance IDs of cards to move to bottom.
    /// Cards not in `to_bottom` stay on top in their original order.
    pub(crate) fn scry(&mut self, player_id: &str, amount: usize) {
        self.scry_with_choices(player_id, amount, &[]);
    }

    /// Scry with explicit choices of which cards go to the bottom.
    pub(crate) fn scry_with_choices(
        &mut self,
        player_id: &str,
        amount: usize,
        to_bottom_ids: &[&str],
    ) {
        let Ok(player) = self.player_state_mut(player_id) else {
            return;
        };

        let n = amount.min(player.library.len());
        if n == 0 {
            return;
        }

        // Take the top N cards
        let top_n: Vec<CardInstance> = player.library.drain(..n).collect();

        // Split into keep-on-top and put-on-bottom
        let mut on_top = Vec::new();
        let mut on_bottom = Vec::new();
        for card in top_n {
            if to_bottom_ids.contains(&card.instance_id()) {
                on_bottom.push(card);
            } else {
                on_top.push(card);
            }
        }

        // Re-insert: top cards go back to front, bottom cards go to end
        for (i, card) in on_top.into_iter().enumerate() {
            player.library.insert(i, card);
        }
        for card in on_bottom {
            player.library.push(card);
        }
    }

    /// Surveil N — look at the top N cards, move chosen ones to graveyard (CR 701.37).
    ///
    /// MVP: auto-surveil sends all looked-at cards to the graveyard.
    pub(crate) fn surveil(&mut self, player_id: &str, amount: usize) {
        let top_ids: Vec<String> = self
            .player_state(player_id)
            .map(|p| {
                let n = amount.min(p.library.len());
                p.library.iter().take(n).map(|c| c.instance_id().to_owned()).collect()
            })
            .unwrap_or_default();
        self.surveil_with_choices(player_id, amount, &top_ids.iter().map(String::as_str).collect::<Vec<_>>());
    }

    /// Surveil with explicit choices of which cards go to the graveyard.
    ///
    /// Cards not in `to_graveyard_ids` stay on top of the library in their
    /// original relative order.
    pub(crate) fn surveil_with_choices(
        &mut self,
        player_id: &str,
        amount: usize,
        to_graveyard_ids: &[&str],
    ) {
        let Ok(player) = self.player_state_mut(player_id) else {
            return;
        };

        let n = amount.min(player.library.len());
        if n == 0 {
            return;
        }

        // Take the top N cards.
        let top_n: Vec<CardInstance> = player.library.drain(..n).collect();

        // Split into keep-on-top and go-to-graveyard.
        let mut on_top = Vec::new();
        let mut to_gy = Vec::new();
        for card in top_n {
            if to_graveyard_ids.contains(&card.instance_id()) {
                to_gy.push(card);
            } else {
                on_top.push(card);
            }
        }

        // Re-insert: top cards go back to front of library.
        for (i, card) in on_top.into_iter().enumerate() {
            player.library.insert(i, card);
        }
        // Surveiled-to-gy cards go to the graveyard.
        for card in to_gy {
            player.graveyard.push(card);
        }
    }

    /// Mill N — move the top N cards of a player's library to their graveyard.
    ///
    /// Per CR 701.13. If the library has fewer than N cards, mills all remaining.
    pub(crate) fn mill(&mut self, player_id: &str, amount: usize) {
        let Ok(player) = self.player_state_mut(player_id) else {
            return;
        };

        let n = amount.min(player.library.len());
        for _ in 0..n {
            let card = player.library.remove(0);
            player.graveyard.push(card);
        }
    }

    /// Discard a specific card from a player's hand to their graveyard.
    ///
    /// Per CR 701.8. Used for forced discard effects (e.g. Mind Rot).
    /// Returns `true` if the card was found and discarded.
    pub(crate) fn discard(&mut self, player_id: &str, card_id: &str) -> bool {
        let Ok(player) = self.player_state_mut(player_id) else {
            return false;
        };
        if let Some(pos) = player.hand.iter().position(|c| c.instance_id() == card_id) {
            let card = player.hand.remove(pos);
            player.graveyard.push(card);
            true
        } else {
            false
        }
    }

    /// Discard the last N cards from a player's hand (LIFO order).
    ///
    /// Discards deterministically from the end of the hand vector.
    /// Used by effects like "discard 2 cards".
    pub(crate) fn discard_from_end(&mut self, player_id: &str, amount: usize) {
        // Collect IDs of the last N cards (MVP: no random, just last cards)
        let ids: Vec<String> = self
            .player_state(player_id)
            .map(|p| {
                let n = amount.min(p.hand.len());
                p.hand.iter().rev().take(n).map(|c| c.instance_id().to_owned()).collect()
            })
            .unwrap_or_default();
        for id in ids {
            self.discard(player_id, &id);
        }
    }

    /// Fight mechanic: each creature deals damage equal to its power to the other (CR 701.14).
    ///
    /// Both creatures must be on the battlefield and have creature state.
    /// Damage is marked simultaneously (not sequential).
    ///
    /// # Errors
    ///
    /// Returns `GameError::InvalidTarget` if either creature is not found or
    /// does not have creature state.
    pub fn fight(
        &mut self,
        creature_a_id: &str,
        creature_b_id: &str,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Validate both permanents exist and are creatures, then read effective power
        // via the layer system (CR 701.14, LS1: power is layer-evaluated).
        if !self.permanent_states.contains_key(creature_a_id) {
            return Err(GameError::InvalidTarget {
                reason: format!("creature '{}' not found on battlefield", creature_a_id),
            });
        }
        if !self.permanent_states.contains_key(creature_b_id) {
            return Err(GameError::InvalidTarget {
                reason: format!("creature '{}' not found on battlefield", creature_b_id),
            });
        }

        let power_a = self.effective_power(creature_a_id)
            .ok_or_else(|| GameError::InvalidTarget {
                reason: format!("'{}' is not a creature", creature_a_id),
            })?;

        let power_b = self.effective_power(creature_b_id)
            .ok_or_else(|| GameError::InvalidTarget {
                reason: format!("'{}' is not a creature", creature_b_id),
            })?;

        // Get deathtouch flags via the layer pipeline so Layer 6 effects
        // (e.g. RemoveAllAbilities) are respected (CR 613.1f).
        let a_has_deathtouch = self
            .effective_abilities(creature_a_id)
            .map(|a| a.contains(&crate::domain::enums::StaticAbility::Deathtouch))
            .unwrap_or(false);
        let b_has_deathtouch = self
            .effective_abilities(creature_b_id)
            .map(|a| a.contains(&crate::domain::enums::StaticAbility::Deathtouch))
            .unwrap_or(false);

        // Deal damage simultaneously.
        self.mark_damage_on_creature(creature_b_id, power_a, a_has_deathtouch);
        self.mark_damage_on_creature(creature_a_id, power_b, b_has_deathtouch);

        // Run SBAs to destroy any creatures with lethal damage.
        let sba_events = self.perform_state_based_actions();
        Ok(sba_events)
    }

    /// Bolster N: put N +1/+1 counters on the creature you control with the
    /// least toughness (CR 701.39).
    ///
    /// If there is a tie in least toughness, the first creature found is chosen.
    /// Does nothing if the player controls no creatures.
    ///
    /// # Errors
    ///
    /// Returns `GameError::InvalidTarget` if the player does not exist.
    pub fn bolster(
        &mut self,
        player_id: &str,
        amount: u32,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Validate player exists.
        let player = self.player_state(player_id)?;

        // Collect (instance_id, effective_toughness) for all creatures the player controls.
        let creature_ids: Vec<String> = player.battlefield
            .iter()
            .filter(|c| c.definition().is_creature())
            .map(|c| c.instance_id().to_owned())
            .collect();

        if creature_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find creature with least toughness using the layer system (CR 701.39, LS1).
        let target_id = creature_ids
            .into_iter()
            .min_by_key(|id| {
                self.effective_toughness(id.as_str())
                    .unwrap_or(i32::MAX)
            });

        let Some(target_id) = target_id else {
            return Ok(Vec::new());
        };

        // Add N +1/+1 counters to the target.
        if let Some(state) = self.permanent_states.get(&target_id).cloned() {
            let new_state = state.add_counters("PLUS_ONE_PLUS_ONE", amount);
            self.permanent_states.insert(target_id, new_state);
        }

        Ok(Vec::new())
    }

    /// Adapt N: if this creature has no +1/+1 counters on it, put N +1/+1
    /// counters on it (CR 701.46).
    ///
    /// Does nothing if the creature already has one or more +1/+1 counters.
    ///
    /// # Errors
    ///
    /// Returns `GameError::InvalidTarget` if the permanent is not found or
    /// does not have creature state.
    pub fn adapt(
        &mut self,
        permanent_id: &str,
        amount: u32,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Validate permanent exists.
        let state = self.permanent_states.get(permanent_id)
            .ok_or_else(|| GameError::InvalidTarget {
                reason: format!("permanent '{}' not found on battlefield", permanent_id),
            })?
            .clone();

        // Validate it is a creature via the layer system (LS1).
        self.effective_power(permanent_id)
            .ok_or_else(|| GameError::InvalidTarget {
                reason: format!("'{}' is not a creature", permanent_id),
            })?;

        // CR 701.46: Only put counters if the creature has NO +1/+1 counters.
        if state.get_counters("PLUS_ONE_PLUS_ONE") > 0 {
            return Ok(Vec::new());
        }

        let new_state = state.add_counters("PLUS_ONE_PLUS_ONE", amount);
        self.permanent_states.insert(permanent_id.to_owned(), new_state);

        Ok(Vec::new())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::domain::game::test_helpers::*;

    // ---- Scry (CR 701.18) -----------------------------------------------

    #[test]
    fn scry_keeps_all_on_top_by_default() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        // Add 3 cards to library
        for i in 0..3 {
            let card = CardInstance::new(
                format!("card-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            game.add_card_to_library_top(&p1, card).unwrap();
        }
        // Library order: card-2, card-1, card-0 (last inserted = top)
        assert_eq!(game.library_count(&p1).unwrap(), 3);

        game.scry(&p1, 2);

        // All cards still there, same count
        assert_eq!(game.library_count(&p1).unwrap(), 3);
    }

    #[test]
    fn scry_with_choices_moves_selected_to_bottom() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        for i in 0..4 {
            let card = CardInstance::new(
                format!("card-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            game.add_card_to_library_top(&p1, card).unwrap();
        }
        // Library: card-3 (top), card-2, card-1, card-0 (bottom)

        // Scry 2, put card-3 on bottom
        game.scry_with_choices(&p1, 2, &["card-3"]);

        // card-2 should now be on top (card-3 went to bottom)
        let player = game.player_state(&p1).unwrap();
        assert_eq!(player.library[0].instance_id(), "card-2");
        // card-3 should be at the bottom
        assert_eq!(player.library.last().unwrap().instance_id(), "card-3");
        assert_eq!(player.library.len(), 4);
    }

    #[test]
    fn scry_zero_is_noop() {
        let (mut game, p1, _p2) = make_started_game();
        game.scry(&p1, 0);
        assert_eq!(game.library_count(&p1).unwrap(), 0);
    }

    #[test]
    fn scry_more_than_library_size_only_looks_at_available() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        let card = CardInstance::new(
            "only-card",
            CardDefinition::new("forest", "Forest", vec![CardType::Land]),
            &p1,
        );
        game.add_card_to_library_top(&p1, card).unwrap();

        game.scry(&p1, 5); // Only 1 card, scry 5

        assert_eq!(game.library_count(&p1).unwrap(), 1);
    }

    // ---- Mill (CR 701.13) -----------------------------------------------

    #[test]
    fn mill_moves_top_cards_to_graveyard() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        for i in 0..4 {
            let card = CardInstance::new(
                format!("card-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            game.add_card_to_library_top(&p1, card).unwrap();
        }
        assert_eq!(game.graveyard(&p1).unwrap().len(), 0);

        game.mill(&p1, 2);

        assert_eq!(game.library_count(&p1).unwrap(), 2);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 2);
    }

    #[test]
    fn mill_more_than_library_mills_all() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        let card = CardInstance::new(
            "only-card",
            CardDefinition::new("forest", "Forest", vec![CardType::Land]),
            &p1,
        );
        game.add_card_to_library_top(&p1, card).unwrap();

        game.mill(&p1, 5);

        assert_eq!(game.library_count(&p1).unwrap(), 0);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    #[test]
    fn mill_zero_is_noop() {
        let (mut game, p1, _p2) = make_started_game();
        game.mill(&p1, 0);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 0);
    }

    // ---- Discard (CR 701.8) ---------------------------------------------

    #[test]
    fn discard_moves_specific_card_to_graveyard() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        let card = CardInstance::new(
            "spell-1",
            CardDefinition::new("bolt", "Bolt", vec![CardType::Instant]),
            &p1,
        );
        add_card_to_hand(&mut game, &p1, card);
        assert_eq!(game.hand(&p1).unwrap().len(), 1);

        let result = game.discard(&p1, "spell-1");

        assert!(result);
        assert_eq!(game.hand(&p1).unwrap().len(), 0);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    #[test]
    fn discard_nonexistent_card_returns_false() {
        let (mut game, p1, _p2) = make_started_game();
        assert!(!game.discard(&p1, "nope"));
    }

    #[test]
    fn discard_from_end_removes_n_cards() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        for i in 0..3 {
            let card = CardInstance::new(
                format!("card-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            add_card_to_hand(&mut game, &p1, card);
        }

        game.discard_from_end(&p1, 2);

        assert_eq!(game.hand(&p1).unwrap().len(), 1);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 2);
    }

    #[test]
    fn discard_from_end_more_than_hand_discards_all() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        let card = CardInstance::new(
            "only",
            CardDefinition::new("forest", "Forest", vec![CardType::Land]),
            &p1,
        );
        add_card_to_hand(&mut game, &p1, card);

        game.discard_from_end(&p1, 5);

        assert_eq!(game.hand(&p1).unwrap().len(), 0);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
    }

    // ---- Surveil (CR 701.37) -----------------------------------------------

    #[test]
    fn surveil_sends_all_to_graveyard_by_default() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        for i in 0..3 {
            let card = CardInstance::new(
                format!("card-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            game.add_card_to_library_top(&p1, card).unwrap();
        }
        assert_eq!(game.graveyard(&p1).unwrap().len(), 0);

        game.surveil(&p1, 2);

        assert_eq!(game.library_count(&p1).unwrap(), 1, "2 surveiled cards should leave library");
        assert_eq!(game.graveyard(&p1).unwrap().len(), 2, "2 cards should go to graveyard");
    }

    #[test]
    fn surveil_with_choices_keeps_selected_on_top() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();
        // Library (top to bottom): card-0, card-1, card-2
        for i in (0..3).rev() {
            let card = CardInstance::new(
                format!("card-{i}"),
                CardDefinition::new("forest", "Forest", vec![CardType::Land]),
                &p1,
            );
            game.add_card_to_library_top(&p1, card).unwrap();
        }

        // Surveil 2: send card-1 to graveyard, keep card-0 on top
        game.surveil_with_choices(&p1, 2, &["card-1"]);

        assert_eq!(game.library_count(&p1).unwrap(), 2, "card-0 stays, card-2 untouched");
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
        assert_eq!(game.graveyard(&p1).unwrap()[0].instance_id(), "card-1");
        // card-0 should remain on top
        let player = game.player_state(&p1).unwrap();
        assert_eq!(player.library[0].instance_id(), "card-0");
    }

    #[test]
    fn surveil_zero_is_noop() {
        let (mut game, p1, _p2) = make_started_game();
        game.surveil(&p1, 0);
        assert_eq!(game.graveyard(&p1).unwrap().len(), 0);
    }

    // ---- Fight mechanic (P10.12) -------------------------------------------

    #[test]
    fn fight_deals_damage_to_both_creatures() {
        let (mut game, p1, p2) = make_started_game();
        let attacker = make_creature_card("a1", &p1, 3, 3);
        let defender = make_creature_card("d1", &p2, 2, 4);
        add_permanent_to_battlefield(&mut game, &p1, attacker);
        add_permanent_to_battlefield(&mut game, &p2, defender);

        game.fight("a1", "d1").expect("fight should succeed");

        // a1 (3/3) takes 2 damage from d1's power
        let a1_state = game.permanent_state("a1").unwrap();
        assert_eq!(
            a1_state.creature_state().unwrap().damage_marked_this_turn(),
            2,
            "a1 should take 2 damage from d1's power"
        );

        // d1 (2/4) takes 3 damage from a1's power
        let d1_state = game.permanent_state("d1").unwrap();
        assert_eq!(
            d1_state.creature_state().unwrap().damage_marked_this_turn(),
            3,
            "d1 should take 3 damage from a1's power"
        );
    }

    #[test]
    fn fight_creature_dies_from_lethal_damage() {
        let (mut game, p1, p2) = make_started_game();
        // 4-power creature fights a 2/2
        let big = make_creature_card("big", &p1, 4, 4);
        let small = make_creature_card("small", &p2, 1, 2);
        add_permanent_to_battlefield(&mut game, &p1, big);
        add_permanent_to_battlefield(&mut game, &p2, small);

        game.fight("big", "small").expect("fight should succeed");

        // "small" should be dead (4 damage > 2 toughness)
        assert!(
            game.battlefield(&p2).unwrap().is_empty(),
            "small creature should have died from fight"
        );
        // "big" should survive (1 damage, toughness 4)
        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
    }

    #[test]
    fn fight_both_creatures_die() {
        let (mut game, p1, p2) = make_started_game();
        let a = make_creature_card("ca", &p1, 3, 1);
        let b = make_creature_card("cb", &p2, 3, 1);
        add_permanent_to_battlefield(&mut game, &p1, a);
        add_permanent_to_battlefield(&mut game, &p2, b);

        game.fight("ca", "cb").expect("fight should succeed");

        assert!(
            game.battlefield(&p1).unwrap().is_empty(),
            "creature a should have died"
        );
        assert!(
            game.battlefield(&p2).unwrap().is_empty(),
            "creature b should have died"
        );
    }

    #[test]
    fn fight_missing_creature_returns_error() {
        let (mut game, p1, _p2) = make_started_game();
        let a = make_creature_card("ca", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, a);

        let result = game.fight("ca", "nonexistent");
        assert!(result.is_err(), "fight with missing creature should fail");
    }

    // ---- Bolster N (P15.6) -------------------------------------------------

    #[test]
    fn bolster_puts_counters_on_lowest_toughness_creature() {
        let (mut game, p1, _p2) = make_started_game();
        let high = make_creature_card("high", &p1, 2, 4);
        let low = make_creature_card("low", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, high);
        add_permanent_to_battlefield(&mut game, &p1, low);

        game.bolster(&p1, 2).expect("bolster should succeed");

        // "low" (2/2) should get the counters
        let low_state = game.permanent_state("low").unwrap();
        assert_eq!(
            low_state.get_counters("PLUS_ONE_PLUS_ONE"),
            2,
            "lowest toughness creature should get the +1/+1 counters"
        );

        // "high" (2/4) should NOT get counters
        let high_state = game.permanent_state("high").unwrap();
        assert_eq!(
            high_state.get_counters("PLUS_ONE_PLUS_ONE"),
            0,
            "higher toughness creature should not get counters"
        );
    }

    #[test]
    fn bolster_does_nothing_with_no_creatures() {
        let (mut game, p1, _p2) = make_started_game();
        // No creatures on the battlefield
        let result = game.bolster(&p1, 2);
        assert!(result.is_ok(), "bolster with no creatures should succeed (no-op)");
    }

    #[test]
    fn bolster_on_single_creature() {
        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        game.bolster(&p1, 3).expect("bolster should succeed");

        let state = game.permanent_state("c1").unwrap();
        assert_eq!(state.get_counters("PLUS_ONE_PLUS_ONE"), 3);
    }

    // ---- Adapt N (P15.7) ---------------------------------------------------

    #[test]
    fn adapt_adds_counters_when_creature_has_none() {
        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        game.adapt("c1", 3).expect("adapt should succeed");

        let state = game.permanent_state("c1").unwrap();
        assert_eq!(
            state.get_counters("PLUS_ONE_PLUS_ONE"),
            3,
            "adapt should add 3 +1/+1 counters when creature has none"
        );
    }

    #[test]
    fn adapt_does_nothing_when_creature_already_has_counters() {
        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Pre-existing counters
        {
            let state = game.permanent_state("c1").unwrap().clone();
            let new_state = state.add_counters("PLUS_ONE_PLUS_ONE", 1);
            game.permanent_states.insert("c1".to_owned(), new_state);
        }

        game.adapt("c1", 3).expect("adapt should succeed");

        let state = game.permanent_state("c1").unwrap();
        assert_eq!(
            state.get_counters("PLUS_ONE_PLUS_ONE"),
            1,
            "adapt should not add counters when creature already has +1/+1 counters"
        );
    }

    #[test]
    fn adapt_on_missing_creature_returns_error() {
        let (mut game, _p1, _p2) = make_started_game();
        let result = game.adapt("nonexistent", 2);
        assert!(result.is_err(), "adapt on missing creature should fail");
    }

    // ---- LS1 migration: fight/bolster/adapt use effective P/T ----------------

    #[test]
    fn fight_uses_effective_power_from_layer_system() {
        // A 1/1 creature pumped to 3/1 by a Layer 7c effect should deal 3 damage
        // in fight, not 1 (the base power).
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, p2) = make_started_game();
        let pumped = make_creature_card("pumped", &p1, 1, 1);
        let target = make_creature_card("target", &p2, 2, 4);
        add_permanent_to_battlefield(&mut game, &p1, pumped);
        add_permanent_to_battlefield(&mut game, &p2, target);

        // +2/+0 pump via Layer 7c
        let pump = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(2, 0),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "pump-spell".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["pumped".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(pump);

        // Effective power of "pumped" should be 3 now.
        assert_eq!(game.effective_power("pumped"), Some(3));

        game.fight("pumped", "target").expect("fight should succeed");

        // "target" (2/4) should have taken 3 damage (effective power of "pumped"),
        // not 1 (its base power).
        let target_state = game.permanent_state("target").unwrap();
        assert_eq!(
            target_state.creature_state().unwrap().damage_marked_this_turn(),
            3,
            "fight should use effective power (3) from layer system, not base power (1)"
        );
    }

    #[test]
    fn bolster_uses_effective_toughness_from_layer_system() {
        // "low" has base 2/2, but a Layer 7b effect sets it to 2/5 (effective).
        // "high" has base 2/4.
        // Without layer awareness, bolster would pick "low" (2 base toughness).
        // With layer awareness, bolster should pick "high" (4 effective toughness < 5 effective toughness).
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        let low = make_creature_card("low", &p1, 2, 2);
        let high = make_creature_card("high", &p1, 2, 4);
        add_permanent_to_battlefield(&mut game, &p1, low);
        add_permanent_to_battlefield(&mut game, &p1, high);

        // Layer 7b: set "low" toughness to 5 → effective toughness becomes 5
        let boost = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(2, 5),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "boost-spell".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["low".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(boost);

        // Effective toughness: "high" = 4, "low" = 5
        assert_eq!(game.effective_toughness("high"), Some(4));
        assert_eq!(game.effective_toughness("low"), Some(5));

        game.bolster(&p1, 2).expect("bolster should succeed");

        // Bolster should target "high" (4 effective toughness < 5 effective toughness of "low").
        let high_state = game.permanent_state("high").unwrap();
        assert_eq!(
            high_state.get_counters("PLUS_ONE_PLUS_ONE"),
            2,
            "bolster should target the creature with least effective toughness (high=4 < low=5)"
        );
        let low_state = game.permanent_state("low").unwrap();
        assert_eq!(
            low_state.get_counters("PLUS_ONE_PLUS_ONE"),
            0,
            "low's effective toughness (5) is greater, so it should not get counters"
        );
    }
}
