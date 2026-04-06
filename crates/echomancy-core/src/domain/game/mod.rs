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

mod accessors;
mod activate_ability;
mod advance_step;
pub mod automation;
pub mod bot;
mod mulligan;
mod cast_spell;
mod declare_attacker;
mod declare_blocker;
mod draw_card;
mod end_turn;
mod export;
mod internals;
pub(crate) mod layer_system;
mod pass_priority;
mod play_land;
mod priority;
mod sacrifice;
mod sba;
mod stack_resolution;
mod zone_transitions;

use std::collections::{HashMap, HashSet};

use crate::domain::actions::Action;
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{GameLifecycleState, ManaColor, Step};
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::entities::the_stack::StackItem;
use crate::domain::game::layer_system::GlobalContinuousEffect;
use crate::domain::rules_engine::RulesEngine;
use crate::domain::types::PlayerId;
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::mulligan_state::MulliganState;
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
    /// CR 104.4b — infinite SBA loop (e.g. indestructible + lethal damage cycling).
    InfiniteLoop,
    /// CR 704.5c — player accumulated 10 or more poison counters.
    PoisonCounters,
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
///
/// # Encapsulation note
///
/// NOTE: Fields are pub(crate) intentionally for the MVP. The Game aggregate
/// is the sole owner and mutation entry point — handlers in this module are
/// the only code that should mutate these fields. Making fields private and
/// adding accessor methods is a future encapsulation improvement.
#[derive(Debug, Clone)]
pub(crate) struct GamePlayerState {
    /// The player's basic info (id, life total).
    pub(crate) player_id: PlayerId,
    pub(crate) life_total: i32,
    /// Cards in hand (ordered).
    pub(crate) hand: Vec<CardInstance>,
    /// Permanents on the battlefield (ordered insertion).
    pub(crate) battlefield: Vec<CardInstance>,
    /// Cards in the graveyard (ordered).
    pub(crate) graveyard: Vec<CardInstance>,
    /// Cards in library, index 0 = top.
    pub(crate) library: Vec<CardInstance>,
    /// Cards in the exile zone.
    pub(crate) exile: Vec<CardInstance>,
    /// The player's current mana pool.
    pub(crate) mana_pool: ManaPool,
    /// Poison counters on this player (CR 704.5c: 10+ = loss).
    pub(crate) poison_counters: u32,
}

impl GamePlayerState {
    fn new(player_id: PlayerId) -> Self {
        Self {
            player_id,
            life_total: 20,
            hand: Vec::new(),
            battlefield: Vec::new(),
            graveyard: Vec::new(),
            library: Vec::new(),
            exile: Vec::new(),
            mana_pool: ManaPool::empty(),
            poison_counters: 0,
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
    /// The ID of the player who goes first (set during `start()`).
    ///
    /// Per MTG Rule 103.7a, only this player skips their draw on the very first
    /// Draw step of the game (turn 1, their turn). All other players draw
    /// normally on their first turn.
    starting_player_id: String,
    /// Optional rules engine (CLIPS) for evaluating spell and ability effects.
    ///
    /// `None` by default — the game operates without a rules engine.
    /// Set via `set_rules_engine()`. The domain depends on the `RulesEngine`
    /// abstraction only; the CLIPS implementation lives in the infrastructure layer.
    rules_engine: Option<Box<dyn RulesEngine>>,
    /// Monotonically increasing counter for generating unique token instance IDs.
    ///
    /// Each call to `create_token()` increments this counter and uses it as part
    /// of the token's instance ID (e.g. `"token-1"`, `"token-2"`).
    next_token_id: u32,
    /// Mulligan phase state.
    ///
    /// `Some(state)` means the game is in the mulligan phase (before Turn 1).
    /// `None` means mulligan is complete and normal gameplay is underway.
    pub(crate) mulligan_state: Option<MulliganState>,

    // =========================================================================
    // Layer System (LS1)
    // =========================================================================

    /// Game-wide list of active continuous effects.
    ///
    /// Effects generated by permanent static abilities are stored here with
    /// `duration: WhileSourceOnBattlefield(source_id)`. They are removed
    /// whenever the source permanent leaves the battlefield.
    ///
    /// Effects generated by spell resolution are stored here with
    /// `duration: UntilEndOfTurn` and a `timestamp` from `next_effect_timestamp`.
    pub(crate) global_continuous_effects: Vec<GlobalContinuousEffect>,

    /// Monotonically increasing counter for assigning timestamps to new effects
    /// and permanents entering the battlefield (CR 613.7).
    ///
    /// Incremented every time a permanent enters the battlefield (ETB timestamp)
    /// or a spell-resolution effect is created.
    pub(crate) next_effect_timestamp: u64,
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
            starting_player_id: String::new(),
            rules_engine: None,
            next_token_id: 0,
            mulligan_state: None,
            global_continuous_effects: Vec::new(),
            next_effect_timestamp: 1,
        }
    }

    /// Attach a rules engine to this game.
    ///
    /// The engine will be called inside `resolve_spell()` to determine spell
    /// effects. Pass `None` implicitly by not calling this (the default).
    ///
    /// # Design
    ///
    /// We store `Option<Box<dyn RulesEngine>>` rather than a concrete type so
    /// that the domain layer depends only on the abstraction. The CLIPS
    /// implementation lives entirely in the infrastructure layer.
    pub fn set_rules_engine(&mut self, engine: Box<dyn RulesEngine>) {
        self.rules_engine = Some(engine);
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
        _name: impl Into<String>,
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
        self.players.push(GamePlayerState::new(player_id));
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

        // Record the starting player (MTG 103.7a — they skip their first draw).
        self.starting_player_id = starting_player_id.to_owned();

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
            Action::CastSpell { player_id, card_id, targets, x_value } => {
                cast_spell::handle(self, player_id.as_str(), card_id.as_str(), targets, x_value)?
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
            Action::ActivateAbility { player_id, permanent_id, ability_index } => {
                activate_ability::handle(self, player_id.as_str(), permanent_id.as_str(), ability_index)?
            }
            Action::Sacrifice { player_id, permanent_id } => {
                sacrifice::handle(self, player_id.as_str(), permanent_id.as_str())?
            }
            Action::DrawCard { player_id, amount } => {
                draw_card::handle(self, player_id.as_str(), amount)?
            }
            Action::MulliganKeep { player_id } => {
                mulligan::handle_keep(self, player_id.as_str())?
            }
            Action::MulliganRedraw { player_id } => {
                mulligan::handle_redraw(self, player_id.as_str())?
            }
            Action::PutCardOnBottom { player_id, card_id } => {
                mulligan::handle_put_card_on_bottom(self, player_id.as_str(), card_id.as_str())?
            }
        };

        self.events.extend_from_slice(&events);
        Ok(events)
    }

    // =========================================================================
    // Test utilities (pub to allow cross-crate test setup)
    // =========================================================================

    /// Add mana directly to a player's pool, bypassing game rules.
    ///
    /// # Warning
    ///
    /// This method exists for test setup only. In a real game, mana is always
    /// added through `Action::ActivateAbility` on a mana source. Calling this
    /// in production code violates game rules and circumvents the rules engine.
    ///
    /// # Errors
    ///
    /// - `GameError::PlayerNotFound` if `player_id` is not in the game.
    /// - `GameError::InvalidManaAmount` if `amount` is 0.
    pub fn add_mana(
        &mut self,
        player_id: &str,
        color: ManaColor,
        amount: u32,
    ) -> Result<(), GameError> {
        self.add_mana_to_pool(player_id, color, amount)
    }

    /// Add a card directly to a player's hand, bypassing game rules.
    ///
    /// # Warning
    ///
    /// This method exists for test setup only. In a real game, cards enter
    /// the hand only through draw effects or mulligan. Calling this in
    /// production code violates game rules.
    ///
    /// # Errors
    ///
    /// - `GameError::PlayerNotFound` if `player_id` is not in the game.
    pub fn add_card_to_hand(
        &mut self,
        player_id: &str,
        card: CardInstance,
    ) -> Result<(), GameError> {
        let player = self.player_state_mut(player_id)?;
        player.hand.push(card);
        Ok(())
    }

    /// Add a permanent directly to a player's battlefield, bypassing game rules.
    ///
    /// Initialises the `PermanentState` for the card (creature stats if applicable).
    ///
    /// # Warning
    ///
    /// This method exists for test setup only. In a real game, permanents enter
    /// the battlefield only through the stack resolution. Calling this in
    /// production code violates game rules.
    ///
    /// # Errors
    ///
    /// - `GameError::PlayerNotFound` if `player_id` is not in the game.
    pub fn add_permanent_to_battlefield(
        &mut self,
        player_id: &str,
        card: CardInstance,
    ) -> Result<(), GameError> {
        let instance_id = card.instance_id().to_owned();
        let perm_state = if card.definition().is_creature() {
            let power = card.definition().power().unwrap_or(0) as i32;
            let toughness = card.definition().toughness().unwrap_or(0) as i32;
            PermanentState::for_creature(power, toughness)
        } else {
            PermanentState::for_non_creature()
        };
        self.permanent_states.insert(instance_id, perm_state);
        let player = self.player_state_mut(player_id)?;
        player.battlefield.push(card);
        Ok(())
    }

    /// Return `true` when the game is currently in the mulligan phase.
    pub fn is_in_mulligan(&self) -> bool {
        self.mulligan_state.is_some()
    }

    /// Start the game with the mulligan phase enabled.
    ///
    /// This is identical to `start()` except:
    /// 1. The game is placed in the mulligan phase (mulligan state is initialized).
    /// 2. P2 (the non-starting player) automatically keeps with 0 mulligans.
    /// 3. The game does NOT advance to Untap step until P1 completes their mulligan.
    ///
    /// Use this in the Bevy app. Use `start()` in tests that do not need the
    /// mulligan phase.
    ///
    /// # Errors
    ///
    /// Same as `start()`.
    pub fn start_with_mulligan(
        &mut self,
        starting_player_id: &str,
        shuffle_seed: Option<u64>,
    ) -> Result<(), crate::domain::errors::GameError> {
        self.start(starting_player_id, shuffle_seed)?;

        // Initialize mulligan state for all players.
        let player_ids: Vec<String> = self
            .players
            .iter()
            .map(|p| p.player_id.as_str().to_owned())
            .collect();
        let mut mulligan_state = MulliganState::new(player_ids);

        // P2 (non-starting player) always keeps immediately with 0 mulligans.
        for player in &self.players {
            let id = player.player_id.as_str();
            if id != starting_player_id {
                if let Some(status) = mulligan_state.status_mut(id) {
                    status.record_keep();
                }
            }
        }

        self.mulligan_state = Some(mulligan_state);
        Ok(())
    }

    /// Add a card to the top of a player's library, bypassing game rules.
    ///
    /// Index 0 is the top of the library (next to be drawn).
    ///
    /// # Warning
    ///
    /// This method exists for test setup only. In a real game, cards are placed
    /// into the library through shuffle effects or special rules. Calling this
    /// in production code violates game rules.
    ///
    /// # Errors
    ///
    /// - `GameError::PlayerNotFound` if `player_id` is not in the game.
    pub fn add_card_to_library_top(
        &mut self,
        player_id: &str,
        card: CardInstance,
    ) -> Result<(), GameError> {
        let player = self.player_state_mut(player_id)?;
        player.library.insert(0, card);
        Ok(())
    }

    /// Add `amount` poison counters to a player (CR 704.5c).
    ///
    /// The SBA check will end the game if the player reaches 10 or more.
    ///
    /// # Errors
    ///
    /// Returns `GameError::PlayerNotFound` if `player_id` is unknown.
    pub fn add_poison_counters(
        &mut self,
        player_id: &str,
        amount: u32,
    ) -> Result<(), GameError> {
        let player = self.player_state_mut(player_id)?;
        player.poison_counters += amount;
        Ok(())
    }

    /// Returns the number of poison counters on a player.
    ///
    /// # Errors
    ///
    /// Returns `GameError::PlayerNotFound` if `player_id` is unknown.
    pub fn player_poison_counters(&self, player_id: &str) -> Result<u32, GameError> {
        self.player_state(player_id).map(|p| p.poison_counters)
    }

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

    /// Create a token and put it directly onto the battlefield under
    /// `controller_id`'s control (CR 111.1).
    ///
    /// Tokens are `CardInstance`s with dynamically generated `CardDefinition`s.
    /// They never come from a deck or the card catalog.
    ///
    /// `types` is a list of type strings (e.g. `["Creature"]`). Unrecognised
    /// strings are silently ignored.
    ///
    /// `keywords` is a list of keyword strings (e.g. `["Flying", "Vigilance"]`).
    /// Unrecognised strings are silently ignored.
    ///
    /// # Token IDs
    ///
    /// Each token receives a unique instance ID of the form `"token-N"` where
    /// `N` is the value of `next_token_id` incremented by each call.
    pub(crate) fn create_token(
        &mut self,
        controller_id: &str,
        name: &str,
        power: i32,
        toughness: i32,
        types: &[String],
        keywords: &[String],
    ) -> Vec<crate::domain::events::GameEvent> {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;
        use crate::domain::enums::{CardType, StaticAbility, ZoneName};

        self.next_token_id += 1;
        let instance_id = format!("token-{}", self.next_token_id);
        let def_id = format!("token-def-{}", instance_id);

        let card_types: Vec<CardType> = types
            .iter()
            .filter_map(|t| parse_card_type(t.as_str()))
            .collect();

        let static_abilities: Vec<StaticAbility> = keywords
            .iter()
            .filter_map(|k| parse_static_ability(k.as_str()))
            .collect();

        let mut def = CardDefinition::new(def_id, name, card_types)
            .with_power_toughness(power.max(0) as u32, toughness.max(0) as u32);
        for ability in static_abilities {
            def = def.with_static_ability(ability);
        }

        let card = CardInstance::new(instance_id, def, controller_id);
        self.enter_battlefield(card, controller_id, ZoneName::Stack)
    }

    /// Create a Treasure token for `controller_id` (CR 111.10b).
    ///
    /// Treasure is a colorless Artifact token with subtype "Treasure".
    /// It has the ability "{T}, Sacrifice this artifact: Add one mana of any color."
    /// For MVP, the token is created on the battlefield without the full
    /// sacrifice-for-mana cost (sacrifice as cost is not yet implemented).
    pub(crate) fn create_treasure_token(
        &mut self,
        controller_id: &str,
    ) -> Vec<crate::domain::events::GameEvent> {
        self.create_token(
            controller_id,
            "Treasure",
            0,
            0,
            &["Artifact".to_owned()],
            &[],
        )
    }

    /// Create a Clue token for `controller_id` (CR 701.34 — Investigate).
    ///
    /// Clue is a colorless Artifact token with subtype "Clue".
    /// It has the ability "{2}, Sacrifice this artifact: Draw a card."
    /// For MVP, the token is created on the battlefield (sacrifice-as-cost not yet implemented).
    pub(crate) fn create_clue_token(
        &mut self,
        controller_id: &str,
    ) -> Vec<crate::domain::events::GameEvent> {
        self.create_token(
            controller_id,
            "Clue",
            0,
            0,
            &["Artifact".to_owned()],
            &[],
        )
    }

    /// Create a Food token for `controller_id` (CR 111.10c).
    ///
    /// Food is a colorless Artifact token with subtype "Food".
    /// It has the ability "{2}, {T}, Sacrifice this artifact: You gain 3 life."
    /// For MVP, the token is created on the battlefield (sacrifice-as-cost not yet implemented).
    pub(crate) fn create_food_token(
        &mut self,
        controller_id: &str,
    ) -> Vec<crate::domain::events::GameEvent> {
        self.create_token(
            controller_id,
            "Food",
            0,
            0,
            &["Artifact".to_owned()],
            &[],
        )
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
    ) -> Result<Vec<crate::domain::events::GameEvent>, GameError> {
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
    ) -> Result<Vec<crate::domain::events::GameEvent>, GameError> {
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
    ) -> Result<Vec<crate::domain::events::GameEvent>, GameError> {
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
// Private helpers for token creation
// ============================================================================

/// Parse a type string into a `CardType`, used when creating tokens from
/// `RulesAction::CreateToken` which carries type names as strings.
fn parse_card_type(s: &str) -> Option<crate::domain::enums::CardType> {
    use crate::domain::enums::CardType;
    match s.to_ascii_lowercase().as_str() {
        "creature" => Some(CardType::Creature),
        "land" => Some(CardType::Land),
        "instant" => Some(CardType::Instant),
        "sorcery" => Some(CardType::Sorcery),
        "enchantment" => Some(CardType::Enchantment),
        "artifact" => Some(CardType::Artifact),
        "planeswalker" => Some(CardType::Planeswalker),
        "kindred" => Some(CardType::Kindred),
        _ => None,
    }
}

/// Parse a keyword string into a `StaticAbility`, used when creating tokens
/// from `RulesAction::CreateToken` which carries keyword names as strings.
fn parse_static_ability(s: &str) -> Option<crate::domain::enums::StaticAbility> {
    use crate::domain::enums::StaticAbility;
    match s.to_ascii_lowercase().as_str() {
        "flying" => Some(StaticAbility::Flying),
        "reach" => Some(StaticAbility::Reach),
        "vigilance" => Some(StaticAbility::Vigilance),
        "haste" => Some(StaticAbility::Haste),
        "flash" => Some(StaticAbility::Flash),
        "first strike" | "firststrike" => Some(StaticAbility::FirstStrike),
        "double strike" | "doublestrike" => Some(StaticAbility::DoubleStrike),
        "trample" => Some(StaticAbility::Trample),
        "deathtouch" => Some(StaticAbility::Deathtouch),
        "lifelink" => Some(StaticAbility::Lifelink),
        "hexproof" => Some(StaticAbility::Hexproof),
        "shroud" => Some(StaticAbility::Shroud),
        "indestructible" => Some(StaticAbility::Indestructible),
        _ => None,
    }
}

// ============================================================================
// Test helpers
// ============================================================================

#[cfg(test)]
pub mod test_helpers {
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
        let (game, _p1, _) = make_started_game();
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
        game.mark_damage_on_creature("bear-1", 2, false);
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

    // ---- Token creation (CR 111) -------------------------------------------

    #[test]
    fn create_token_puts_creature_on_battlefield() {
        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        let keywords = vec![];
        game.create_token(&p1, "Soldier", 1, 1, &types, &keywords);

        let battlefield = game.battlefield(&p1).expect("player should exist");
        assert_eq!(battlefield.len(), 1, "token should be on the battlefield");
        assert_eq!(battlefield[0].definition().name(), "Soldier");
    }

    #[test]
    fn create_token_initialises_correct_power_toughness() {
        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);

        let battlefield = game.battlefield(&p1).unwrap();
        let token = &battlefield[0];
        assert_eq!(token.definition().power(), Some(1));
        assert_eq!(token.definition().toughness(), Some(1));

        let pstate = game
            .permanent_state(token.instance_id())
            .expect("permanent state should exist");
        let creature = pstate.creature_state().expect("should be a creature state");
        assert_eq!(creature.base_power(), 1);
        assert_eq!(creature.base_toughness(), 1);
    }

    #[test]
    fn create_token_with_keywords_has_correct_abilities() {
        use crate::domain::enums::StaticAbility;

        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        let keywords = vec!["Flying".to_owned(), "Vigilance".to_owned()];
        game.create_token(&p1, "Angel", 4, 4, &types, &keywords);

        let battlefield = game.battlefield(&p1).unwrap();
        let token = &battlefield[0];
        assert!(token.definition().has_static_ability(StaticAbility::Flying));
        assert!(token.definition().has_static_ability(StaticAbility::Vigilance));
        assert!(!token.definition().has_static_ability(StaticAbility::Haste));
    }

    #[test]
    fn create_token_has_correct_type() {
        use crate::domain::enums::CardType;

        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);

        let battlefield = game.battlefield(&p1).unwrap();
        let token = &battlefield[0];
        assert!(token.definition().types().contains(&CardType::Creature));
    }

    #[test]
    fn multiple_tokens_get_unique_instance_ids() {
        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);

        let battlefield = game.battlefield(&p1).unwrap();
        assert_eq!(battlefield.len(), 3, "all three tokens should be on the battlefield");

        // All instance IDs must be distinct
        let ids: std::collections::HashSet<&str> =
            battlefield.iter().map(|c| c.instance_id()).collect();
        assert_eq!(ids.len(), 3, "each token must have a unique instance ID");
    }

    #[test]
    fn token_ids_follow_sequential_pattern() {
        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);
        game.create_token(&p1, "Soldier", 1, 1, &types, &[]);

        let battlefield = game.battlefield(&p1).unwrap();
        let mut ids: Vec<&str> = battlefield.iter().map(|c| c.instance_id()).collect();
        ids.sort();
        assert_eq!(ids[0], "token-1");
        assert_eq!(ids[1], "token-2");
    }

    #[test]
    fn create_token_unknown_type_is_ignored() {
        let (mut game, p1, _p2) = make_started_game();

        // "Goblin" is a subtype, not a type — should be filtered out
        let types = vec!["Creature".to_owned(), "NotAType".to_owned()];
        game.create_token(&p1, "Token", 1, 1, &types, &[]);

        let battlefield = game.battlefield(&p1).unwrap();
        assert_eq!(battlefield.len(), 1);
        // Only "Creature" survived
        assert_eq!(battlefield[0].definition().types().len(), 1);
    }

    #[test]
    fn create_token_unknown_keyword_is_ignored() {
        let (mut game, p1, _p2) = make_started_game();

        let types = vec!["Creature".to_owned()];
        let keywords = vec!["Flying".to_owned(), "NotAKeyword".to_owned()];
        game.create_token(&p1, "Token", 1, 1, &types, &keywords);

        let battlefield = game.battlefield(&p1).unwrap();
        use crate::domain::enums::StaticAbility;
        assert!(battlefield[0].definition().has_static_ability(StaticAbility::Flying));
        // Only 1 recognized ability (Flying); "NotAKeyword" was silently dropped
        assert_eq!(battlefield[0].definition().static_abilities().len(), 1);
    }

    // ---- Treasure token (P10.17) -------------------------------------------

    #[test]
    fn create_treasure_token_puts_artifact_on_battlefield() {
        let (mut game, p1, _p2) = make_started_game();
        game.create_treasure_token(&p1);
        let battlefield = game.battlefield(&p1).unwrap();
        assert_eq!(battlefield.len(), 1, "treasure token should be on battlefield");
        let token = &battlefield[0];
        assert!(token.definition().is_artifact(), "treasure token should be an artifact");
        assert_eq!(token.definition().name(), "Treasure");
    }

    #[test]
    fn create_treasure_token_has_treasure_name() {
        let (mut game, p1, _p2) = make_started_game();
        game.create_treasure_token(&p1);
        let battlefield = game.battlefield(&p1).unwrap();
        assert_eq!(battlefield[0].definition().name(), "Treasure");
    }

    // ---- Clue token (P10.15) -----------------------------------------------

    #[test]
    fn create_clue_token_puts_artifact_on_battlefield() {
        let (mut game, p1, _p2) = make_started_game();
        game.create_clue_token(&p1);
        let battlefield = game.battlefield(&p1).unwrap();
        assert_eq!(battlefield.len(), 1, "clue token should be on battlefield");
        let token = &battlefield[0];
        assert!(token.definition().is_artifact(), "clue token should be an artifact");
        assert_eq!(token.definition().name(), "Clue");
    }

    // ---- Food token (P10.16) -----------------------------------------------

    #[test]
    fn create_food_token_puts_artifact_on_battlefield() {
        let (mut game, p1, _p2) = make_started_game();
        game.create_food_token(&p1);
        let battlefield = game.battlefield(&p1).unwrap();
        assert_eq!(battlefield.len(), 1, "food token should be on battlefield");
        let token = &battlefield[0];
        assert!(token.definition().is_artifact(), "food token should be an artifact");
        assert_eq!(token.definition().name(), "Food");
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

    // ============================================================================
    // Layer System Integration Tests (LS1)
    // ============================================================================

    #[test]
    fn effective_power_no_effects_returns_base_power() {
        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        assert_eq!(game.effective_power("c1"), Some(2));
        assert_eq!(game.effective_toughness("c1"), Some(2));
    }

    #[test]
    fn effective_power_returns_none_for_nonexistent_permanent() {
        let (game, _p1, _p2) = make_started_game();
        assert_eq!(game.effective_power("no-such-permanent"), None);
    }

    #[test]
    fn effective_power_with_layer7c_global_effect() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Add a +3/+3 Layer 7c global effect
        let effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(3, 3),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "giant-growth-1".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: crate::domain::game::layer_system::EffectTargeting::LockedSet(
                vec!["c1".to_owned()]
            ),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(effect);

        assert_eq!(game.effective_power("c1"), Some(5));
        assert_eq!(game.effective_toughness("c1"), Some(5));
    }

    #[test]
    fn effective_power_layer7b_set_then_layer7c_pump_correct_order() {
        // Critical: 7b sets 1/1, 7c adds +3/+3 → must be 4/4, not 2+3=5
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Layer 7b: set to 1/1 (Humility-like)
        let set_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(1, 1),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 200, // later timestamp
            source_id: "humility".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };

        // Layer 7c: +3/+3 (Giant Growth)
        let pump_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(3, 3),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100, // earlier timestamp
            source_id: "giant-growth".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };

        game.add_global_continuous_effect(set_effect);
        game.add_global_continuous_effect(pump_effect);

        // 7b sets 1/1, then 7c adds +3/+3 → 4/4
        assert_eq!(game.effective_power("c1"), Some(4));
        assert_eq!(game.effective_toughness("c1"), Some(4));
    }

    #[test]
    fn effective_types_land_returns_land_type() {
        let (mut game, p1, _p2) = make_started_game();
        let land = make_land_card("land1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);

        let types = game.effective_types("land1").expect("should get types");
        assert!(types.contains(&crate::domain::enums::CardType::Land));
        assert!(!types.contains(&crate::domain::enums::CardType::Creature));
    }

    #[test]
    fn effective_types_with_add_creature_type_effect() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        let land = make_land_card("land1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);

        // Layer 4: animate the land (add Creature type)
        let animate_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer4Type,
            payload: EffectPayload::AddTypes(vec![crate::domain::enums::CardType::Creature]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "animate-land".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["land1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(animate_effect);

        let types = game.effective_types("land1").expect("should get types");
        assert!(types.contains(&crate::domain::enums::CardType::Land));
        assert!(types.contains(&crate::domain::enums::CardType::Creature));
    }

    #[test]
    fn effective_colors_colorless_artifact_no_effects_returns_empty() {
        use crate::domain::enums::{CardType, ManaColor};
        let (mut game, p1, _p2) = make_started_game();
        let def = crate::domain::cards::card_definition::CardDefinition::new(
            "artifact", "Artifact", vec![CardType::Artifact]
        );
        let card = crate::domain::cards::card_instance::CardInstance::new("art1", def, &p1);
        add_permanent_to_battlefield(&mut game, &p1, card);

        let colors = game.effective_colors("art1").expect("should get colors");
        assert!(colors.is_empty(), "colorless artifact should have no colors");

        // Add a "becomes blue" Layer 5 effect
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let blue_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer5Color,
            payload: EffectPayload::SetColors(vec![ManaColor::Blue]),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "make-blue".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["art1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(blue_effect);

        let colors = game.effective_colors("art1").expect("should get colors");
        assert_eq!(colors, vec![ManaColor::Blue], "should be blue after Layer 5 effect");
    }

    #[test]
    fn effective_abilities_creature_with_flying_on_definition() {
        use crate::domain::enums::StaticAbility;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_with_ability("c1", &p1, 2, 2, StaticAbility::Flying);
        add_permanent_to_battlefield(&mut game, &p1, c);

        let abilities = game.effective_abilities("c1").expect("should get abilities");
        assert!(abilities.contains(&StaticAbility::Flying));
    }

    #[test]
    fn effective_abilities_grants_trample_via_layer6_effect() {
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Grant Trample via Layer 6 effect
        let grant_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer6Ability,
            payload: EffectPayload::GrantAbility(StaticAbility::Trample),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "trample-spell".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(grant_effect);

        let abilities = game.effective_abilities("c1").expect("should get abilities");
        assert!(abilities.contains(&StaticAbility::Trample));
    }

    #[test]
    fn until_end_of_turn_global_effect_expires_at_cleanup() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_game_in_first_main();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Add +3/+3 until end of turn
        let effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(3, 3),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "giant-growth".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(effect);
        assert_eq!(game.effective_power("c1"), Some(5));

        // Advance all the way to end of turn (through combat, main2, end, cleanup)
        // Use EndTurn action
        game.apply(Action::EndTurn {
            player_id: crate::domain::types::PlayerId::new(&p1),
        }).unwrap();

        // After cleanup + start of next turn, the effect should be gone
        // The game is now in p2's turn (or p1's next turn if auto-advanced)
        // At minimum the global effect list should be cleared of UntilEndOfTurn effects
        assert!(game.global_continuous_effects.is_empty(),
            "UntilEndOfTurn effects should be cleared after cleanup");
        // Effective power should return to base
        assert_eq!(game.effective_power("c1"), Some(2));
    }

    #[test]
    fn while_source_on_battlefield_effect_removed_when_source_leaves() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;
        use crate::domain::enums::GraveyardReason;

        let (mut game, p1, _p2) = make_started_game();
        let anthem = make_creature_card("anthem-src", &p1, 2, 2);
        let creature = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, anthem);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        // Add a WhileSourceOnBattlefield +1/+1 effect from "anthem-src"
        let effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(1, 1),
            duration: EffectDuration::WhileSourceOnBattlefield("anthem-src".to_owned()),
            timestamp: 50,
            source_id: "anthem-src".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(effect);
        assert_eq!(game.effective_power("c1"), Some(3));

        // Remove the source permanent (the anthem)
        game.move_permanent_to_graveyard("anthem-src", GraveyardReason::StateBased)
            .expect("should move to graveyard");

        // The effect should be gone
        assert!(game.global_continuous_effects.is_empty(),
            "WhileSourceOnBattlefield effect should be removed when source leaves");
        assert_eq!(game.effective_power("c1"), Some(2));
    }

    #[test]
    fn sba_uses_effective_toughness_from_layer7b_effect() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        // 2/2 creature
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Layer 7b sets it to 1/1
        let set_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(1, 1),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "humility".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(set_effect);

        // Apply 2 damage to the 1/1 (would not be lethal on a 2/2)
        {
            let state = game.permanent_state("c1").unwrap().clone();
            let new_state = state.with_damage(2).unwrap();
            game.permanent_states.insert("c1".to_owned(), new_state);
        }

        // SBA: effective toughness is 1, damage is 2 → lethal → creature should die
        game.perform_state_based_actions();

        assert!(
            game.permanent_states.get("c1").is_none(),
            "creature with 2 damage and effective toughness 1 should be destroyed by SBA"
        );
    }

    #[test]
    fn sba_layer7b_zero_toughness_destroys_creature() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Layer 7b reduces to 0/0
        let zero_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(0, 0),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "reducer".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(zero_effect);

        game.perform_state_based_actions();

        assert!(
            game.permanent_states.get("c1").is_none(),
            "creature with 0 effective toughness should be destroyed by SBA"
        );
    }

    #[test]
    fn etb_timestamp_assigned_when_permanent_enters_via_game_enter_battlefield() {
        let (mut game, p1, _p2) = make_started_game();

        // Initial timestamp counter
        let initial_ts = game.next_effect_timestamp;

        let c1 = make_creature_card("c1", &p1, 2, 2);
        let c2 = make_creature_card("c2", &p1, 3, 3);

        // Enter via the proper enter_battlefield method (using zone transition)
        use crate::domain::enums::ZoneName;
        game.enter_battlefield(c1, &p1, ZoneName::Hand);
        game.enter_battlefield(c2, &p1, ZoneName::Hand);

        let ts1 = game.permanent_state("c1").map(|s| s.etb_timestamp());
        let ts2 = game.permanent_state("c2").map(|s| s.etb_timestamp());

        assert_eq!(ts1, Some(initial_ts), "c1 should get initial timestamp");
        assert_eq!(ts2, Some(initial_ts + 1), "c2 should get next timestamp");
        assert!(game.next_effect_timestamp > initial_ts + 1, "counter should have advanced");
    }

    // ---- Issue 1: keyword counter integration --------------------------------

    #[test]
    fn effective_abilities_includes_keyword_counter_on_permanent() {
        use crate::domain::enums::StaticAbility;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Place a FLYING_COUNTER on the permanent via the permanent_state counter map.
        let updated_state = game
            .permanent_states
            .get("c1")
            .expect("state should exist")
            .add_counters("FLYING_COUNTER", 1);
        game.permanent_states.insert("c1".to_owned(), updated_state);

        let abilities = game.effective_abilities("c1").expect("should get abilities");
        assert!(
            abilities.contains(&StaticAbility::Flying),
            "FLYING_COUNTER on permanent must translate to Flying in effective abilities"
        );
    }

    #[test]
    fn effective_abilities_includes_trample_keyword_counter() {
        use crate::domain::enums::StaticAbility;

        let (mut game, p1, _p2) = make_started_game();
        let c = make_creature_card("c1", &p1, 3, 3);
        add_permanent_to_battlefield(&mut game, &p1, c);

        let updated_state = game
            .permanent_states
            .get("c1")
            .expect("state should exist")
            .add_counters("TRAMPLE_COUNTER", 1);
        game.permanent_states.insert("c1".to_owned(), updated_state);

        let abilities = game.effective_abilities("c1").expect("should get abilities");
        assert!(
            abilities.contains(&StaticAbility::Trample),
            "TRAMPLE_COUNTER on permanent must translate to Trample in effective abilities"
        );
    }

    // ---- Layer-aware export (LS1 integration) ---------------------------------

    #[test]
    fn game_snapshot_export_reflects_layer7b_set_power_toughness() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;
        use crate::infrastructure::game_state_export::export_game_state;

        let (mut game, p1, _p2) = make_started_game();
        // Base 4/4 creature
        let c = make_creature_card("c1", &p1, 4, 4);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Layer 7b: set P/T to 1/1 (e.g. Humility effect)
        let set_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(1, 1),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "humility".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(set_effect);

        // Export the state and check the creature's exported P/T
        let export = export_game_state(&game);
        let player_export = export.players.get(&p1).expect("player should exist");
        let card_export = player_export
            .zones
            .battlefield
            .cards
            .iter()
            .find(|c| c.instance_id == "c1")
            .expect("c1 should be on battlefield");

        let creature_state = card_export
            .creature_state
            .as_ref()
            .expect("c1 should have creature state");

        assert_eq!(
            creature_state.power, 1,
            "exported power should reflect Layer 7b SetPowerToughness effect, not base 4"
        );
        assert_eq!(
            creature_state.toughness, 1,
            "exported toughness should reflect Layer 7b SetPowerToughness effect, not base 4"
        );
    }

    #[test]
    fn game_snapshot_export_reflects_layer7c_modify_power_toughness() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;
        use crate::infrastructure::game_state_export::export_game_state;

        let (mut game, p1, _p2) = make_started_game();
        // Base 2/2 creature
        let c = make_creature_card("c1", &p1, 2, 2);
        add_permanent_to_battlefield(&mut game, &p1, c);

        // Layer 7c: +3/+3 pump (e.g. Giant Growth)
        let pump_effect = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(3, 3),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "giant-growth".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["c1".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(pump_effect);

        let export = export_game_state(&game);
        let player_export = export.players.get(&p1).expect("player should exist");
        let card_export = player_export
            .zones
            .battlefield
            .cards
            .iter()
            .find(|c| c.instance_id == "c1")
            .expect("c1 should be on battlefield");

        let creature_state = card_export
            .creature_state
            .as_ref()
            .expect("c1 should have creature state");

        assert_eq!(
            creature_state.power, 5,
            "exported power should be 2 (base) + 3 (Layer 7c pump)"
        );
        assert_eq!(
            creature_state.toughness, 5,
            "exported toughness should be 2 (base) + 3 (Layer 7c pump)"
        );
    }

    // ---- Layer-aware combat resolution (LS1 integration) ---------------------

    /// Set up a game at DeclareAttackers step with a creature for p1.
    /// Returns (game, p1, p2) with the game at DeclareAttackers, attacker placed
    /// and summoning sickness cleared.
    fn setup_at_declare_attackers_with_creature(
        power: u32,
        toughness: u32,
        creature_id: &str,
    ) -> (Game, String, String) {
        use crate::domain::types::PlayerId;

        let (mut game, p1, p2) = make_started_game();

        // Advance Untap → Upkeep → Draw → FirstMain → BeginCombat → DeclareAttackers
        // (5 AdvanceStep calls from Untap)
        for _ in 0..5 {
            let current = game.current_player_id().to_owned();
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(&current),
            })
            .unwrap();
        }

        let creature = make_creature_card(creature_id, &p1, power, toughness);
        add_permanent_to_battlefield(&mut game, &p1, creature);
        clear_summoning_sickness(&mut game, creature_id);

        (game, p1, p2)
    }

    #[test]
    fn combat_damage_uses_layer7b_effective_power() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::types::{CardInstanceId, PlayerId};
        use crate::domain::value_objects::permanent_state::EffectDuration;

        // p1 has a 2/2 attacker boosted to 5/5 by Layer 7b; p2 has no blockers.
        // Unblocked combat damage to p2 should be 5, not 2.
        let (mut game, p1, p2) =
            setup_at_declare_attackers_with_creature(2, 2, "atk");

        // Layer 7b: set P/T to 5/5
        let boost = GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(5, 5),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "boost-spell".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["atk".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(boost);

        // Sanity: effective power is 5 before combat
        assert_eq!(game.effective_power("atk"), Some(5));

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("atk"),
        })
        .unwrap();

        // Advance to DeclareBlockers (p2 has no creatures, just advance)
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → DeclareBlockers

        // Advance to FirstStrikeDamage
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → FirstStrikeDamage

        // Advance to CombatDamage (where regular combat damage is dealt)
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → CombatDamage

        // p2 should have taken 5 damage (layer-effective power), not 2 (base power)
        let p2_life = game.player_life_total(&p2).unwrap();
        assert_eq!(
            p2_life, 15,
            "p2 should have taken 5 damage (layer-effective power 5/5), ending at 15 life"
        );
    }

    #[test]
    fn combat_damage_uses_layer7c_modify_power_toughness() {
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::types::{CardInstanceId, PlayerId};
        use crate::domain::value_objects::permanent_state::EffectDuration;

        // 2/2 attacker gets +3/+3 from Layer 7c; should deal 5 combat damage unblocked.
        let (mut game, p1, p2) =
            setup_at_declare_attackers_with_creature(2, 2, "atk");

        let pump = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(3, 3),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 100,
            source_id: "giant-growth".to_owned(),
            controller_id: p1.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["atk".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(pump);

        assert_eq!(game.effective_power("atk"), Some(5));

        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("atk"),
        })
        .unwrap();

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → DeclareBlockers

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → FirstStrikeDamage

        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap(); // → CombatDamage

        let p2_life = game.player_life_total(&p2).unwrap();
        assert_eq!(
            p2_life, 15,
            "p2 should have 15 life after taking 5 layer-effective damage (2 base + 3 pump)"
        );
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

    #[test]
    fn skulk_uses_effective_power_from_layer_system() {
        // Attacker has Skulk and base 2/2. A layer pump boosts blocker to 3/3 effective.
        // Even though blocker's base power matches the attacker, its effective power (3)
        // exceeds the attacker's (2) so it must be rejected.
        use crate::domain::enums::StaticAbility;
        use crate::domain::game::layer_system::{
            EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::types::{CardInstanceId, PlayerId};
        use crate::domain::value_objects::permanent_state::EffectDuration;

        // Set up at DeclareBlockers step.
        let (mut game, p1, p2) = setup_at_declare_attackers_with_creature(2, 2, "skulker");
        // Give skulker the Skulk ability via a permanent with it.
        {
            let state = game.permanent_states.get("skulker").unwrap().clone();
            game.permanent_states.insert("skulker".to_owned(), state);
        }
        // We need a card with Skulk on the definition. Re-add with the ability.
        if let Ok(p) = game.player_state_mut(&p1) {
            if let Some(pos) = p.battlefield.iter().position(|c| c.instance_id() == "skulker") {
                p.battlefield.remove(pos);
            }
        }
        game.permanent_states.remove("skulker");

        let skulker_card = make_creature_with_ability("skulker", &p1, 2, 2, StaticAbility::Skulk);
        add_permanent_to_battlefield(&mut game, &p1, skulker_card);
        clear_summoning_sickness(&mut game, "skulker");

        // Add a 2/2 blocker for p2 (base power = 2, equal to attacker → allowed without pump)
        let blocker_card = make_creature_card("blocker", &p2, 2, 2);
        add_permanent_to_battlefield(&mut game, &p2, blocker_card);

        // Declare skulker as attacker.
        game.apply(Action::DeclareAttacker {
            player_id: PlayerId::new(&p1),
            creature_id: CardInstanceId::new("skulker"),
        })
        .unwrap();

        // Advance to DeclareBlockers.
        game.apply(Action::AdvanceStep {
            player_id: PlayerId::new(&p1),
        })
        .unwrap();

        // Without pump: blocker power (2) == attacker power (2) → allowed.
        // Now pump the blocker to 3/2 via Layer 7c — effective power now exceeds skulker.
        let pump = GlobalContinuousEffect {
            layer: EffectLayer::Layer7c,
            payload: EffectPayload::ModifyPowerToughness(1, 0),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: 200,
            source_id: "pump-blocker".to_owned(),
            controller_id: p2.clone(),
            is_cda: false,
            targeting: EffectTargeting::LockedSet(vec!["blocker".to_owned()]),
            locked_target_set: None,
        };
        game.add_global_continuous_effect(pump);

        // Effective power of blocker is now 3 > attacker's 2 → must be rejected by Skulk.
        assert_eq!(game.effective_power("blocker"), Some(3));

        let result = game.apply(Action::DeclareBlocker {
            player_id: PlayerId::new(&p2),
            blocker_id: CardInstanceId::new("blocker"),
            attacker_id: CardInstanceId::new("skulker"),
        });
        assert!(
            result.is_err(),
            "Skulk: blocker with effective power (3) > attacker (2) must be rejected"
        );
    }
}
