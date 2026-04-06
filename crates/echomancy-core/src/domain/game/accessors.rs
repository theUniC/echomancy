//! Read-only accessor methods for the `Game` aggregate.
//!
//! All methods in this file take `&self` and return read-only views of game
//! state. They are kept separate from mutation helpers to make the public API
//! surface easy to survey.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::entities::the_stack::StackItem;
use crate::domain::enums::{CardType, GameLifecycleState, ManaColor, StaticAbility, Step};
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::game::layer_system::{
    EffectiveCharacteristics, LayerContext, evaluate_layers,
};
use crate::domain::types::PlayerId;
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;
use crate::infrastructure::game_state_export::GameStateExport;
use crate::infrastructure::game_state_export::export_game_state;

use super::{Game, GameOutcome, GamePlayerState};

// ============================================================================
// Keyword counter translation
// ============================================================================

/// Map a keyword counter string (e.g. `"FLYING_COUNTER"`) to its `StaticAbility`.
///
/// Returns `None` for any counter type that is not a recognized keyword counter.
fn keyword_counter_to_ability(counter_type: &str) -> Option<StaticAbility> {
    match counter_type {
        "FLYING_COUNTER" => Some(StaticAbility::Flying),
        "REACH_COUNTER" => Some(StaticAbility::Reach),
        "VIGILANCE_COUNTER" => Some(StaticAbility::Vigilance),
        "HASTE_COUNTER" => Some(StaticAbility::Haste),
        "FLASH_COUNTER" => Some(StaticAbility::Flash),
        "FIRST_STRIKE_COUNTER" => Some(StaticAbility::FirstStrike),
        "DOUBLE_STRIKE_COUNTER" => Some(StaticAbility::DoubleStrike),
        "TRAMPLE_COUNTER" => Some(StaticAbility::Trample),
        "DEATHTOUCH_COUNTER" => Some(StaticAbility::Deathtouch),
        "LIFELINK_COUNTER" => Some(StaticAbility::Lifelink),
        "HEXPROOF_COUNTER" => Some(StaticAbility::Hexproof),
        "SHROUD_COUNTER" => Some(StaticAbility::Shroud),
        "INDESTRUCTIBLE_COUNTER" => Some(StaticAbility::Indestructible),
        "MENACE_COUNTER" => Some(StaticAbility::Menace),
        _ => None,
    }
}

/// Collect all keyword counters on a permanent state and return the corresponding
/// `StaticAbility` values. Only includes abilities for counters that are actually
/// present (count > 0).
fn keyword_counter_abilities(state: &PermanentState) -> Vec<StaticAbility> {
    // All recognizable keyword counter names — checked against the permanent's counter map.
    const KEYWORD_COUNTER_NAMES: &[&str] = &[
        "FLYING_COUNTER",
        "REACH_COUNTER",
        "VIGILANCE_COUNTER",
        "HASTE_COUNTER",
        "FLASH_COUNTER",
        "FIRST_STRIKE_COUNTER",
        "DOUBLE_STRIKE_COUNTER",
        "TRAMPLE_COUNTER",
        "DEATHTOUCH_COUNTER",
        "LIFELINK_COUNTER",
        "HEXPROOF_COUNTER",
        "SHROUD_COUNTER",
        "INDESTRUCTIBLE_COUNTER",
        "MENACE_COUNTER",
    ];

    KEYWORD_COUNTER_NAMES
        .iter()
        .filter(|&&name| state.get_counters(name) > 0)
        .filter_map(|&name| keyword_counter_to_ability(name))
        .collect()
}

impl Game {
    // =========================================================================
    // Public accessors
    // =========================================================================

    /// The game's unique ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns all unique card definition IDs across all players' zones
    /// (library, hand, battlefield, graveyard, exile).
    pub fn all_card_definition_ids(&self) -> Vec<String> {
        let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for player in &self.players {
            for card in player.library.iter()
                .chain(player.hand.iter())
                .chain(player.battlefield.iter())
                .chain(player.graveyard.iter())
                .chain(player.exile.iter())
            {
                ids.insert(card.definition().id().to_owned());
            }
        }
        ids.into_iter().collect()
    }

    /// Current lifecycle state.
    pub fn lifecycle(&self) -> GameLifecycleState {
        self.lifecycle
    }

    /// The ID of the player whose turn it currently is.
    pub fn current_player_id(&self) -> &str {
        self.turn_state.current_player_id().as_str()
    }

    /// The current step in the turn.
    pub fn current_step(&self) -> Step {
        self.turn_state.current_step()
    }

    /// The current turn number (1-indexed).
    pub fn turn_number(&self) -> u32 {
        self.turn_state.turn_number()
    }

    /// The ID of the player who currently holds priority, if any.
    pub fn priority_player_id(&self) -> Option<&str> {
        self.priority_player_id.as_ref().map(PlayerId::as_str)
    }

    /// Number of lands the active player has played this turn.
    pub fn played_lands_this_turn(&self) -> u32 {
        self.turn_state.played_lands()
    }

    /// Player IDs in turn order.
    pub fn turn_order(&self) -> Vec<&str> {
        self.players.iter().map(|p| p.player_id.as_str()).collect()
    }

    /// Returns `true` if a player with the given ID is in the game.
    pub fn has_player(&self, player_id: &str) -> bool {
        self.players.iter().any(|p| p.player_id.as_str() == player_id)
    }

    /// Life total for a player.
    ///
    /// # Errors
    ///
    /// Returns `GameError::PlayerNotFound` if the player ID is unknown.
    pub fn player_life_total(&self, player_id: &str) -> Result<i32, GameError> {
        self.player_state(player_id).map(|p| p.life_total)
    }

    /// All cards in a player's hand (read-only slice).
    pub fn hand(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.hand.as_slice())
    }

    /// All permanents on a player's battlefield (read-only slice).
    pub fn battlefield(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.battlefield.as_slice())
    }

    /// All cards in a player's graveyard (read-only slice).
    pub fn graveyard(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.graveyard.as_slice())
    }

    /// All cards in a player's exile zone (read-only slice).
    pub fn exile(&self, player_id: &str) -> Result<&[CardInstance], GameError> {
        self.player_state(player_id).map(|p| p.exile.as_slice())
    }

    /// Number of cards in a player's library.
    pub fn library_count(&self, player_id: &str) -> Result<usize, GameError> {
        self.player_state(player_id).map(|p| p.library.len())
    }

    /// The mana pool for a player.
    pub fn mana_pool(&self, player_id: &str) -> Result<&ManaPool, GameError> {
        self.player_state(player_id).map(|p| &p.mana_pool)
    }

    /// The `PermanentState` for a permanent, if it exists.
    pub fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
        self.permanent_states.get(instance_id)
    }

    /// The stack items from bottom (index 0) to top (last).
    pub fn stack(&self) -> &[StackItem] {
        &self.stack
    }

    /// All accumulated events since the game started.
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// The game outcome, if the game has finished.
    pub fn outcome(&self) -> Option<&GameOutcome> {
        self.outcome.as_ref()
    }

    /// Returns `true` if the stack has at least one item.
    pub fn stack_has_items(&self) -> bool {
        !self.stack.is_empty()
    }

    /// The opponent of a player in a 2-player game.
    ///
    /// # Errors
    ///
    /// Returns `GameError::PlayerNotFound` if `player_id` is not in the game.
    pub fn opponent_of(&self, player_id: &str) -> Result<&str, GameError> {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() != player_id)
            .map(|p| p.player_id.as_str())
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: PlayerId::new(player_id),
            })
    }

    /// Export the complete game state as a plain data structure.
    pub fn export_state(&self) -> GameStateExport {
        export_game_state(self)
    }

    // =========================================================================
    // Internal accessors (used by handlers and services)
    // =========================================================================

    /// Read-only reference to a player's state.
    pub(crate) fn player_state(&self, player_id: &str) -> Result<&GamePlayerState, GameError> {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: PlayerId::new(player_id),
            })
    }

    /// Mutable reference to a player's state.
    pub(crate) fn player_state_mut(
        &mut self,
        player_id: &str,
    ) -> Result<&mut GamePlayerState, GameError> {
        self.players
            .iter_mut()
            .find(|p| p.player_id.as_str() == player_id)
            .ok_or_else(|| GameError::PlayerNotFound {
                player_id: PlayerId::new(player_id),
            })
    }

    /// Whether all players have passed priority.
    pub(crate) fn both_players_have_passed(&self) -> bool {
        self.players_who_passed_priority.len() >= self.players.len()
    }

    /// Whether the current player holds priority.
    pub(crate) fn has_priority(&self, player_id: &str) -> bool {
        self.priority_player_id
            .as_ref()
            .map(|id| id.as_str() == player_id)
            .unwrap_or(false)
    }

    // =========================================================================
    // Layer-aware characteristic queries (LS1)
    // =========================================================================

    /// Returns the effective characteristics of a permanent after running the
    /// full layer pipeline (Layers 4–7, CR 613).
    ///
    /// Returns `None` if the permanent is not found on any battlefield.
    pub fn effective_characteristics(&self, permanent_id: &str) -> Option<EffectiveCharacteristics> {
        // Find the permanent's card definition and controller.
        let (card_def, controller_id) = {
            let mut found = None;
            for player in &self.players {
                if let Some(card) = player.battlefield.iter().find(|c| c.instance_id() == permanent_id) {
                    found = Some((card.definition().clone(), player.player_id.as_str().to_owned()));
                    break;
                }
            }
            found?
        };

        let perm_state = self.permanent_states.get(permanent_id)?;

        // Count cards in controller's hand for CardsInControllerHand CDA.
        // Invariant: controller_id was found alongside the battlefield card, so
        // the player must exist. unwrap_or(0) is a safe fallback rather than panic.
        let controller_hand_size = self
            .player_state(&controller_id)
            .map(|p| p.hand.len())
            .unwrap_or(0);

        // Build keyword counters: translate named counter strings (e.g. "FLYING_COUNTER")
        // into StaticAbility values. Any permanent can have keyword counters placed on it
        // via effects (Ikoria-style, CR 122.6). Each counter type name matches the pattern
        // "<KEYWORD>_COUNTER" which maps 1:1 to a StaticAbility variant.
        let keyword_counters: Vec<StaticAbility> = keyword_counter_abilities(perm_state);

        let ctx = LayerContext {
            permanent_id,
            controller_id: controller_id.as_str(),
            base_types: card_def.types(),
            base_subtypes: card_def.subtypes(),
            base_colors: card_def.colors(),
            base_abilities: card_def.static_abilities(),
            base_power: card_def.power().map(|p| p as i32),
            base_toughness: card_def.toughness().map(|t| t as i32),
            plus_counters: perm_state.get_counters("PLUS_ONE_PLUS_ONE"),
            minus_counters: perm_state.get_counters("MINUS_ONE_MINUS_ONE"),
            keyword_counters,
            controller_hand_size,
            effects: &self.global_continuous_effects,
        };

        Some(evaluate_layers(&ctx))
    }

    /// Returns the effective power of a permanent after running the layer pipeline.
    ///
    /// Returns `None` if the permanent is not found or is not a creature after Layer 4.
    pub fn effective_power(&self, permanent_id: &str) -> Option<i32> {
        self.effective_characteristics(permanent_id)?.power
    }

    /// Returns the effective toughness of a permanent after running the layer pipeline.
    ///
    /// Returns `None` if the permanent is not found or is not a creature after Layer 4.
    pub fn effective_toughness(&self, permanent_id: &str) -> Option<i32> {
        self.effective_characteristics(permanent_id)?.toughness
    }

    /// Returns the effective card types of a permanent after running the layer pipeline.
    ///
    /// Returns `None` if the permanent is not found.
    pub fn effective_types(&self, permanent_id: &str) -> Option<Vec<CardType>> {
        Some(self.effective_characteristics(permanent_id)?.types)
    }

    /// Returns the effective colors of a permanent after running the layer pipeline.
    ///
    /// Returns `None` if the permanent is not found.
    pub fn effective_colors(&self, permanent_id: &str) -> Option<Vec<ManaColor>> {
        Some(self.effective_characteristics(permanent_id)?.colors)
    }

    /// Returns the effective keyword abilities of a permanent after running the layer pipeline.
    ///
    /// Returns `None` if the permanent is not found.
    pub fn effective_abilities(&self, permanent_id: &str) -> Option<Vec<StaticAbility>> {
        Some(self.effective_characteristics(permanent_id)?.abilities)
    }
}
