//! Zone transition helpers for the `Game` aggregate.
//!
//! Methods for moving cards between zones: entering the battlefield,
//! moving permanents to the graveyard, and tapping/untapping permanents.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{GraveyardReason, ZoneName};
use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
use crate::domain::value_objects::permanent_state::{EffectDuration, PermanentState};

use super::Game;

impl Game {
    /// Enter a permanent onto a player's battlefield and initialize its state.
    ///
    /// This is the single entry point for ALL permanents entering the battlefield.
    /// After moving the card:
    /// 1. Initializes permanent state (creature or non-creature).
    /// 2. Evaluates ETB triggers.
    pub(crate) fn enter_battlefield(
        &mut self,
        permanent: CardInstance,
        controller_id: &str,
        from_zone: ZoneName,
    ) -> Vec<GameEvent> {
        let snapshot = CardInstanceSnapshot {
            instance_id: CardInstanceId::new(permanent.instance_id()),
            definition_id: CardDefinitionId::new(permanent.definition().id()),
            owner_id: PlayerId::new(permanent.owner_id()),
        };

        // Assign an ETB timestamp (CR 613.7d): monotonically increasing.
        let etb_timestamp = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;

        // Initialize permanent state
        let enters_tapped = permanent
            .definition()
            .has_static_ability(crate::domain::enums::StaticAbility::EntersTapped);

        if permanent.definition().is_creature() {
            let power = permanent.definition().power().unwrap_or(0) as i32;
            let toughness = permanent.definition().toughness().unwrap_or(0) as i32;
            let state = PermanentState::for_creature(power, toughness)
                .with_etb_timestamp(etb_timestamp);
            let state = if enters_tapped { state.with_tapped(true) } else { state };
            self.permanent_states.insert(
                permanent.instance_id().to_owned(),
                state,
            );
        } else if self.is_permanent_type(&permanent) {
            let state = PermanentState::for_non_creature()
                .with_etb_timestamp(etb_timestamp);
            let state = if enters_tapped { state.with_tapped(true) } else { state };
            self.permanent_states.insert(
                permanent.instance_id().to_owned(),
                state,
            );
        }

        // Add to battlefield
        if let Ok(player) = self.player_state_mut(controller_id) {
            player.battlefield.push(permanent);
        }

        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone,
            to_zone: ZoneName::Battlefield,
            controller_id: PlayerId::new(controller_id),
        };
        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);
        vec![event]
    }

    /// Move a permanent from any battlefield to its owner's graveyard.
    ///
    /// Cleans up permanent state and evaluates dies/LTB triggers.
    pub(crate) fn move_permanent_to_graveyard(
        &mut self,
        permanent_id: &str,
        _reason: GraveyardReason,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Find which player controls this permanent and build the snapshot
        // before removing it so we have all card data available for event
        // construction and trigger collection.
        let (controller_id, card_idx) = {
            let mut found = None;
            for player in &self.players {
                if let Some(idx) = player
                    .battlefield
                    .iter()
                    .position(|c| c.instance_id() == permanent_id)
                {
                    found = Some((player.player_id.as_str().to_owned(), idx));
                    break;
                }
            }
            found.ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            })?
        };

        // Build the event snapshot while the card is still on the battlefield.
        let (snapshot, owner_id) = {
            let player = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() == controller_id)
                .ok_or_else(|| GameError::PlayerNotFound {
                    player_id: PlayerId::new(&controller_id),
                })?;
            let card = &player.battlefield[card_idx];
            let snap = CardInstanceSnapshot {
                instance_id: CardInstanceId::new(card.instance_id()),
                definition_id: CardDefinitionId::new(card.definition().id()),
                owner_id: PlayerId::new(card.owner_id()),
            };
            (snap, card.owner_id().to_owned())
        };

        // Build the zone-change event now so we can collect triggers BEFORE
        // removing the card from the battlefield. This is necessary because
        // trigger collection (`find_matching_triggers`) iterates battlefield
        // permanents — if we remove first, death-triggered cards won't be seen.
        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone: ZoneName::Battlefield,
            to_zone: ZoneName::Graveyard,
            controller_id: PlayerId::new(&controller_id),
        };

        // Collect triggers while the source permanent is still on the battlefield.
        let triggered = self.collect_triggered_abilities(&event);

        // Now perform the actual zone change.
        let card = {
            let player = self.player_state_mut(&controller_id)?;
            player.battlefield.remove(card_idx)
        };

        // Clean up permanent state.
        self.permanent_states.remove(permanent_id);

        // Remove all WhileSourceOnBattlefield global effects whose source was this permanent (LS1).
        // CR 613.7a: when a permanent leaves the battlefield, effects it generated end.
        self.remove_effects_for_source(permanent_id);

        // Add to owner's graveyard.
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.graveyard.push(card);
        }

        // Execute triggers now that the card is in the graveyard (so CLIPS can
        // see the updated battlefield state if needed).
        self.execute_triggered_abilities(triggered);

        Ok(vec![event])
    }

    /// Move a permanent from any battlefield to its owner's exile zone.
    ///
    /// Cleans up permanent state. Use this for effects like Path to Exile or
    /// Swords to Plowshares.
    pub(crate) fn move_permanent_to_exile(
        &mut self,
        permanent_id: &str,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Find which player controls this permanent.
        let (controller_id, card_idx) = {
            let mut found = None;
            for player in &self.players {
                if let Some(idx) = player
                    .battlefield
                    .iter()
                    .position(|c| c.instance_id() == permanent_id)
                {
                    found = Some((player.player_id.as_str().to_owned(), idx));
                    break;
                }
            }
            found.ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            })?
        };

        // Build the event snapshot while the card is still on the battlefield.
        let (snapshot, owner_id) = {
            let player = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() == controller_id)
                .ok_or_else(|| GameError::PlayerNotFound {
                    player_id: PlayerId::new(&controller_id),
                })?;
            let card = &player.battlefield[card_idx];
            let snap = CardInstanceSnapshot {
                instance_id: CardInstanceId::new(card.instance_id()),
                definition_id: CardDefinitionId::new(card.definition().id()),
                owner_id: PlayerId::new(card.owner_id()),
            };
            (snap, card.owner_id().to_owned())
        };

        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone: ZoneName::Battlefield,
            to_zone: ZoneName::Exile,
            controller_id: PlayerId::new(&controller_id),
        };

        // Collect triggers while the source permanent is still on the battlefield.
        let triggered = self.collect_triggered_abilities(&event);

        // Perform the zone change.
        let card = {
            let player = self.player_state_mut(&controller_id)?;
            player.battlefield.remove(card_idx)
        };

        // Clean up permanent state.
        self.permanent_states.remove(permanent_id);

        // Remove all WhileSourceOnBattlefield global effects whose source was this permanent (LS1).
        self.remove_effects_for_source(permanent_id);

        // Add to owner's exile zone.
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.exile.push(card);
        }

        self.execute_triggered_abilities(triggered);

        Ok(vec![event])
    }

    /// Return a permanent from any battlefield to its owner's hand (bounce).
    ///
    /// Removes the permanent from the battlefield, cleans up its permanent state,
    /// and places the card in its owner's hand.
    pub(crate) fn return_permanent_to_hand(
        &mut self,
        permanent_id: &str,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Find which player controls this permanent.
        let (controller_id, card_idx) = {
            let mut found = None;
            for player in &self.players {
                if let Some(idx) = player
                    .battlefield
                    .iter()
                    .position(|c| c.instance_id() == permanent_id)
                {
                    found = Some((player.player_id.as_str().to_owned(), idx));
                    break;
                }
            }
            found.ok_or_else(|| GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            })?
        };

        // Build the event snapshot while the card is still on the battlefield.
        let (snapshot, owner_id) = {
            let player = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() == controller_id)
                .ok_or_else(|| GameError::PlayerNotFound {
                    player_id: PlayerId::new(&controller_id),
                })?;
            let card = &player.battlefield[card_idx];
            let snap = CardInstanceSnapshot {
                instance_id: CardInstanceId::new(card.instance_id()),
                definition_id: CardDefinitionId::new(card.definition().id()),
                owner_id: PlayerId::new(card.owner_id()),
            };
            (snap, card.owner_id().to_owned())
        };

        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone: ZoneName::Battlefield,
            to_zone: ZoneName::Hand,
            controller_id: PlayerId::new(&controller_id),
        };

        // Perform the zone change.
        let card = {
            let player = self.player_state_mut(&controller_id)?;
            player.battlefield.remove(card_idx)
        };

        // Clean up permanent state.
        self.permanent_states.remove(permanent_id);

        // Remove all WhileSourceOnBattlefield global effects whose source was this permanent (LS1).
        self.remove_effects_for_source(permanent_id);

        // Add to owner's hand.
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.hand.push(card);
        }

        Ok(vec![event])
    }

    /// Tap a permanent.
    pub(crate) fn tap_permanent(&mut self, permanent_id: &str) -> Result<(), GameError> {
        let state = self.permanent_states.get(permanent_id).ok_or_else(|| {
            GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            }
        })?;
        let new_state = state.with_tapped(true);
        self.permanent_states.insert(permanent_id.to_owned(), new_state);
        Ok(())
    }

    /// Untap a permanent.
    #[allow(dead_code)]
    pub(crate) fn untap_permanent(&mut self, permanent_id: &str) -> Result<(), GameError> {
        let state = self.permanent_states.get(permanent_id).ok_or_else(|| {
            GameError::PermanentNotFound {
                permanent_id: CardInstanceId::new(permanent_id),
            }
        })?;
        let new_state = state.with_tapped(false);
        self.permanent_states.insert(permanent_id.to_owned(), new_state);
        Ok(())
    }

    /// Update the `PermanentState` for a permanent.
    pub(crate) fn set_permanent_state(&mut self, permanent_id: &str, state: PermanentState) {
        self.permanent_states.insert(permanent_id.to_owned(), state);
    }

    /// Remove a permanent state entry (used when a permanent leaves the battlefield
    /// through means other than `move_permanent_to_graveyard`).
    #[allow(dead_code)]
    pub(crate) fn remove_permanent_state(&mut self, permanent_id: &str) {
        self.permanent_states.remove(permanent_id);
    }

    /// Remove all global continuous effects whose `source_id` equals `permanent_id`.
    ///
    /// Called whenever a permanent leaves the battlefield, to clean up the
    /// `WhileSourceOnBattlefield` effects it generated (CR 613.7a).
    pub(crate) fn remove_effects_for_source(&mut self, permanent_id: &str) {
        self.global_continuous_effects.retain(|e| {
            match &e.duration {
                EffectDuration::WhileSourceOnBattlefield(source) => source != permanent_id,
                EffectDuration::UntilEndOfTurn => true,
            }
        });
    }

    /// Add a global continuous effect to the game's effect list.
    ///
    /// Use this for:
    /// - Spell-resolution effects with `UntilEndOfTurn` and an auto-incremented timestamp.
    /// - Static-ability effects with `WhileSourceOnBattlefield` and the source's ETB timestamp.
    pub(crate) fn add_global_continuous_effect(
        &mut self,
        effect: crate::domain::game::layer_system::GlobalContinuousEffect,
    ) {
        self.global_continuous_effects.push(effect);
    }

    /// Allocate the next monotonically increasing effect timestamp and increment the counter.
    ///
    /// Used for spell-resolution effects (CR 613.7b).
    pub(crate) fn next_timestamp(&mut self) -> u64 {
        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        ts
    }
}

// ============================================================================
// Tests for exile zone
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::domain::game::test_helpers::{add_permanent_to_battlefield, make_game_in_first_main};

    // ---- return_permanent_to_hand (bounce) -----------------------------------

    #[test]
    fn return_permanent_to_hand_moves_to_hand() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let card = CardInstance::new("bear-1", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);
        let initial_hand_size = game.hand(&p1).expect("hand should exist").len();

        game.return_permanent_to_hand("bear-1")
            .expect("return to hand should succeed");

        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            0,
            "battlefield should be empty after bounce"
        );
        assert_eq!(
            game.hand(&p1).expect("hand should exist").len(),
            initial_hand_size + 1,
            "hand should have one more card"
        );
    }

    #[test]
    fn return_permanent_to_hand_removes_permanent_state() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let card = CardInstance::new("bear-bounce", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        assert!(game.permanent_state("bear-bounce").is_some());

        game.return_permanent_to_hand("bear-bounce")
            .expect("return to hand should succeed");

        assert!(
            game.permanent_state("bear-bounce").is_none(),
            "permanent state should be cleaned up"
        );
    }

    #[test]
    fn return_permanent_to_hand_unknown_id_returns_error() {
        let (mut game, _p1, _p2) = make_game_in_first_main();
        let result = game.return_permanent_to_hand("nonexistent-999");
        assert!(result.is_err(), "should fail for unknown permanent id");
    }

    #[test]
    fn exile_starts_empty() {
        let (game, p1, _p2) = make_game_in_first_main();
        assert_eq!(
            game.exile(&p1).expect("player should exist").len(),
            0,
            "exile zone should be empty at game start"
        );
    }

    #[test]
    fn exile_accessor_returns_exiled_cards() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let card = CardInstance::new("bear-1", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        game.move_permanent_to_exile("bear-1")
            .expect("exile should succeed");

        let exile = game.exile(&p1).expect("player should exist");
        assert_eq!(exile.len(), 1);
        assert_eq!(exile[0].instance_id(), "bear-1");
    }

    #[test]
    fn move_permanent_to_exile_removes_from_battlefield() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(2, 2);
        let card = CardInstance::new("bear-1", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        assert_eq!(game.battlefield(&p1).unwrap().len(), 1);

        game.move_permanent_to_exile("bear-1")
            .expect("exile should succeed");

        assert_eq!(
            game.battlefield(&p1).unwrap().len(),
            0,
            "battlefield should be empty after exile"
        );
        assert_eq!(
            game.exile(&p1).unwrap().len(),
            1,
            "exile should have the card"
        );
    }

    #[test]
    fn move_permanent_to_exile_unknown_id_returns_error() {
        let (mut game, _p1, _p2) = make_game_in_first_main();
        let result = game.move_permanent_to_exile("nonexistent-999");
        assert!(result.is_err(), "should fail for unknown permanent id");
    }

    #[test]
    fn enters_tapped_permanent_starts_tapped() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::{CardType, StaticAbility, ZoneName};

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("tapland", "Tapland", vec![CardType::Land])
            .with_static_ability(StaticAbility::EntersTapped);
        let card = CardInstance::new("tapland-1", def, &p1);
        game.enter_battlefield(card, &p1, ZoneName::Hand);

        let state = game.permanent_state("tapland-1").expect("should exist");
        assert!(state.is_tapped(), "EntersTapped permanent should start tapped");
    }

    #[test]
    fn normal_permanent_starts_untapped() {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::{CardType, ZoneName};

        let (mut game, p1, _p2) = make_game_in_first_main();

        let def = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        let card = CardInstance::new("forest-1", def, &p1);
        game.enter_battlefield(card, &p1, ZoneName::Hand);

        let state = game.permanent_state("forest-1").expect("should exist");
        assert!(!state.is_tapped(), "normal permanent should start untapped");
    }
}
