//! Game aggregate root — the central mutable state of an Echomancy game.
//!
//! # Architecture
//!
//! The Game struct owns all zones, players, and state. It is the single
//! source of truth for a game in progress.
//!
//! Command handlers live in submodules (play_land, cast_spell, etc.) and
//! receive `&mut Game` via the `apply()` dispatcher. Each handler:
//! 1. Validates the action using specifications/services (pure reads).
//! 2. Mutates the game state in place.
//! 3. Returns a `Vec<GameEvent>` of what happened.
//!
//! Domain services (combat_declarations, state_based_actions, etc.) take
//! `&Game` and return computed results; the handlers then apply those.
//!
//! # Lifecycle
//!
//! ```text
//! Game::create() → add_player() × N → start() → apply() × N → (finished)
//! ```

mod advance_step;
mod activate_ability;
mod cast_spell;
mod declare_attacker;
mod declare_blocker;
mod draw_card;
mod end_turn;
mod pass_priority;
mod play_land;

use std::collections::{HashMap, HashSet};

use crate::domain::actions::Action;
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{CardType, GameLifecycleState, GraveyardReason, StaticAbility, Step, ZoneName};
use crate::domain::errors::GameError;
use crate::domain::events::{CardInstanceSnapshot, GameEvent};
use crate::domain::services::combat_declarations::CombatValidationContext;
use crate::domain::services::combat_resolution::{calculate_damage_assignments, CreatureCombatEntry};
use crate::domain::services::game_state_export::{
    export_game_state, ExportableGameContext, GameStateExport, StackItemExport, StackItemKind,
};
use crate::domain::services::mana_payment::pay_cost;
use crate::domain::services::state_based_actions::{
    CreatureSbaEntry, PlayerSbaEntry, find_creatures_to_destroy, find_players_who_attempted_empty_library_draw,
    find_players_with_zero_or_less_life,
};
use crate::domain::services::step_machine::advance;
use crate::domain::services::trigger_evaluation::{
    find_matching_triggers, PermanentOnBattlefield, TriggeredAbilityInfo,
};
use crate::domain::entities::the_stack::{AbilityOnStack, SpellOnStack, StackItem};
use crate::domain::types::{CardInstanceId, PlayerId};
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;
use crate::domain::value_objects::turn_state::TurnState;

// ============================================================================
// Game outcome types
// ============================================================================

/// The reason a game ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEndReason {
    LifeTotal,
    EmptyLibrary,
    SimultaneousLoss,
}

/// The winner/draw result when a game finishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameOutcome {
    Win {
        winner_id: PlayerId,
        reason: GameEndReason,
    },
    Draw {
        reason: GameEndReason,
    },
}

// ============================================================================
// Per-player mutable state
// ============================================================================

/// All zone state + mana pool for a single player, owned by the Game.
#[derive(Debug, Clone)]
pub(crate) struct GamePlayerState {
    /// The player's basic info (id, name, life total).
    pub(crate) player_id: PlayerId,
    // Reserved for export / display — will be used when state export is wired up.
    #[allow(dead_code)]
    pub(crate) name: String,
    pub(crate) life_total: i32,
    /// Cards in hand (ordered).
    pub(crate) hand: Vec<CardInstance>,
    /// Permanents on the battlefield (ordered insertion).
    pub(crate) battlefield: Vec<CardInstance>,
    /// Cards in the graveyard (ordered).
    pub(crate) graveyard: Vec<CardInstance>,
    /// Cards in library, index 0 = top.
    pub(crate) library: Vec<CardInstance>,
    /// The player's current mana pool.
    pub(crate) mana_pool: ManaPool,
}

impl GamePlayerState {
    fn new(player_id: PlayerId, name: String) -> Self {
        Self {
            player_id,
            name,
            life_total: 20,
            hand: Vec::new(),
            battlefield: Vec::new(),
            graveyard: Vec::new(),
            library: Vec::new(),
            mana_pool: ManaPool::empty(),
        }
    }
}

// ============================================================================
// Game aggregate root
// ============================================================================

/// The central aggregate root for a single game of Echomancy.
///
/// All state mutations go through `apply()` which dispatches to handler
/// submodules. All reads are available via accessor methods.
///
/// # Invariants enforced at compile time
///
/// - Only one `&mut Game` can exist at a time (Rust's borrow checker).
/// - Handler modules only mutate through `&mut Game` methods.
/// - Domain services receive `&Game` (read-only) and return results.
pub struct Game {
    /// Unique identifier for this game.
    id: String,
    /// Current lifecycle state.
    lifecycle: GameLifecycleState,
    /// Players in turn order. The index encodes the turn order.
    players: Vec<GamePlayerState>,
    /// Cached player IDs in turn order, kept in sync with `players`.
    /// Required to satisfy the `ExportableGameContext::turn_order()` lifetime.
    turn_order_ids: Vec<String>,
    /// Permanent states keyed by card instance ID string.
    permanent_states: HashMap<String, PermanentState>,
    /// The stack (LIFO).
    stack: Vec<StackItem>,
    /// Turn state value object.
    turn_state: TurnState,
    /// Which player currently holds priority (None before game starts).
    priority_player_id: Option<PlayerId>,
    /// Players who have passed priority in the current priority window.
    players_who_passed_priority: HashSet<String>,
    /// Players who will automatically pass priority (END_TURN shortcut).
    auto_pass_players: HashSet<String>,
    /// Additional steps scheduled to run (extra phases).
    scheduled_steps: Vec<Step>,
    /// The step to resume after scheduled steps are consumed.
    resume_step_after_scheduled: Option<Step>,
    /// Accumulated events from all actions (full game log).
    events: Vec<GameEvent>,
    /// Players who attempted to draw from an empty library (SBA flag).
    players_who_attempted_empty_library_draw: HashSet<String>,
    /// The outcome when the game finishes.
    outcome: Option<GameOutcome>,
}

impl Game {
    // =========================================================================
    // Constructor / lifecycle
    // =========================================================================

    /// Create a new game in the `Created` state.
    ///
    /// Players must be added with `add_player()` before calling `start()`.
    pub fn create(id: impl Into<String>) -> Self {
        Game {
            id: id.into(),
            lifecycle: GameLifecycleState::Created,
            players: Vec::new(),
            turn_order_ids: Vec::new(),
            permanent_states: HashMap::new(),
            stack: Vec::new(),
            turn_state: TurnState::initial(PlayerId::new("")),
            priority_player_id: None,
            players_who_passed_priority: HashSet::new(),
            auto_pass_players: HashSet::new(),
            scheduled_steps: Vec::new(),
            resume_step_after_scheduled: None,
            events: Vec::new(),
            players_who_attempted_empty_library_draw: HashSet::new(),
            outcome: None,
        }
    }

    /// Add a player to the game before it has started.
    ///
    /// # Errors
    ///
    /// - `GameError::CannotAddPlayerAfterStart` if the game has already started.
    /// - `GameError::DuplicatePlayer` if the player ID is already registered.
    pub fn add_player(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(), GameError> {
        if self.lifecycle != GameLifecycleState::Created {
            return Err(GameError::CannotAddPlayerAfterStart);
        }
        let player_id = PlayerId::new(id);
        if self.players.iter().any(|p| p.player_id == player_id) {
            return Err(GameError::DuplicatePlayer {
                player_id: player_id.clone(),
            });
        }
        self.turn_order_ids.push(player_id.as_str().to_owned());
        self.players.push(GamePlayerState::new(player_id, name.into()));
        Ok(())
    }

    /// Assign a deck (ordered list of cards) to a player before the game starts.
    ///
    /// Call this before `start()`. The cards are placed in library order (index 0
    /// = top of library). No shuffling is performed here; `start()` handles
    /// shuffling when `shuffle_seed` is provided.
    ///
    /// # Errors
    ///
    /// - `GameError::PlayerNotFound` if `player_id` is not in the game.
    pub fn assign_deck(
        &mut self,
        player_id: &str,
        cards: Vec<CardInstance>,
    ) -> Result<(), GameError> {
        let player = self.player_state_mut(player_id)?;
        player.library = cards;
        Ok(())
    }

    /// Start the game: shuffle libraries, deal opening hands, assign priority.
    ///
    /// Transitions lifecycle from `Created` → `Started`.
    ///
    /// # Parameters
    ///
    /// - `starting_player_id` — which player goes first.
    /// - `shuffle_seed` — optional seed for deterministic shuffling (tests).
    ///   `None` uses OS entropy.
    ///
    /// # Errors
    ///
    /// - `GameError::GameAlreadyStarted` if already started.
    /// - `GameError::InvalidPlayerCount` if fewer than 2 players.
    /// - `GameError::InvalidStartingPlayer` if `starting_player_id` not found.
    pub fn start(
        &mut self,
        starting_player_id: &str,
        shuffle_seed: Option<u64>,
    ) -> Result<(), GameError> {
        if self.lifecycle != GameLifecycleState::Created {
            return Err(GameError::GameAlreadyStarted);
        }
        if self.players.len() < 2 {
            return Err(GameError::InvalidPlayerCount {
                player_count: self.players.len(),
            });
        }
        if !self.players.iter().any(|p| p.player_id.as_str() == starting_player_id) {
            return Err(GameError::InvalidStartingPlayer {
                player_id: PlayerId::new(starting_player_id),
            });
        }

        // Shuffle libraries
        use rand::seq::SliceRandom;
        use rand::SeedableRng;
        for player in &mut self.players {
            match shuffle_seed {
                Some(seed) => {
                    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
                    player.library.shuffle(&mut rng);
                }
                None => {
                    let mut rng = rand::rngs::SmallRng::from_os_rng();
                    player.library.shuffle(&mut rng);
                }
            }
        }

        // Deal 7-card opening hands (only if libraries are non-empty)
        if self.players.iter().any(|p| !p.library.is_empty()) {
            let player_ids: Vec<String> =
                self.players.iter().map(|p| p.player_id.as_str().to_owned()).collect();
            for pid in &player_ids {
                self.draw_cards_internal(pid, 7);
            }
        }

        // Initialize turn state
        self.turn_state = TurnState::initial(PlayerId::new(starting_player_id));
        self.priority_player_id = Some(PlayerId::new(starting_player_id));
        self.lifecycle = GameLifecycleState::Started;

        Ok(())
    }

    // =========================================================================
    // Command dispatcher
    // =========================================================================

    /// Apply a game action and return the events it produced.
    ///
    /// # Errors
    ///
    /// - `GameError::GameNotStarted` if the game is still in `Created` state.
    /// - `GameError::GameFinished` if the game is already in `Finished` state.
    /// - Various `GameError` variants from the specific action handler.
    pub fn apply(&mut self, action: Action) -> Result<Vec<GameEvent>, GameError> {
        match self.lifecycle {
            GameLifecycleState::Created => return Err(GameError::GameNotStarted),
            GameLifecycleState::Finished => return Err(GameError::GameFinished),
            GameLifecycleState::Started => {}
        }

        let events = match action {
            Action::AdvanceStep { player_id } => {
                advance_step::handle(self, player_id.as_str())?
            }
            Action::EndTurn { player_id } => {
                end_turn::handle(self, player_id.as_str())?
            }
            Action::PlayLand { player_id, card_id } => {
                play_land::handle(self, player_id.as_str(), card_id.as_str())?
            }
            Action::CastSpell { player_id, card_id } => {
                cast_spell::handle(self, player_id.as_str(), card_id.as_str())?
            }
            Action::PassPriority { player_id } => {
                pass_priority::handle(self, player_id.as_str())?
            }
            Action::DeclareAttacker { player_id, creature_id } => {
                declare_attacker::handle(self, player_id.as_str(), creature_id.as_str())?
            }
            Action::DeclareBlocker { player_id, blocker_id, attacker_id } => {
                declare_blocker::handle(
                    self,
                    player_id.as_str(),
                    blocker_id.as_str(),
                    attacker_id.as_str(),
                )?
            }
            Action::ActivateAbility { player_id, permanent_id } => {
                activate_ability::handle(self, player_id.as_str(), permanent_id.as_str())?
            }
            Action::DrawCard { player_id, amount } => {
                draw_card::handle(self, player_id.as_str(), amount)?
            }
        };

        self.events.extend_from_slice(&events);
        Ok(events)
    }

    // =========================================================================
    // Public accessors
    // =========================================================================

    /// The game's unique ID.
    pub fn id(&self) -> &str {
        &self.id
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

    /// Whether a player has passed priority in the current window.
    #[allow(dead_code)]
    pub(crate) fn has_passed_priority(&self, player_id: &str) -> bool {
        self.players_who_passed_priority.contains(player_id)
    }

    /// Whether all players have passed priority.
    pub(crate) fn both_players_have_passed(&self) -> bool {
        self.players_who_passed_priority.len() >= self.players.len()
    }

    /// Whether a player is in auto-pass mode.
    #[allow(dead_code)]
    pub(crate) fn is_auto_pass(&self, player_id: &str) -> bool {
        self.auto_pass_players.contains(player_id)
    }

    /// Whether the current player holds priority.
    pub(crate) fn has_priority(&self, player_id: &str) -> bool {
        self.priority_player_id
            .as_ref()
            .map(|id| id.as_str() == player_id)
            .unwrap_or(false)
    }

    // =========================================================================
    // Internal mutations (called by handlers)
    // =========================================================================

    /// Draw `amount` cards from a player's library to their hand.
    ///
    /// If the library is empty, the "attempted draw from empty library" flag is
    /// set. State-based actions will check this flag.
    ///
    /// Returns the events produced (one `ZoneChanged` per card drawn).
    pub(crate) fn draw_cards_internal(
        &mut self,
        player_id: &str,
        amount: u32,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        for _ in 0..amount {
            // We need the library length first, then draw
            let library_empty = self
                .player_state(player_id)
                .map(|p| p.library.is_empty())
                .unwrap_or(true);

            if library_empty {
                self.players_who_attempted_empty_library_draw
                    .insert(player_id.to_owned());
                continue;
            }

            // Draw the top card
            let (card_snapshot, player_id_owned) =
                match self.player_state_mut(player_id) {
                    Ok(player) => {
                        let card = player.library.remove(0);
                        let snapshot = CardInstanceSnapshot {
                            instance_id: CardInstanceId::new(card.instance_id()),
                            definition_id: crate::domain::types::CardDefinitionId::new(
                                card.definition().id(),
                            ),
                            owner_id: PlayerId::new(card.owner_id()),
                        };
                        player.hand.push(card);
                        (snapshot, player.player_id.as_str().to_owned())
                    }
                    Err(_) => continue,
                };

            let event = GameEvent::ZoneChanged {
                card: card_snapshot,
                from_zone: ZoneName::Library,
                to_zone: ZoneName::Hand,
                controller_id: PlayerId::new(&player_id_owned),
            };
            events.push(event.clone());
            // Evaluate triggers for this draw event
            let triggered = self.collect_triggered_abilities(&event);
            self.execute_triggered_abilities(triggered);
        }
        events
    }

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
            definition_id: crate::domain::types::CardDefinitionId::new(
                permanent.definition().id(),
            ),
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
        // Find which player controls this permanent
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

        // Remove from battlefield
        let card = {
            let player = self.player_state_mut(&controller_id)?;
            player.battlefield.remove(card_idx)
        };

        // Clean up permanent state
        self.permanent_states.remove(permanent_id);

        // Add to owner's graveyard
        let owner_id = card.owner_id().to_owned();
        let snapshot = CardInstanceSnapshot {
            instance_id: CardInstanceId::new(card.instance_id()),
            definition_id: crate::domain::types::CardDefinitionId::new(card.definition().id()),
            owner_id: PlayerId::new(&owner_id),
        };
        if let Ok(owner) = self.player_state_mut(&owner_id) {
            owner.graveyard.push(card);
        }

        let event = GameEvent::ZoneChanged {
            card: snapshot,
            from_zone: ZoneName::Battlefield,
            to_zone: ZoneName::Graveyard,
            controller_id: PlayerId::new(&controller_id),
        };
        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);
        Ok(vec![event])
    }

    /// Assign priority to a player, triggering auto-pass logic if applicable.
    pub(crate) fn assign_priority_to(&mut self, player_id: &str) -> Vec<GameEvent> {
        self.priority_player_id = Some(PlayerId::new(player_id));

        // Auto-pass: if player is in auto-pass mode
        if self.auto_pass_players.contains(player_id) {
            if self.stack_has_items() {
                // Auto-pass priority when stack is non-empty
                return self.perform_internal_pass(player_id);
            } else if player_id == self.turn_state.current_player_id().as_str() {
                // Auto-advance steps when stack is empty and they're the active player
                return self.process_auto_pass();
            }
        }
        Vec::new()
    }

    /// Give priority to the opponent of the given player.
    pub(crate) fn give_priority_to_opponent_of(
        &mut self,
        player_id: &str,
    ) -> Vec<GameEvent> {
        self.players_who_passed_priority.clear();
        if let Ok(opponent_id) = self.opponent_of(player_id).map(str::to_owned) {
            self.assign_priority_to(&opponent_id)
        } else {
            Vec::new()
        }
    }

    /// Record that a player has passed priority.
    pub(crate) fn record_passed_priority(&mut self, player_id: &str) {
        self.players_who_passed_priority.insert(player_id.to_owned());
    }

    /// Resolve the top item on the stack.
    pub(crate) fn resolve_top_of_stack(&mut self) -> Vec<GameEvent> {
        if self.stack.is_empty() {
            return Vec::new();
        }
        let stack_item = match self.stack.pop() {
            Some(item) => item,
            None => return Vec::new(),
        };

        let mut events = match stack_item {
            StackItem::Spell(spell) => self.resolve_spell(spell),
            StackItem::Ability(ability) => self.resolve_ability(ability),
        };

        self.players_who_passed_priority.clear();
        let current_player = self.turn_state.current_player_id().as_str().to_owned();
        events.extend(self.assign_priority_to(&current_player));
        events
    }

    /// Add an item to the top of the stack.
    pub(crate) fn push_stack(&mut self, item: StackItem) {
        self.stack.push(item);
    }

    /// Add mana to a player's pool.
    pub fn add_mana(
        &mut self,
        player_id: &str,
        color: crate::domain::enums::ManaColor,
        amount: u32,
    ) -> Result<(), GameError> {
        if amount == 0 {
            return Err(GameError::InvalidManaAmount { amount: 0 });
        }
        let player = self.player_state_mut(player_id)?;
        player.mana_pool = player
            .mana_pool
            .add(color, amount)
            .map_err(|_| GameError::InsufficientMana {
                player_id: PlayerId::new(player_id),
                color: color.to_string(),
                requested: amount,
                available: 0,
            })?;
        Ok(())
    }

    /// Clear a player's mana pool.
    #[allow(dead_code)]
    pub(crate) fn clear_mana_pool(&mut self, player_id: &str) -> Result<(), GameError> {
        let player = self.player_state_mut(player_id)?;
        player.mana_pool = ManaPool::empty();
        Ok(())
    }

    /// Clear all players' mana pools (called at CLEANUP step).
    pub(crate) fn clear_all_mana_pools(&mut self) {
        for player in &mut self.players {
            player.mana_pool = ManaPool::empty();
        }
    }

    /// Pay the mana cost for a spell.
    ///
    /// Uses the auto-pay algorithm from `ManaPaymentService`.
    ///
    /// # Errors
    ///
    /// Returns `GameError::InsufficientManaForSpell` if the cost cannot be paid.
    pub(crate) fn pay_mana_cost_for_spell(
        &mut self,
        player_id: &str,
        card: &CardInstance,
    ) -> Result<(), GameError> {
        let mana_cost = match card.definition().mana_cost() {
            Some(cost) => cost.clone(),
            None => return Ok(()), // Free spell
        };

        let player = self.player_state_mut(player_id)?;
        let new_pool = pay_cost(player.mana_pool.clone(), &mana_cost)
            .map_err(|e| GameError::InsufficientManaForSpell {
                message: e.to_string(),
            })?;
        player.mana_pool = new_pool;
        Ok(())
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

    /// Deal damage to a player by reducing their life total.
    pub(crate) fn deal_damage_to_player(&mut self, player_id: &str, damage: i32) {
        if let Ok(player) = self.player_state_mut(player_id) {
            player.life_total -= damage;
        }
    }

    /// Mark damage on a creature (accumulates in `damage_marked_this_turn`).
    pub(crate) fn mark_damage_on_creature(&mut self, creature_id: &str, damage: i32) {
        if let Some(state) = self.permanent_states.get(creature_id) {
            if let Some(cs) = state.creature_state() {
                let new_damage = cs.damage_marked_this_turn + damage;
                if let Ok(new_state) = state.with_damage(new_damage) {
                    self.permanent_states.insert(creature_id.to_owned(), new_state);
                }
            }
        }
    }

    /// Perform state-based actions (SBA).
    ///
    /// Destroys creatures with lethal damage or zero toughness, and ends the
    /// game if a player has lost their win condition.
    pub(crate) fn perform_state_based_actions(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // 1. Destroy creatures with lethal damage or zero toughness
        let creature_entries: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .map(|(id, s)| (id.clone(), s.clone()))
            .collect();

        let sba_entries: Vec<CreatureSbaEntry<'_>> = creature_entries
            .iter()
            .map(|(id, s)| CreatureSbaEntry {
                instance_id: id.as_str(),
                state: s,
            })
            .collect();

        let to_destroy = find_creatures_to_destroy(&sba_entries);
        for creature_id in to_destroy {
            if let Ok(evts) =
                self.move_permanent_to_graveyard(&creature_id, GraveyardReason::StateBased)
            {
                events.extend(evts);
            }
        }

        // 2. Check player loss conditions
        let player_entries: Vec<(String, i32, bool)> = self
            .players
            .iter()
            .map(|p| {
                let attempted =
                    self.players_who_attempted_empty_library_draw.contains(p.player_id.as_str());
                (p.player_id.as_str().to_owned(), p.life_total, attempted)
            })
            .collect();

        let sba_player_entries: Vec<PlayerSbaEntry<'_>> = player_entries
            .iter()
            .map(|(id, life, attempted)| PlayerSbaEntry {
                player_id: id.as_str(),
                life_total: *life,
                attempted_empty_library_draw: *attempted,
            })
            .collect();

        let losers_by_life = find_players_with_zero_or_less_life(&sba_player_entries);
        let losers_by_library =
            find_players_who_attempted_empty_library_draw(&sba_player_entries);

        // Clear empty library draw flags
        for pid in &losers_by_library {
            self.players_who_attempted_empty_library_draw.remove(pid.as_str());
        }

        let all_losers: HashSet<&str> = losers_by_life
            .iter()
            .map(String::as_str)
            .chain(losers_by_library.iter().map(String::as_str))
            .collect();

        if !all_losers.is_empty() {
            let reason = if !losers_by_life.is_empty() && !losers_by_library.is_empty() {
                GameEndReason::SimultaneousLoss
            } else if !losers_by_life.is_empty() {
                GameEndReason::LifeTotal
            } else {
                GameEndReason::EmptyLibrary
            };

            if all_losers.len() >= self.players.len() {
                // All players lost simultaneously — draw
                self.outcome = Some(GameOutcome::Draw {
                    reason: GameEndReason::SimultaneousLoss,
                });
                self.lifecycle = GameLifecycleState::Finished;
            } else {
                // The remaining player wins
                let winner_id = self
                    .players
                    .iter()
                    .find(|p| !all_losers.contains(p.player_id.as_str()))
                    .map(|p| p.player_id.clone());
                if let Some(winner_id) = winner_id {
                    self.outcome = Some(GameOutcome::Win {
                        winner_id,
                        reason,
                    });
                    self.lifecycle = GameLifecycleState::Finished;
                }
            }
        }

        events
    }

    /// Advance to the next step/phase of the current turn.
    pub(crate) fn perform_step_advance(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Emit combat ended event and clear isAttacking when leaving END_OF_COMBAT
        if self.turn_state.current_step() == Step::EndOfCombat {
            let active_player = self.turn_state.current_player_id().clone();
            let event = GameEvent::CombatEnded {
                active_player_id: active_player,
            };
            events.push(event.clone());
            let triggered = self.collect_triggered_abilities(&event);
            self.execute_triggered_abilities(triggered);
            self.clear_attacking_state();
        }

        // Consume scheduled phases first
        if !self.scheduled_steps.is_empty() {
            let next_step = self.scheduled_steps.remove(0);
            events.extend(self.set_current_step(next_step));
            return events;
        }

        // Jump to resume step after scheduled steps
        if let Some(resume_step) = self.resume_step_after_scheduled.take() {
            events.extend(self.set_current_step(resume_step));
            return events;
        }

        // Normal flow
        let step_result = advance(self.turn_state.current_step());
        if step_result.should_advance_player {
            self.advance_to_next_player();
        }
        events.extend(self.set_current_step(step_result.next_step));

        if self.turn_state.is_main_phase() {
            self.players_who_passed_priority.clear();
            let current_player = self.turn_state.current_player_id().as_str().to_owned();
            events.extend(self.assign_priority_to(&current_player));
        }

        events
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn is_permanent_type(&self, card: &CardInstance) -> bool {
        let permanent_types = [
            CardType::Creature,
            CardType::Artifact,
            CardType::Enchantment,
            CardType::Planeswalker,
            CardType::Land,
        ];
        card.definition()
            .types()
            .iter()
            .any(|t| permanent_types.contains(t))
    }

    fn set_current_step(&mut self, next_step: Step) -> Vec<GameEvent> {
        self.turn_state = self.turn_state.with_step(next_step);
        self.on_enter_step(next_step)
    }

    fn on_enter_step(&mut self, step: Step) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Clear auto-pass intent and untap at the start of a new turn
        if step == Step::Untap {
            self.auto_pass_players.clear();
            self.auto_untap_for_current_player();
        }

        // Automatic draw during DRAW step (MTG 504.1 — not using stack)
        // First turn player skips their first draw (MTG 103.8a)
        if step == Step::Draw {
            let is_first_turn = self.turn_state.turn_number() == 1;
            if !is_first_turn {
                let current_player = self.turn_state.current_player_id().as_str().to_owned();
                let draw_events = self.draw_cards_internal(&current_player, 1);
                events.extend(draw_events);
                events.extend(self.perform_state_based_actions());
            }
        }

        // Combat damage resolution at COMBAT_DAMAGE step
        if step == Step::CombatDamage {
            events.extend(self.resolve_combat_damage());
            events.extend(self.perform_state_based_actions());
        }

        // Clear mana pools and damage at CLEANUP
        if step == Step::Cleanup {
            self.clear_all_mana_pools();
            self.clear_damage_on_all_creatures();
        }

        // Emit step started event and evaluate triggers
        let active_player = self.turn_state.current_player_id().clone();
        let step_event = GameEvent::StepStarted {
            step,
            active_player_id: active_player,
        };
        events.push(step_event.clone());
        let triggered = self.collect_triggered_abilities(&step_event);
        self.execute_triggered_abilities(triggered);

        events
    }

    fn auto_untap_for_current_player(&mut self) {
        let current_player = self.turn_state.current_player_id().as_str().to_owned();

        let card_instances: Vec<(String, bool)> = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() == current_player)
            .map(|p| {
                p.battlefield
                    .iter()
                    .map(|c| (c.instance_id().to_owned(), c.definition().is_creature()))
                    .collect()
            })
            .unwrap_or_default();

        for (instance_id, is_creature) in card_instances {
            if let Some(state) = self.permanent_states.get(&instance_id) {
                let new_state = if is_creature {
                    // Creatures: untap and clear summoning sickness
                    let untapped = state.with_tapped(false);
                    untapped
                        .with_summoning_sickness(false)
                        .unwrap_or_else(|_| untapped.with_tapped(false))
                } else {
                    state.with_tapped(false)
                };
                self.permanent_states.insert(instance_id, new_state);
            }
        }
    }

    fn advance_to_next_player(&mut self) {
        let current_player = self.turn_state.current_player_id().as_str().to_owned();
        let current_index = self
            .players
            .iter()
            .position(|p| p.player_id.as_str() == current_player)
            .unwrap_or_else(|| {
                debug_assert!(
                    false,
                    "Current player '{current_player}' not found in player list — invariant violated"
                );
                0
            });

        let next_index = (current_index + 1) % self.players.len();
        let next_player_id = self.players[next_index].player_id.clone();

        self.turn_state = self.turn_state.for_new_turn(next_player_id);

        // Increment turn number when wrapping around to first player
        if next_index == 0 {
            self.turn_state = self.turn_state.with_incremented_turn_number();
        }

        // Reset creature states when turn changes
        self.reset_creature_states_for_new_turn();
    }

    fn resolve_spell(&mut self, spell: SpellOnStack) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Execute effect if present
        // Note: In the Rust port, effects are handled through the Effect enum.
        // For MVP, we just resolve the placement.
        // TODO: Execute spell.card.definition().effect when Effect::resolve() is added

        // Move card to appropriate zone
        if self.is_permanent_type(&spell.card) {
            events.extend(self.enter_battlefield(spell.card.clone(), &spell.controller_id, ZoneName::Stack));
        } else {
            // Non-permanent (instant/sorcery) goes to graveyard
            let owner_id = spell.card.owner_id().to_owned();
            if let Ok(owner) = self.player_state_mut(&owner_id) {
                owner.graveyard.push(spell.card.clone());
            }
        }

        // Emit spell resolved event
        let snapshot = CardInstanceSnapshot {
            instance_id: CardInstanceId::new(spell.card.instance_id()),
            definition_id: crate::domain::types::CardDefinitionId::new(spell.card.definition().id()),
            owner_id: PlayerId::new(spell.card.owner_id()),
        };
        let event = GameEvent::SpellResolved {
            card: snapshot,
            controller_id: PlayerId::new(&spell.controller_id),
        };
        events.push(event.clone());
        let triggered = self.collect_triggered_abilities(&event);
        self.execute_triggered_abilities(triggered);

        events
    }

    fn resolve_ability(&mut self, _ability: AbilityOnStack) -> Vec<GameEvent> {
        // Find the source permanent (for Last Known Information)
        // The effect was stored when activated, so it can resolve even if the source left.
        // In MVP: effects are simple and operate on the game state directly.
        // TODO: Call ability.effect.resolve() when Effect::resolve() signature is implemented
        Vec::new()
    }

    fn perform_internal_pass(&mut self, player_id: &str) -> Vec<GameEvent> {
        self.players_who_passed_priority.insert(player_id.to_owned());

        if self.both_players_have_passed() {
            self.resolve_top_of_stack()
        } else {
            let opponent_id = self
                .players
                .iter()
                .find(|p| p.player_id.as_str() != player_id)
                .map(|p| p.player_id.as_str().to_owned())
                .unwrap_or_default();
            self.assign_priority_to(&opponent_id)
        }
    }

    fn process_auto_pass(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let max_iterations = 100;
        let mut iterations = 0;

        while iterations < max_iterations {
            iterations += 1;

            let current_player = self.turn_state.current_player_id().as_str().to_owned();

            // Stop if active player is not in auto-pass
            if !self.auto_pass_players.contains(&current_player) {
                break;
            }

            // Stop if there's something on the stack
            if self.stack_has_items() {
                break;
            }

            // Advance through cleanup to next turn
            if self.turn_state.current_step() == Step::Cleanup {
                events.extend(self.perform_step_advance());
                break; // Turn has ended
            }

            events.extend(self.perform_step_advance());
        }

        events
    }

    fn clear_attacking_state(&mut self) {
        let ids_to_update: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .filter_map(|(id, s)| {
                s.clear_combat_state()
                    .ok()
                    .map(|new_state| (id.clone(), new_state))
            })
            .collect();

        for (id, new_state) in ids_to_update {
            self.permanent_states.insert(id, new_state);
        }
    }

    fn reset_creature_states_for_new_turn(&mut self) {
        let ids_to_update: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .filter_map(|(id, s)| {
                s.clear_combat_state()
                    .ok()
                    .and_then(|cleared| cleared.with_has_attacked_this_turn(false).ok())
                    .map(|new_state| (id.clone(), new_state))
            })
            .collect();

        for (id, new_state) in ids_to_update {
            self.permanent_states.insert(id, new_state);
        }
    }

    fn resolve_combat_damage(&mut self) -> Vec<GameEvent> {
        // Collect attacker entries
        let active_player = self.turn_state.current_player_id().as_str().to_owned();

        // We need to snapshot attackers before mutating
        let attacker_entries: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| {
                s.creature_state()
                    .map(|cs| cs.is_attacking)
                    .unwrap_or(false)
            })
            .map(|(id, s)| (id.clone(), s.clone()))
            .collect();

        let combat_entries: Vec<CreatureCombatEntry<'_>> = attacker_entries
            .iter()
            .map(|(id, s)| CreatureCombatEntry {
                instance_id: id.as_str(),
                state: s,
            })
            .collect();

        let defending_player_id = self
            .players
            .iter()
            .find(|p| p.player_id.as_str() != active_player)
            .map(|p| p.player_id.as_str().to_owned())
            .unwrap_or_default();

        let assignments = calculate_damage_assignments(&combat_entries, &defending_player_id);

        // Apply all damage
        for assignment in assignments {
            if assignment.is_player {
                self.deal_damage_to_player(&assignment.target_id, assignment.amount);
            } else {
                self.mark_damage_on_creature(&assignment.target_id, assignment.amount);
            }
        }

        Vec::new()
    }

    fn clear_damage_on_all_creatures(&mut self) {
        let ids_to_update: Vec<(String, PermanentState)> = self
            .permanent_states
            .iter()
            .filter(|(_, s)| s.creature_state().is_some())
            .filter_map(|(id, s)| {
                s.clear_damage().ok().map(|new_state| (id.clone(), new_state))
            })
            .collect();

        for (id, new_state) in ids_to_update {
            self.permanent_states.insert(id, new_state);
        }
    }

    fn collect_triggered_abilities(&self, event: &GameEvent) -> Vec<TriggeredAbilityInfo> {
        let all_permanents: Vec<PermanentOnBattlefield<'_>> = self
            .players
            .iter()
            .flat_map(|p| {
                p.battlefield.iter().map(move |card| PermanentOnBattlefield {
                    permanent: card,
                    controller_id: p.player_id.as_str(),
                })
            })
            .collect();

        find_matching_triggers(&all_permanents, event)
    }

    fn execute_triggered_abilities(&mut self, abilities: Vec<TriggeredAbilityInfo>) {
        for _ability in abilities {
            // TODO: Call ability.effect.resolve(self, context) when Effect trait is implemented
            // In MVP, triggered abilities execute immediately (not placed on stack)
        }
    }
}

// ============================================================================
// ExportableGameContext implementation
// ============================================================================

impl ExportableGameContext for Game {
    fn game_id(&self) -> &str {
        &self.id
    }

    fn lifecycle_state(&self) -> GameLifecycleState {
        self.lifecycle
    }

    fn current_turn_number(&self) -> u32 {
        self.turn_state.turn_number()
    }

    fn current_player_id(&self) -> &str {
        self.turn_state.current_player_id().as_str()
    }

    fn current_step(&self) -> Step {
        self.turn_state.current_step()
    }

    fn priority_player_id(&self) -> Option<&str> {
        self.priority_player_id.as_ref().map(PlayerId::as_str)
    }

    fn turn_order(&self) -> &[String] {
        &self.turn_order_ids
    }

    fn player_life_total(&self, player_id: &str) -> i32 {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.life_total)
            .unwrap_or(0)
    }

    fn played_lands_this_turn(&self, _player_id: &str) -> u32 {
        self.turn_state.played_lands()
    }

    fn player_mana_pool(&self, player_id: &str) -> &ManaPool {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| &p.mana_pool)
            .unwrap_or_else(|| {
                // Static empty pool for missing players
                static EMPTY: std::sync::OnceLock<ManaPool> = std::sync::OnceLock::new();
                EMPTY.get_or_init(ManaPool::empty)
            })
    }

    fn hand_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.hand.as_slice())
            .unwrap_or(&[])
    }

    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.battlefield.as_slice())
            .unwrap_or(&[])
    }

    fn graveyard_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.graveyard.as_slice())
            .unwrap_or(&[])
    }

    fn library_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.library.as_slice())
            .unwrap_or(&[])
    }

    fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
        self.permanent_states.get(instance_id)
    }

    fn stack_items(&self) -> Vec<StackItemExport> {
        self.stack
            .iter()
            .map(|item| match item {
                StackItem::Spell(spell) => StackItemExport {
                    kind: StackItemKind::Spell,
                    source_card_instance_id: spell.card.instance_id().to_owned(),
                    source_card_definition_id: spell.card.definition().id().to_owned(),
                    controller_id: spell.controller_id.clone(),
                    targets: Vec::new(),
                },
                StackItem::Ability(ability) => StackItemExport {
                    kind: StackItemKind::ActivatedAbility,
                    source_card_instance_id: ability.source_id.clone(),
                    source_card_definition_id: String::new(),
                    controller_id: ability.controller_id.clone(),
                    targets: Vec::new(),
                },
            })
            .collect()
    }
}

// ============================================================================
// CombatValidationContext implementation
// ============================================================================

impl CombatValidationContext for Game {
    fn current_step(&self) -> Step {
        self.turn_state.current_step()
    }

    fn current_player_id(&self) -> &str {
        self.turn_state.current_player_id().as_str()
    }

    fn opponent_of(&self, player_id: &str) -> &str {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() != player_id)
            .map(|p| p.player_id.as_str())
            .unwrap_or("")
    }

    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.battlefield.as_slice())
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

// ============================================================================
// Test helpers
// ============================================================================

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::{CardType, StaticAbility};

    /// Create a started 2-player game with empty decks.
    ///
    /// Both players have no cards. Use `add_card_to_hand()` to add cards
    /// before testing actions.
    pub fn make_started_game() -> (Game, String, String) {
        let mut game = Game::create("test-game");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(42)).unwrap();
        (game, "p1".to_owned(), "p2".to_owned())
    }

    /// Create a started game where p1 is in FirstMain with priority.
    pub fn make_game_in_first_main() -> (Game, String, String) {
        let (mut game, p1, p2) = make_started_game();
        // Advance from Untap → Upkeep → Draw → FirstMain
        game.apply(Action::AdvanceStep {
            player_id: crate::domain::types::PlayerId::new(&p1),
        })
        .unwrap(); // → Upkeep
        game.apply(Action::AdvanceStep {
            player_id: crate::domain::types::PlayerId::new(&p1),
        })
        .unwrap(); // → Draw
        game.apply(Action::AdvanceStep {
            player_id: crate::domain::types::PlayerId::new(&p1),
        })
        .unwrap(); // → FirstMain
        (game, p1, p2)
    }

    /// Create a minimal land card instance.
    pub fn make_land_card(instance_id: &str, owner_id: &str) -> CardInstance {
        let def = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Create a minimal creature card instance with given power/toughness.
    pub fn make_creature_card(
        instance_id: &str,
        owner_id: &str,
        power: u32,
        toughness: u32,
    ) -> CardInstance {
        let def = CardDefinition::new("bear", "Bear", vec![CardType::Creature])
            .with_power_toughness(power, toughness);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Create a creature with a given static ability.
    pub fn make_creature_with_ability(
        instance_id: &str,
        owner_id: &str,
        power: u32,
        toughness: u32,
        ability: StaticAbility,
    ) -> CardInstance {
        let def = CardDefinition::new("creature", "Creature", vec![CardType::Creature])
            .with_power_toughness(power, toughness)
            .with_static_ability(ability);
        CardInstance::new(instance_id, def, owner_id)
    }

    /// Add a card directly to a player's hand (test helper).
    pub fn add_card_to_hand(game: &mut Game, player_id: &str, card: CardInstance) {
        if let Ok(player) = game.player_state_mut(player_id) {
            player.hand.push(card);
        }
    }

    /// Add a permanent directly to a player's battlefield (test helper).
    pub fn add_permanent_to_battlefield(game: &mut Game, player_id: &str, card: CardInstance) {
        // Initialize permanent state
        if card.definition().is_creature() {
            let power = card.definition().power().unwrap_or(0) as i32;
            let toughness = card.definition().toughness().unwrap_or(0) as i32;
            game.permanent_states.insert(
                card.instance_id().to_owned(),
                PermanentState::for_creature(power, toughness),
            );
        } else {
            game.permanent_states.insert(
                card.instance_id().to_owned(),
                PermanentState::for_non_creature(),
            );
        }
        if let Ok(player) = game.player_state_mut(player_id) {
            player.battlefield.push(card);
        }
    }

    /// Clear a creature's summoning sickness (test helper).
    pub fn clear_summoning_sickness(game: &mut Game, instance_id: &str) {
        if let Some(state) = game.permanent_states.get(instance_id).cloned() {
            if let Ok(new_state) = state.with_summoning_sickness(false) {
                game.permanent_states.insert(instance_id.to_owned(), new_state);
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::*;
    use crate::domain::enums::Step;

    // ---- Lifecycle: create / add_player / start ----------------------------

    #[test]
    fn create_game_is_in_created_state() {
        let game = Game::create("game-1");
        assert_eq!(game.lifecycle(), GameLifecycleState::Created);
        assert_eq!(game.id(), "game-1");
    }

    #[test]
    fn add_player_before_start_succeeds() {
        let mut game = Game::create("g");
        assert!(game.add_player("p1", "Alice").is_ok());
        assert!(game.has_player("p1"));
    }

    #[test]
    fn add_duplicate_player_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        let err = game.add_player("p1", "AliceClone").unwrap_err();
        assert!(matches!(err, GameError::DuplicatePlayer { .. }));
    }

    #[test]
    fn add_player_after_start_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(1)).unwrap();
        let err = game.add_player("p3", "Carol").unwrap_err();
        assert!(matches!(err, GameError::CannotAddPlayerAfterStart));
    }

    #[test]
    fn start_with_less_than_2_players_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        let err = game.start("p1", Some(1)).unwrap_err();
        assert!(matches!(err, GameError::InvalidPlayerCount { .. }));
    }

    #[test]
    fn start_with_invalid_starting_player_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let err = game.start("p3", Some(1)).unwrap_err();
        assert!(matches!(err, GameError::InvalidStartingPlayer { .. }));
    }

    #[test]
    fn start_transitions_to_started_state() {
        let (game, _, _) = make_started_game();
        assert_eq!(game.lifecycle(), GameLifecycleState::Started);
    }

    #[test]
    fn start_already_started_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(1)).unwrap();
        let err = game.start("p1", Some(1)).unwrap_err();
        assert!(matches!(err, GameError::GameAlreadyStarted));
    }

    #[test]
    fn apply_on_not_started_game_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let err = game
            .apply(Action::AdvanceStep {
                player_id: PlayerId::new("p1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::GameNotStarted));
    }

    #[test]
    fn turn_order_matches_player_registration_order() {
        let (game, _, _) = make_started_game();
        let order = game.turn_order();
        assert_eq!(order[0], "p1");
        assert_eq!(order[1], "p2");
    }

    #[test]
    fn starting_player_has_priority() {
        let (game, p1, _) = make_started_game();
        assert_eq!(game.priority_player_id(), Some("p1"));
        assert_eq!(game.current_player_id(), "p1");
    }

    #[test]
    fn starting_step_is_untap() {
        let (game, _, _) = make_started_game();
        assert_eq!(game.current_step(), Step::Untap);
    }

    // ---- Deck assignment + draw -------------------------------------------

    #[test]
    fn assign_deck_sets_library() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let land = make_land_card("land-1", "p1");
        game.assign_deck("p1", vec![land]).unwrap();
        // Before start, library should have 1 card
        assert_eq!(game.player_state("p1").unwrap().library.len(), 1);
    }

    #[test]
    fn start_with_decks_deals_opening_hands() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        // Assign 10-card decks
        let p1_deck: Vec<_> = (0..10)
            .map(|i| make_land_card(&format!("p1-card-{i}"), "p1"))
            .collect();
        let p2_deck: Vec<_> = (0..10)
            .map(|i| make_land_card(&format!("p2-card-{i}"), "p2"))
            .collect();
        game.assign_deck("p1", p1_deck).unwrap();
        game.assign_deck("p2", p2_deck).unwrap();
        game.start("p1", Some(42)).unwrap();

        // Both players should have 7 cards in hand
        assert_eq!(game.hand("p1").unwrap().len(), 7);
        assert_eq!(game.hand("p2").unwrap().len(), 7);
        // Both libraries should have 3 cards left
        assert_eq!(game.library_count("p1").unwrap(), 3);
        assert_eq!(game.library_count("p2").unwrap(), 3);
    }

    // ---- apply() lifecycle guards -----------------------------------------

    #[test]
    fn apply_on_finished_game_returns_error() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        game.start("p1", Some(1)).unwrap();
        // Manually finish the game
        game.lifecycle = GameLifecycleState::Finished;
        let err = game
            .apply(Action::AdvanceStep {
                player_id: PlayerId::new("p1"),
            })
            .unwrap_err();
        assert!(matches!(err, GameError::GameFinished));
    }

    // ---- Accessor tests ----------------------------------------------------

    #[test]
    fn player_life_total_defaults_to_20() {
        let (game, p1, _) = make_started_game();
        assert_eq!(game.player_life_total(&p1).unwrap(), 20);
    }

    #[test]
    fn player_not_found_error_for_unknown_id() {
        let (game, _, _) = make_started_game();
        let err = game.player_life_total("unknown").unwrap_err();
        assert!(matches!(err, GameError::PlayerNotFound { .. }));
    }

    #[test]
    fn opponent_of_returns_other_player() {
        let (game, p1, p2) = make_started_game();
        assert_eq!(game.opponent_of(&p1).unwrap(), p2);
        assert_eq!(game.opponent_of(&p2).unwrap(), p1);
    }

    #[test]
    fn stack_is_empty_initially() {
        let (game, _, _) = make_started_game();
        assert!(!game.stack_has_items());
        assert!(game.stack().is_empty());
    }

    #[test]
    fn mana_pool_is_empty_initially() {
        let (game, p1, _) = make_started_game();
        assert!(game.mana_pool(&p1).unwrap().is_empty());
    }

    // ---- State-based actions: player life loss ----------------------------

    #[test]
    fn player_with_zero_life_loses() {
        let (mut game, p1, p2) = make_started_game();
        // Manually set life total to 0
        game.player_state_mut(&p1).unwrap().life_total = 0;
        game.perform_state_based_actions();
        assert_eq!(game.lifecycle(), GameLifecycleState::Finished);
        assert!(matches!(game.outcome(), Some(GameOutcome::Win { .. })));
        if let Some(GameOutcome::Win { winner_id, .. }) = game.outcome() {
            assert_eq!(winner_id.as_str(), &p2);
        }
    }

    #[test]
    fn both_players_at_zero_life_is_draw() {
        let (mut game, p1, p2) = make_started_game();
        game.player_state_mut(&p1).unwrap().life_total = 0;
        game.player_state_mut(&p2).unwrap().life_total = 0;
        game.perform_state_based_actions();
        assert_eq!(game.lifecycle(), GameLifecycleState::Finished);
        assert!(matches!(game.outcome(), Some(GameOutcome::Draw { .. })));
    }

    // ---- State-based actions: creature death ------------------------------

    #[test]
    fn creature_with_lethal_damage_is_destroyed() {
        let (mut game, p1, _) = make_game_in_first_main();
        let creature = make_creature_card("bear-1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        // Mark lethal damage (2 = toughness)
        game.mark_damage_on_creature("bear-1", 2);
        game.perform_state_based_actions();

        // Should be in graveyard
        assert_eq!(game.graveyard(&p1).unwrap().len(), 1);
        // Should not be on battlefield
        assert_eq!(game.battlefield(&p1).unwrap().len(), 0);
        // PermanentState should be cleaned up
        assert!(game.permanent_state("bear-1").is_none());
    }

    // ---- Draw cards -------------------------------------------------------

    #[test]
    fn draw_cards_moves_from_library_to_hand() {
        let mut game = Game::create("g");
        game.add_player("p1", "Alice").unwrap();
        game.add_player("p2", "Bob").unwrap();
        let cards: Vec<_> = (0..5)
            .map(|i| make_land_card(&format!("card-{i}"), "p1"))
            .collect();
        game.assign_deck("p1", cards).unwrap();
        game.assign_deck("p2", vec![]).unwrap();
        game.start("p1", None).unwrap();

        // After start, 0 cards drawn (empty deck for p2, 5 for p1 but all drawn = < 7)
        // Start draws up to 7, but only 5 exist
        let hand_count = game.hand("p1").unwrap().len();
        assert_eq!(hand_count, 5); // Only 5 cards available
        assert_eq!(game.library_count("p1").unwrap(), 0);
    }

    #[test]
    fn draw_from_empty_library_sets_flag() {
        let (mut game, p1, _) = make_started_game();
        // Library is empty (no deck assigned)
        game.draw_cards_internal(&p1, 1);
        assert!(game
            .players_who_attempted_empty_library_draw
            .contains(&p1));
    }
}
