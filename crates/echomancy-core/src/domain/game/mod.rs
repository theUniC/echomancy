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
mod combat_damage;
mod declare_attacker;
mod declare_blocker;
mod draw_card;
mod end_turn;
mod export;
mod internals;
pub(crate) mod layer_system;
mod mechanics;
mod pass_priority;
mod play_land;
mod priority;
pub(crate) mod replacement_effects;
mod sacrifice;
mod sba;
mod stack_resolution;
mod tokens;
mod zone_transitions;

use std::collections::{HashMap, HashSet};

use crate::domain::actions::Action;
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{GameLifecycleState, ManaColor, Step};
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::entities::the_stack::StackItem;
use crate::domain::game::layer_system::GlobalContinuousEffect;
use crate::domain::game::replacement_effects::ReplacementEffect;
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

    // =========================================================================
    // Replacement Effects Framework (R11)
    // =========================================================================

    /// Game-wide list of active replacement effects (CR 614).
    ///
    /// Replacement effects intercept game events (damage, destroy, ETB) BEFORE
    /// they occur and modify them. Each effect carries a filter (what it
    /// watches for), an outcome (what happens instead), and a duration.
    ///
    /// This registry is checked at each interception point. Effects are removed
    /// when their duration expires or they are consumed.
    pub(crate) replacement_effects: Vec<ReplacementEffect>,

    /// Monotonically increasing counter for assigning event instance IDs.
    ///
    /// Each damage/destroy/ETB event gets a unique ID so the apply-once rule
    /// (CR 614.6) can be enforced: a replacement effect records event IDs it
    /// has already been applied to.
    pub(crate) next_event_instance_id: u64,
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
            replacement_effects: Vec::new(),
            next_event_instance_id: 1,
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

    // =========================================================================
    // Replacement Effects test utilities (pub to allow cross-crate test setup)
    // =========================================================================

    /// Register a prevention shield on `target_id` (creature or player).
    ///
    /// Prevents up to `amount` damage. Decrements on each use; removed when
    /// depleted. This is a test-only helper — in a real game, shields are
    /// registered through CLIPS spell resolution.
    pub fn register_prevention_shield(&mut self, target_id: &str, amount: i32) {
        use crate::domain::game::replacement_effects::{
            ReplacementDuration, ReplacementEffect, ReplacementEventFilter, ReplacementOutcome,
        };
        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        let effect = ReplacementEffect::new(
            format!("prevention-shield-{target_id}"),
            format!("source-{target_id}"),
            "test",
            ReplacementEventFilter::DamageToPermanent {
                permanent_id: target_id.to_owned(),
            },
            ReplacementOutcome::PreventDamage { amount },
            ReplacementDuration::UntilDepleted { remaining: amount },
            ts,
        );
        self.replacement_effects.push(effect);
    }

    /// Register a regeneration shield on `creature_id`.
    ///
    /// The next time the creature would be destroyed by lethal damage or a
    /// destroy effect, it is instead tapped, cleared of damage, and removed
    /// from combat. The shield is consumed on first use (`NextOccurrence`).
    ///
    /// This is a test-only helper — in a real game, shields come from activated
    /// abilities resolved through CLIPS.
    pub fn register_regeneration_shield(&mut self, creature_id: &str, controller_id: &str) {
        use crate::domain::game::replacement_effects::{
            ReplacementDuration, ReplacementEffect, ReplacementEventFilter, ReplacementOutcome,
        };
        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        let effect = ReplacementEffect::new(
            format!("regen-shield-{creature_id}"),
            creature_id,
            controller_id,
            ReplacementEventFilter::DestroyPermanent {
                permanent_id: creature_id.to_owned(),
            },
            ReplacementOutcome::Regenerate,
            ReplacementDuration::NextOccurrence,
            ts,
        );
        self.replacement_effects.push(effect);
    }

    /// Deal `amount` damage to creature `target_id`, going through the
    /// replacement effects framework.
    ///
    /// The final (possibly reduced) damage is marked on the creature's state.
    /// Returns the final damage amount after all replacements were applied.
    ///
    /// This is a test-only helper — in a real game, damage comes from spell
    /// resolution or combat, both of which call `apply_damage_with_replacement`
    /// and `mark_damage_on_creature` internally.
    pub fn deal_damage_to_creature(&mut self, target_id: &str, amount: i32) -> i32 {
        let final_damage =
            self.apply_damage_with_replacement("test-source", target_id, amount, false, false);
        if final_damage > 0 {
            self.mark_damage_on_creature(target_id, final_damage, false);
        }
        final_damage
    }

    /// Run state-based actions.
    ///
    /// Checks for creatures with lethal damage or zero toughness and moves
    /// them to the graveyard. This is a test-only helper — in production, SBA
    /// is run automatically after every game action.
    pub fn run_sba(&mut self) {
        self.perform_state_based_actions();
    }

    /// Return the number of replacement effects currently active in the registry.
    ///
    /// Useful in tests to verify that an effect has been consumed or is still
    /// present after an event.
    pub fn replacement_effect_count(&self) -> usize {
        self.replacement_effects.len()
    }

    /// Return the remaining damage budget of the first prevention shield on
    /// `target_id`, or `None` if no prevention shield is registered for it.
    ///
    /// This is a test-only helper to verify partial shield depletion.
    pub fn prevention_shield_remaining(&self, target_id: &str) -> Option<i32> {
        use crate::domain::game::replacement_effects::{
            ReplacementDuration, ReplacementEventFilter,
        };
        self.replacement_effects.iter().find_map(|e| {
            let matches_target = matches!(
                &e.event_filter,
                ReplacementEventFilter::DamageToPermanent { permanent_id }
                    if permanent_id == target_id
            );
            if matches_target {
                if let ReplacementDuration::UntilDepleted { remaining } = &e.duration {
                    return Some(*remaining);
                }
            }
            None
        })
    }

    /// Return `true` if a regeneration shield is registered for `creature_id`.
    ///
    /// This is a test-only helper to verify that a shield has or has not been
    /// consumed.
    pub fn has_regeneration_shield(&self, creature_id: &str) -> bool {
        use crate::domain::game::replacement_effects::{
            ReplacementEventFilter, ReplacementOutcome,
        };
        self.replacement_effects.iter().any(|e| {
            matches!(
                &e.event_filter,
                ReplacementEventFilter::DestroyPermanent { permanent_id }
                    if permanent_id == creature_id
            ) && matches!(&e.replacement, ReplacementOutcome::Regenerate)
        })
    }

    /// Inject a Layer 7b "set P/T to (power, toughness)" effect on `creature_id`
    /// that lasts until end of turn.
    ///
    /// This is a test-only helper. In real gameplay, such effects come from
    /// spell resolution (e.g. "Turn to Frog" sets P/T to 1/1 until end of turn).
    pub fn inject_set_pt_effect(&mut self, creature_id: &str, power: i32, toughness: i32) {
        use crate::domain::game::layer_system::{
            EffectFilter, EffectLayer, EffectPayload, EffectTargeting, GlobalContinuousEffect,
        };
        use crate::domain::value_objects::permanent_state::EffectDuration;

        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        self.global_continuous_effects.push(GlobalContinuousEffect {
            layer: EffectLayer::Layer7b,
            payload: EffectPayload::SetPowerToughness(power, toughness),
            duration: EffectDuration::UntilEndOfTurn,
            timestamp: ts,
            source_id: "test-inject".to_owned(),
            controller_id: "test".to_owned(),
            is_cda: false,
            targeting: EffectTargeting::Filter(EffectFilter::Permanent(creature_id.to_owned())),
            locked_target_set: None,
        });
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

    // ---- Scry, Surveil, Mill, Discard, Token, Fight, Bolster, Adapt tests
    // have moved to their respective module test files:
    // mechanics.rs (scry/surveil/mill/discard/fight/bolster/adapt)
    // tokens.rs (token creation)

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
