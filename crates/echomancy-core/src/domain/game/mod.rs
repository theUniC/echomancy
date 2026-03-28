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
mod cast_spell;
mod declare_attacker;
mod declare_blocker;
mod draw_card;
mod end_turn;
mod export;
mod internals;
mod pass_priority;
mod play_land;

use std::collections::{HashMap, HashSet};

use crate::domain::actions::Action;
use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{GameLifecycleState, Step};
use crate::domain::errors::GameError;
use crate::domain::events::GameEvent;
use crate::domain::entities::the_stack::StackItem;
use crate::domain::types::PlayerId;
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
