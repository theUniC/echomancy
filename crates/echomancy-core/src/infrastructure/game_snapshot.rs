//! GameSnapshot — player-relative view of the game state.
//!
//! Creates a filtered, UI-friendly snapshot from a [`GameStateExport`]:
//! - The viewer sees their own hand, battlefield, graveyard.
//! - Opponent's hand is hidden (only the count is exposed).
//! - Battlefield, graveyard, stack: visible to all.
//! - Library contents: hidden (only the count is exposed).
//!
//! Mirrors the TypeScript `GameSnapshot.ts` and `createGameSnapshot()`.
//!
//! # Design principles
//!
//! - Lives outside the domain engine.
//! - Derived entirely from `GameStateExport`.
//! - Contains NO rules logic.
//! - Immutable and reconstructible.
//! - Player-specific: always created FOR a specific viewer.

use std::collections::HashMap;

use crate::domain::enums::{CardType, StaticAbility, Step};
use crate::domain::services::game_state_export::{
    CardInstanceExport, GameStateExport, StackItemExport, StackItemKind,
};

// ============================================================================
// Public snapshot types
// ============================================================================

/// Combat state for a card, suitable for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatStateSnapshot {
    pub is_attacking: bool,
    pub is_blocking: bool,
    /// Instance IDs of blockers (currently 0 or 1 in MVP).
    pub blocked_by: Vec<String>,
    /// Instance IDs of creatures being blocked (0 or 1 in MVP).
    pub blocking: Vec<String>,
}

/// Combat summary: number of attackers and blockers during combat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatSummary {
    pub attacker_count: usize,
    pub blocker_count: usize,
}

/// UI-friendly card snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSnapshot {
    pub instance_id: String,
    /// Resolved human-readable name (from the card registry).
    pub name: String,
    pub types: Vec<CardType>,
    /// Static keyword abilities (Flying, Reach, Vigilance, …).
    pub static_keywords: Vec<StaticAbility>,

    pub controller_id: String,
    pub owner_id: String,

    /// `None` for non-creatures.
    pub tapped: Option<bool>,
    /// Counter counts keyed by counter type name. `None` for non-creatures.
    pub counters: Option<HashMap<String, u32>>,
    /// Damage marked this turn. `None` for non-creatures.
    pub damage_marked: Option<i32>,

    /// Base power from definition (plus counter bonus). `None` for non-creatures.
    pub power: Option<i32>,
    /// Base toughness from definition (plus counter bonus). `None` for non-creatures.
    pub toughness: Option<i32>,

    /// `None` for non-creatures.
    pub combat_state: Option<CombatStateSnapshot>,
}

/// Public game state visible to all players.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicGameState {
    pub turn_number: u32,
    pub current_player_id: String,
    /// Same as `current_player_id` in the current MVP.
    pub active_player_id: String,
    pub priority_player_id: Option<String>,

    /// Derived from `current_step` (e.g. "Beginning", "Combat").
    pub current_phase: String,
    pub current_step: Step,

    /// `None` when not in combat.
    pub combat_summary: Option<CombatSummary>,
    pub stack_size: usize,
}

/// Private player state (the viewer's own zones, fully visible).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivatePlayerState {
    pub player_id: String,
    pub life_total: i32,
    /// Always 0 in the current MVP.
    pub poison_counters: u32,
    /// Mana pool: color name → amount (W, U, B, R, G, C).
    pub mana_pool: HashMap<String, u32>,

    pub hand: Vec<CardSnapshot>,
    pub battlefield: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
    /// Always empty in the current MVP.
    pub exile: Vec<CardSnapshot>,
}

/// Opponent state with hidden information applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpponentState {
    pub player_id: String,
    pub life_total: i32,
    /// Always 0 in the current MVP.
    pub poison_counters: u32,
    /// Mana pool (visible in MVP; could be hidden in future).
    pub mana_pool: Option<HashMap<String, u32>>,

    /// Number of cards in hand — the actual cards are hidden.
    pub hand_size: usize,
    pub battlefield: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
    /// Always empty in the current MVP.
    pub exile: Vec<CardSnapshot>,
}

/// A single item on the stack, with human-readable info.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackItemSnapshot {
    pub source_card_name: String,
    pub controller_id: String,
    pub kind: StackItemKind,
    pub target_descriptions: Vec<String>,
}

/// Stack snapshot ordered top-to-bottom (index 0 = top of stack).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackSnapshot {
    pub items: Vec<StackItemSnapshot>,
}

/// UI convenience flags (no rules logic encoded here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiHints {
    pub can_pass_priority: bool,
    /// Instance IDs of attacking creatures.
    pub highlighted_attackers: Vec<String>,
    /// Instance IDs of blocking creatures.
    pub highlighted_blockers: Vec<String>,
}

/// Complete player-relative game snapshot.
///
/// # Invariants
///
/// - Created FOR a specific viewer (`viewer_player_id`).
/// - Immutable after creation.
/// - Reconstructible from the same `GameStateExport` at any time.
/// - Contains no engine references.
/// - Applies visibility rules correctly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSnapshot {
    pub viewer_player_id: String,
    pub public_game_state: PublicGameState,
    pub private_player_state: PrivatePlayerState,
    pub opponent_states: Vec<OpponentState>,
    pub visible_stack: StackSnapshot,
    pub ui_hints: Option<UiHints>,
}

// ============================================================================
// Card registry
// ============================================================================

/// Resolves a `card_definition_id` to a human-readable card name.
///
/// Callers must provide an implementation (e.g. a static lookup table).
pub trait CardRegistry {
    fn card_name(&self, definition_id: &str) -> String;
}

// ============================================================================
// Primary factory function
// ============================================================================

/// Create a `GameSnapshot` for a specific viewer from a `GameStateExport`.
///
/// # Errors
///
/// Returns an error string if `viewer_player_id` is not in the exported state.
pub fn create_game_snapshot(
    state: &GameStateExport,
    viewer_player_id: &str,
    registry: &dyn CardRegistry,
) -> Result<GameSnapshot, String> {
    // Validate viewer is in the game.
    let viewer_state = state
        .players
        .get(viewer_player_id)
        .ok_or_else(|| format!("Player {viewer_player_id} not found in game state"))?;

    // ---- Viewer's zones ----
    let viewer_hand = map_zone(&viewer_state.zones.hand.cards, registry, state);
    let viewer_battlefield = map_zone(&viewer_state.zones.battlefield.cards, registry, state);
    let viewer_graveyard = map_zone(&viewer_state.zones.graveyard.cards, registry, state);

    // ---- Opponent states ----
    let mut opponent_states = Vec::new();
    for (player_id, player_state) in &state.players {
        if player_id == viewer_player_id {
            continue;
        }

        let battlefield = map_zone(&player_state.zones.battlefield.cards, registry, state);
        let graveyard = map_zone(&player_state.zones.graveyard.cards, registry, state);

        opponent_states.push(OpponentState {
            player_id: player_id.clone(),
            life_total: player_state.life_total,
            poison_counters: 0,
            mana_pool: Some(mana_pool_to_map(&player_state.mana_pool)),
            hand_size: player_state.zones.hand.cards.len(),
            battlefield,
            graveyard,
            exile: Vec::new(),
        });
    }

    // ---- Public state ----
    let current_phase = phase_from_step(state.current_step);
    let combat_summary = build_combat_summary(state);

    let public_game_state = PublicGameState {
        turn_number: state.current_turn_number,
        current_player_id: state.current_player_id.clone(),
        active_player_id: state.current_player_id.clone(),
        priority_player_id: state.priority_player_id.clone(),
        current_phase,
        current_step: state.current_step,
        combat_summary,
        stack_size: state.stack.len(),
    };

    // ---- Private player state ----
    let private_player_state = PrivatePlayerState {
        player_id: viewer_player_id.to_owned(),
        life_total: viewer_state.life_total,
        poison_counters: 0,
        mana_pool: mana_pool_to_map(&viewer_state.mana_pool),
        hand: viewer_hand,
        battlefield: viewer_battlefield,
        graveyard: viewer_graveyard,
        exile: Vec::new(),
    };

    // ---- Stack snapshot ----
    let visible_stack = build_stack_snapshot(&state.stack, registry, state);

    // ---- UI hints ----
    let ui_hints = build_ui_hints(state, viewer_player_id);

    Ok(GameSnapshot {
        viewer_player_id: viewer_player_id.to_owned(),
        public_game_state,
        private_player_state,
        opponent_states,
        visible_stack,
        ui_hints,
    })
}

// ============================================================================
// Internal helpers
// ============================================================================

fn map_zone(
    cards: &[CardInstanceExport],
    registry: &dyn CardRegistry,
    state: &GameStateExport,
) -> Vec<CardSnapshot> {
    cards
        .iter()
        .map(|c| make_card_snapshot(c, registry, state))
        .collect()
}

fn make_card_snapshot(
    card: &CardInstanceExport,
    registry: &dyn CardRegistry,
    _state: &GameStateExport,
) -> CardSnapshot {
    let name = registry.card_name(&card.card_definition_id);

    let (tapped, counters, damage_marked, power, toughness, combat_state) =
        if let Some(cs) = &card.creature_state {
            let combat = CombatStateSnapshot {
                is_attacking: cs.is_attacking,
                is_blocking: cs.blocking_creature_id.is_some(),
                blocked_by: cs.blocked_by.iter().cloned().collect(),
                blocking: cs.blocking_creature_id.iter().cloned().collect(),
            };
            (
                Some(cs.is_tapped),
                Some(cs.counters.clone()),
                Some(cs.damage_marked_this_turn),
                Some(cs.power),
                Some(cs.toughness),
                Some(combat),
            )
        } else {
            (None, None, None, None, None, None)
        };

    CardSnapshot {
        instance_id: card.instance_id.clone(),
        name,
        types: card.types.clone(),
        static_keywords: card.static_abilities.clone(),
        controller_id: card.controller_id.clone(),
        owner_id: card.owner_id.clone(),
        tapped,
        counters,
        damage_marked,
        power,
        toughness,
        combat_state,
    }
}

fn build_stack_snapshot(
    stack: &[StackItemExport],
    registry: &dyn CardRegistry,
    state: &GameStateExport,
) -> StackSnapshot {
    // Engine stores stack bottom-to-top; we want top-to-bottom (index 0 = top).
    let items = stack
        .iter()
        .rev()
        .map(|item| {
            let source_card_name = registry.card_name(&item.source_card_definition_id);
            let target_descriptions = item
                .targets
                .iter()
                .map(|target_id| resolve_target(state, target_id, registry))
                .collect();
            StackItemSnapshot {
                source_card_name,
                controller_id: item.controller_id.clone(),
                kind: item.kind.clone(),
                target_descriptions,
            }
        })
        .collect();

    StackSnapshot { items }
}

/// Resolve a target ID to a human-readable description.
fn resolve_target(state: &GameStateExport, target_id: &str, registry: &dyn CardRegistry) -> String {
    if let Some(card) = find_card_in_state(state, target_id) {
        return registry.card_name(&card.card_definition_id);
    }
    if state.players.contains_key(target_id) {
        return format!("Player {target_id}");
    }
    "Unknown target".to_owned()
}

/// Search all player zones for a card instance by ID.
fn find_card_in_state<'a>(
    state: &'a GameStateExport,
    instance_id: &str,
) -> Option<&'a CardInstanceExport> {
    for player_state in state.players.values() {
        for card in player_state
            .zones
            .hand
            .cards
            .iter()
            .chain(player_state.zones.battlefield.cards.iter())
            .chain(player_state.zones.graveyard.cards.iter())
            .chain(player_state.zones.library.cards.iter())
        {
            if card.instance_id == instance_id {
                return Some(card);
            }
        }
    }
    None
}

fn build_combat_summary(state: &GameStateExport) -> Option<CombatSummary> {
    let in_combat = matches!(
        state.current_step,
        Step::BeginningOfCombat
            | Step::DeclareAttackers
            | Step::DeclareBlockers
            | Step::CombatDamage
            | Step::EndOfCombat
    );

    if !in_combat {
        return None;
    }

    let mut attacker_count = 0;
    let mut blocker_count = 0;

    for player_state in state.players.values() {
        for card in &player_state.zones.battlefield.cards {
            if let Some(cs) = &card.creature_state {
                if cs.is_attacking {
                    attacker_count += 1;
                }
                if cs.blocking_creature_id.is_some() {
                    blocker_count += 1;
                }
            }
        }
    }

    Some(CombatSummary {
        attacker_count,
        blocker_count,
    })
}

fn build_ui_hints(state: &GameStateExport, viewer_player_id: &str) -> Option<UiHints> {
    if !state.players.contains_key(viewer_player_id) {
        return None;
    }

    let can_pass_priority = state
        .priority_player_id
        .as_deref()
        .map(|id| id == viewer_player_id)
        .unwrap_or(false);

    let mut highlighted_attackers = Vec::new();
    let mut highlighted_blockers = Vec::new();

    for player_state in state.players.values() {
        for card in &player_state.zones.battlefield.cards {
            if let Some(cs) = &card.creature_state {
                if cs.is_attacking {
                    highlighted_attackers.push(card.instance_id.clone());
                }
                if cs.blocking_creature_id.is_some() {
                    highlighted_blockers.push(card.instance_id.clone());
                }
            }
        }
    }

    Some(UiHints {
        can_pass_priority,
        highlighted_attackers,
        highlighted_blockers,
    })
}

/// Derive the current phase name from the step.
fn phase_from_step(step: Step) -> String {
    match step {
        Step::Untap | Step::Upkeep | Step::Draw => "Beginning".to_owned(),
        Step::FirstMain => "Precombat Main".to_owned(),
        Step::BeginningOfCombat
        | Step::DeclareAttackers
        | Step::DeclareBlockers
        | Step::CombatDamage
        | Step::EndOfCombat => "Combat".to_owned(),
        Step::SecondMain => "Postcombat Main".to_owned(),
        Step::EndStep | Step::Cleanup => "Ending".to_owned(),
    }
}

/// Convert a `ManaPoolExport` to a `HashMap<String, u32>` (W, U, B, R, G, C).
fn mana_pool_to_map(
    pool: &crate::domain::services::game_state_export::ManaPoolExport,
) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    map.insert("W".to_owned(), pool.white);
    map.insert("U".to_owned(), pool.blue);
    map.insert("B".to_owned(), pool.black);
    map.insert("R".to_owned(), pool.red);
    map.insert("G".to_owned(), pool.green);
    map.insert("C".to_owned(), pool.colorless);
    map
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::actions::Action;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::CardType;
    use crate::domain::game::Game;
    use crate::domain::types::PlayerId;
    use uuid::Uuid;

    // ---- Test card registry ------------------------------------------------

    struct MockRegistry;

    impl CardRegistry for MockRegistry {
        fn card_name(&self, id: &str) -> String {
            match id {
                "test-spell" => "Test Spell".to_owned(),
                "test-spell-2" => "Test Spell 2".to_owned(),
                "test-creature" => "Test Creature".to_owned(),
                "test-creature-def" => "Test Creature".to_owned(),
                "test-land" => "Test Land".to_owned(),
                "flying-creature" => "Flying Creature".to_owned(),
                "forest" => "Forest".to_owned(),
                "plains" => "Plains".to_owned(),
                "grizzly-bears" => "Grizzly Bears".to_owned(),
                "elite-vanguard" => "Elite Vanguard".to_owned(),
                "giant-spider" => "Giant Spider".to_owned(),
                "serra-angel" => "Serra Angel".to_owned(),
                "llanowar-elves" => "Llanowar Elves".to_owned(),
                other => other.to_owned(),
            }
        }
    }

    // ---- Test helpers -------------------------------------------------------

    fn uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// Set up a started two-player game and return (game, p1_id, p2_id).
    fn make_started_game() -> (Game, String, String) {
        let p1 = uuid();
        let p2 = uuid();
        let mut game = Game::create(uuid());
        game.add_player(&p1, "Alice").unwrap();
        game.add_player(&p2, "Bob").unwrap();

        // Assign decks so start() can deal hands.
        let forest_def = CardDefinition::new("forest", "Forest", vec![CardType::Land]);
        let creature_def = CardDefinition::new("test-creature-def", "Test Creature", vec![CardType::Creature])
            .with_power_toughness(2, 2);

        for player_id in [&p1, &p2] {
            let mut deck = Vec::new();
            for _ in 0..4 {
                deck.push(CardInstance::new(uuid(), forest_def.clone(), player_id.as_str()));
            }
            for _ in 0..56 {
                deck.push(CardInstance::new(uuid(), creature_def.clone(), player_id.as_str()));
            }
            game.assign_deck(player_id, deck).unwrap();
        }

        game.start(&p1, Some(42)).unwrap();
        (game, p1, p2)
    }

    /// Advance to FIRST_MAIN step.
    fn advance_to_first_main(game: &mut Game, player_id: &str) {
        // UNTAP → UPKEEP → DRAW → FIRST_MAIN (3 advances)
        for _ in 0..3 {
            game.apply(Action::AdvanceStep {
                player_id: PlayerId::new(player_id),
            })
            .unwrap();
        }
    }

    // ---- create_game_snapshot: basic ---------------------------------------

    #[test]
    fn snapshot_sets_viewer_player_id() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.viewer_player_id, p1);
    }

    #[test]
    fn snapshot_errors_for_unknown_viewer() {
        let (game, _, _) = make_started_game();
        let export = game.export_state();
        let result = create_game_snapshot(&export, "unknown-player", &MockRegistry);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown-player"));
    }

    #[test]
    fn snapshot_is_reconstructible_from_same_export() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap1 = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        let snap2 = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap1, snap2);
    }

    // ---- Public game state -------------------------------------------------

    #[test]
    fn public_state_turn_number_is_one() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.public_game_state.turn_number, 1);
    }

    #[test]
    fn public_state_current_player_is_starting_player() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.public_game_state.current_player_id, p1);
    }

    #[test]
    fn public_state_step_is_untap_at_start() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.public_game_state.current_step, Step::Untap);
        assert_eq!(snap.public_game_state.current_phase, "Beginning");
    }

    #[test]
    fn public_state_phase_is_precombat_main_in_first_main() {
        let (mut game, p1, _) = make_started_game();
        advance_to_first_main(&mut game, &p1);
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.public_game_state.current_phase, "Precombat Main");
        assert_eq!(snap.public_game_state.current_step, Step::FirstMain);
    }

    #[test]
    fn public_state_stack_size_is_zero_initially() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.public_game_state.stack_size, 0);
    }

    #[test]
    fn public_state_combat_summary_is_none_in_main_phase() {
        let (mut game, p1, _) = make_started_game();
        advance_to_first_main(&mut game, &p1);
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert!(snap.public_game_state.combat_summary.is_none());
    }

    #[test]
    fn both_players_see_same_public_state() {
        let (game, p1, p2) = make_started_game();
        let export = game.export_state();
        let snap1 = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        let snap2 = create_game_snapshot(&export, &p2, &MockRegistry).unwrap();
        assert_eq!(snap1.public_game_state, snap2.public_game_state);
    }

    // ---- Private player state ----------------------------------------------

    #[test]
    fn private_state_has_correct_player_id() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.private_player_state.player_id, p1);
    }

    #[test]
    fn private_state_life_total_is_20() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.private_player_state.life_total, 20);
    }

    #[test]
    fn private_state_hand_is_visible() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        // 7-card hand from the 60-card deck
        assert_eq!(snap.private_player_state.hand.len(), 7);
    }

    #[test]
    fn private_state_mana_pool_starts_empty() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        let pool = &snap.private_player_state.mana_pool;
        assert_eq!(pool.get("W"), Some(&0));
        assert_eq!(pool.get("U"), Some(&0));
        assert_eq!(pool.get("B"), Some(&0));
        assert_eq!(pool.get("R"), Some(&0));
        assert_eq!(pool.get("G"), Some(&0));
        assert_eq!(pool.get("C"), Some(&0));
    }

    // ---- Opponent state (hidden information) --------------------------------

    #[test]
    fn opponent_state_has_one_opponent() {
        let (game, p1, p2) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.opponent_states.len(), 1);
        assert_eq!(snap.opponent_states[0].player_id, p2);
    }

    #[test]
    fn opponent_hand_is_hidden_but_size_is_visible() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        // Opponent has 7 cards but we only see the count.
        assert_eq!(snap.opponent_states[0].hand_size, 7);
    }

    #[test]
    fn opponent_life_total_is_visible() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(snap.opponent_states[0].life_total, 20);
    }

    // ---- Stack snapshot ----------------------------------------------------

    #[test]
    fn empty_stack_snapshot() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert!(snap.visible_stack.items.is_empty());
    }

    // ---- UI hints ----------------------------------------------------------

    #[test]
    fn viewer_with_priority_can_pass_priority() {
        let (game, p1, _) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        assert_eq!(
            snap.ui_hints.as_ref().unwrap().can_pass_priority,
            true
        );
    }

    #[test]
    fn viewer_without_priority_cannot_pass_priority() {
        let (game, _, p2) = make_started_game();
        let export = game.export_state();
        let snap = create_game_snapshot(&export, &p2, &MockRegistry).unwrap();
        assert_eq!(
            snap.ui_hints.as_ref().unwrap().can_pass_priority,
            false
        );
    }

    // ---- Phase derivation --------------------------------------------------

    #[test]
    fn phase_from_untap_is_beginning() {
        assert_eq!(phase_from_step(Step::Untap), "Beginning");
        assert_eq!(phase_from_step(Step::Upkeep), "Beginning");
        assert_eq!(phase_from_step(Step::Draw), "Beginning");
    }

    #[test]
    fn phase_from_first_main_is_precombat_main() {
        assert_eq!(phase_from_step(Step::FirstMain), "Precombat Main");
    }

    #[test]
    fn phase_from_combat_steps_is_combat() {
        assert_eq!(phase_from_step(Step::BeginningOfCombat), "Combat");
        assert_eq!(phase_from_step(Step::DeclareAttackers), "Combat");
        assert_eq!(phase_from_step(Step::DeclareBlockers), "Combat");
        assert_eq!(phase_from_step(Step::CombatDamage), "Combat");
        assert_eq!(phase_from_step(Step::EndOfCombat), "Combat");
    }

    #[test]
    fn phase_from_second_main_is_postcombat_main() {
        assert_eq!(phase_from_step(Step::SecondMain), "Postcombat Main");
    }

    #[test]
    fn phase_from_ending_steps_is_ending() {
        assert_eq!(phase_from_step(Step::EndStep), "Ending");
        assert_eq!(phase_from_step(Step::Cleanup), "Ending");
    }

    // ---- Player perspective symmetry ---------------------------------------

    #[test]
    fn two_viewers_produce_different_private_states() {
        let (game, p1, p2) = make_started_game();
        let export = game.export_state();
        let snap1 = create_game_snapshot(&export, &p1, &MockRegistry).unwrap();
        let snap2 = create_game_snapshot(&export, &p2, &MockRegistry).unwrap();

        // Each sees themselves as the viewer.
        assert_eq!(snap1.private_player_state.player_id, p1);
        assert_eq!(snap2.private_player_state.player_id, p2);

        // Each sees the other as opponent.
        assert_eq!(snap1.opponent_states[0].player_id, p2);
        assert_eq!(snap2.opponent_states[0].player_id, p1);
    }
}
