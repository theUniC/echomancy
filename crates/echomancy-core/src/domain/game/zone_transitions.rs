//! Zone transition helpers for the `Game` aggregate.
//!
//! Methods for moving cards between zones: entering the battlefield,
//! moving permanents to the graveyard, and tapping/untapping permanents.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{GraveyardReason, ZoneName};
use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
use crate::domain::value_objects::permanent_state::PermanentState;

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

        // Initialize permanent state
        if permanent.definition().is_creature() {
            let power = permanent.definition().power().unwrap_or(0) as i32;
            let toughness = permanent.definition().toughness().unwrap_or(0) as i32;
            self.permanent_states.insert(
                permanent.instance_id().to_owned(),
                PermanentState::for_creature(power, toughness),
            );
        } else if self.is_permanent_type(&permanent) {
            self.permanent_states.insert(
                permanent.instance_id().to_owned(),
                PermanentState::for_non_creature(),
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

        // Add to owner's graveyard.
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.graveyard.push(card);
        }

        // Execute triggers now that the card is in the graveyard (so CLIPS can
        // see the updated battlefield state if needed).
        self.execute_triggered_abilities(triggered);

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
}
