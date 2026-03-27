//! GameStateExport — serialisable snapshot of the complete game state.
//!
//! Provides the export data types and a pure conversion function from a
//! context trait to `GameStateExport`.  The Game aggregate (Phase 6) will
//! implement `ExportableGameContext` to produce snapshots.
//!
//! Mirrors `GameStateExport.ts` and `GameStateExporter.ts`.
//!
//! Design notes:
//! - The export is neutral (not UI-oriented).
//! - Complete: no hidden information (hands, libraries are included).
//! - Plain data only: no methods, no behaviour.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::enums::{CardType, GameLifecycleState, StaticAbility, Step};
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;

// ============================================================================
// Export types
// ============================================================================

/// Mana pool snapshot for export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManaPoolExport {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
}

/// Creature-specific state export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CreatureStateExport {
    pub is_tapped: bool,
    pub is_attacking: bool,
    pub has_attacked_this_turn: bool,
    pub has_summoning_sickness: bool,
    pub power: i32,
    pub toughness: i32,
    pub damage_marked_this_turn: i32,
    pub blocking_creature_id: Option<String>,
    pub blocked_by: Option<String>,
    /// Counter type name → count. Includes all counter types on the creature.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub counters: HashMap<String, u32>,
}

/// Card instance export representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CardInstanceExport {
    pub instance_id: String,
    pub owner_id: String,
    pub controller_id: String,
    pub card_definition_id: String,
    pub types: Vec<CardType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub static_abilities: Vec<StaticAbility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toughness: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creature_state: Option<CreatureStateExport>,
    /// Planeswalker state — placeholder for future expansion.
    pub is_planeswalker: bool,
}

/// Zone export — all cards in a zone, unfiltered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ZoneExport {
    pub cards: Vec<CardInstanceExport>,
}

/// Stack item kind for export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum StackItemKind {
    Spell,
    ActivatedAbility,
    TriggeredAbility,
}

/// A stack item export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StackItemExport {
    pub kind: StackItemKind,
    pub source_card_instance_id: String,
    pub source_card_definition_id: String,
    pub controller_id: String,
    /// Target instance IDs.
    pub targets: Vec<String>,
}

/// Zones for a player in the export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlayerZonesExport {
    pub hand: ZoneExport,
    pub battlefield: ZoneExport,
    pub graveyard: ZoneExport,
    pub library: ZoneExport,
}

/// Per-player state export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PlayerStateExport {
    pub life_total: i32,
    pub mana_pool: ManaPoolExport,
    pub played_lands_this_turn: u32,
    pub zones: PlayerZonesExport,
}

/// Win outcome export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WinOutcomeExport {
    pub winner_id: String,
    pub reason: String,
}

/// Draw outcome export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DrawOutcomeExport {
    pub reason: String,
}

/// Game outcome export (win or draw).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum GameOutcomeExport {
    Win(WinOutcomeExport),
    Draw(DrawOutcomeExport),
}

/// The complete game state export.
///
/// INVARIANTS:
/// - Every card instance referenced exists exactly once.
/// - No derived or computed UI state.
/// - No validation logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GameStateExport {
    pub game_id: String,
    pub lifecycle_state: GameLifecycleState,
    pub outcome: Option<GameOutcomeExport>,
    pub current_turn_number: u32,
    pub current_player_id: String,
    pub current_step: Step,
    pub priority_player_id: Option<String>,
    pub turn_order: Vec<String>,
    pub players: HashMap<String, PlayerStateExport>,
    pub stack: Vec<StackItemExport>,
    pub scheduled_steps: Vec<Step>,
    pub resume_step_after_scheduled: Option<Step>,
}

// ============================================================================
// Context trait
// ============================================================================

/// Read-only context interface for exporting game state.
///
/// The Game aggregate (Phase 6) will implement this trait. In tests, minimal
/// structs can implement it.
///
/// Mirrors the shape of `GameStateExporter.ts`.
pub(crate) trait ExportableGameContext {
    /// The unique identifier for this game.
    fn game_id(&self) -> &str;

    /// The current lifecycle state of the game.
    fn lifecycle_state(&self) -> GameLifecycleState;

    /// The current turn number.
    fn current_turn_number(&self) -> u32;

    /// The ID of the player whose turn it currently is.
    fn current_player_id(&self) -> &str;

    /// The current step or phase.
    fn current_step(&self) -> Step;

    /// The ID of the player who currently holds priority, if any.
    fn priority_player_id(&self) -> Option<&str>;

    /// All player IDs in turn order.
    fn turn_order(&self) -> &[String];

    /// The life total for the given player.
    fn player_life_total(&self, player_id: &str) -> i32;

    /// The number of lands the given player has played this turn.
    fn played_lands_this_turn(&self, player_id: &str) -> u32;

    /// The mana pool for the given player.
    fn player_mana_pool(&self, player_id: &str) -> &ManaPool;

    /// All cards in the given player's hand.
    fn hand_cards(&self, player_id: &str) -> &[CardInstance];

    /// All cards on the given player's battlefield.
    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance];

    /// All cards in the given player's graveyard.
    fn graveyard_cards(&self, player_id: &str) -> &[CardInstance];

    /// All cards in the given player's library, in order (top = index 0).
    fn library_cards(&self, player_id: &str) -> &[CardInstance];

    /// The `PermanentState` for the given card instance, if it is on a
    /// battlefield.
    fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState>;

    /// All items currently on the stack, from bottom (index 0) to top.
    fn stack_items(&self) -> Vec<StackItemExport>;
}

/// Produces a complete `GameStateExport` from any type that implements
/// `ExportableGameContext`.
pub(crate) fn export_game_state(ctx: &impl ExportableGameContext) -> GameStateExport {
    let mut players = HashMap::new();
    for player_id in ctx.turn_order() {
        let mana_pool = export_mana_pool(ctx.player_mana_pool(player_id));

        let hand = ZoneExport {
            cards: ctx
                .hand_cards(player_id)
                .iter()
                .map(|c| export_card_instance(c, player_id, None))
                .collect(),
        };

        let battlefield = ZoneExport {
            cards: ctx
                .battlefield_cards(player_id)
                .iter()
                .map(|c| {
                    let state = ctx.permanent_state(c.instance_id());
                    export_card_instance(c, player_id, state)
                })
                .collect(),
        };

        let graveyard = ZoneExport {
            cards: ctx
                .graveyard_cards(player_id)
                .iter()
                .map(|c| export_card_instance(c, player_id, None))
                .collect(),
        };

        let library = ZoneExport {
            cards: ctx
                .library_cards(player_id)
                .iter()
                .map(|c| export_card_instance(c, player_id, None))
                .collect(),
        };

        players.insert(
            player_id.clone(),
            PlayerStateExport {
                life_total: ctx.player_life_total(player_id),
                mana_pool,
                played_lands_this_turn: ctx.played_lands_this_turn(player_id),
                zones: PlayerZonesExport {
                    hand,
                    battlefield,
                    graveyard,
                    library,
                },
            },
        );
    }

    GameStateExport {
        game_id: ctx.game_id().to_owned(),
        lifecycle_state: ctx.lifecycle_state(),
        outcome: None,
        current_turn_number: ctx.current_turn_number(),
        current_player_id: ctx.current_player_id().to_owned(),
        current_step: ctx.current_step(),
        priority_player_id: ctx.priority_player_id().map(str::to_owned),
        turn_order: ctx.turn_order().to_vec(),
        players,
        stack: ctx.stack_items(),
        scheduled_steps: Vec::new(),
        resume_step_after_scheduled: None,
    }
}

// ============================================================================
// Conversion helpers (pure functions)
// ============================================================================

/// Converts a `ManaPool` to a `ManaPoolExport`.
pub(crate) fn export_mana_pool(pool: &ManaPool) -> ManaPoolExport {
    use crate::domain::enums::ManaColor;
    ManaPoolExport {
        white: pool.get(ManaColor::White),
        blue: pool.get(ManaColor::Blue),
        black: pool.get(ManaColor::Black),
        red: pool.get(ManaColor::Red),
        green: pool.get(ManaColor::Green),
        colorless: pool.get(ManaColor::Colorless),
    }
}

/// Converts a `CardInstance` with its `PermanentState` to a `CardInstanceExport`.
///
/// `controller_id` is the ID of the player who controls this card.
/// `permanent_state` is `Some` for permanents on the battlefield with a
/// creature sub-state, `None` for cards in other zones or non-creature
/// permanents.
/// `is_planeswalker` marks cards of type Planeswalker.
pub(crate) fn export_card_instance(
    card: &CardInstance,
    controller_id: &str,
    permanent_state: Option<&PermanentState>,
) -> CardInstanceExport {
    let def = card.definition();
    let is_planeswalker = def.types().contains(&CardType::Planeswalker);

    let creature_state = permanent_state.and_then(export_creature_state);

    CardInstanceExport {
        instance_id: card.instance_id().to_owned(),
        owner_id: card.owner_id().to_owned(),
        controller_id: controller_id.to_owned(),
        card_definition_id: def.id().to_owned(),
        types: def.types().to_vec(),
        static_abilities: def.static_abilities().to_vec(),
        power: def.power(),
        toughness: def.toughness(),
        creature_state,
        is_planeswalker,
    }
}

/// Converts a `PermanentState` to a `CreatureStateExport` if it has creature
/// sub-state; returns `None` otherwise.
pub(crate) fn export_creature_state(state: &PermanentState) -> Option<CreatureStateExport> {
    let cs = state.creature_state()?;
    let power = state.current_power().ok()?;
    let toughness = state.current_toughness().ok()?;

    // Collect all counters from the snapshot.
    let counters = state.to_snapshot().counters;

    Some(CreatureStateExport {
        is_tapped: state.is_tapped(),
        is_attacking: cs.is_attacking,
        has_attacked_this_turn: cs.has_attacked_this_turn,
        has_summoning_sickness: cs.has_summoning_sickness,
        power,
        toughness,
        damage_marked_this_turn: cs.damage_marked_this_turn,
        blocking_creature_id: cs
            .blocking_creature_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        blocked_by: cs.blocked_by.as_ref().map(|id| id.as_str().to_owned()),
        counters,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::enums::{CardType, ManaColor};
    use crate::domain::types::CardInstanceId;
    use crate::domain::value_objects::mana::ManaPool;
    use crate::domain::value_objects::permanent_state::PermanentState;

    fn make_creature(id: &str, owner: &str, power: u32, toughness: u32) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Creature])
            .with_power_toughness(power, toughness);
        CardInstance::new(id, def, owner)
    }

    fn make_land(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Land]);
        CardInstance::new(id, def, owner)
    }

    fn make_planeswalker(id: &str, owner: &str) -> CardInstance {
        let def = CardDefinition::new(id, id, vec![CardType::Planeswalker]);
        CardInstance::new(id, def, owner)
    }

    // ---- export_mana_pool --------------------------------------------------

    #[test]
    fn export_mana_pool_reflects_all_colors() {
        let pool = ManaPool::empty()
            .add(ManaColor::White, 1)
            .unwrap()
            .add(ManaColor::Blue, 2)
            .unwrap()
            .add(ManaColor::Colorless, 3)
            .unwrap();

        let export = export_mana_pool(&pool);
        assert_eq!(export.white, 1);
        assert_eq!(export.blue, 2);
        assert_eq!(export.black, 0);
        assert_eq!(export.red, 0);
        assert_eq!(export.green, 0);
        assert_eq!(export.colorless, 3);
    }

    #[test]
    fn export_empty_mana_pool_all_zeros() {
        let export = export_mana_pool(&ManaPool::empty());
        assert_eq!(export.white, 0);
        assert_eq!(export.blue, 0);
        assert_eq!(export.black, 0);
        assert_eq!(export.red, 0);
        assert_eq!(export.green, 0);
        assert_eq!(export.colorless, 0);
    }

    // ---- export_card_instance: land / non-creature -------------------------

    #[test]
    fn export_land_has_no_creature_state() {
        let card = make_land("forest", "p1");
        let export = export_card_instance(&card, "p1", None);
        assert_eq!(export.instance_id, "forest");
        assert_eq!(export.owner_id, "p1");
        assert_eq!(export.controller_id, "p1");
        assert_eq!(export.card_definition_id, "forest");
        assert_eq!(export.types, vec![CardType::Land]);
        assert!(export.creature_state.is_none());
        assert!(!export.is_planeswalker);
    }

    // ---- export_card_instance: creature ------------------------------------

    #[test]
    fn export_creature_includes_creature_state() {
        let card = make_creature("bear", "p1", 2, 2);
        let state = PermanentState::for_creature(2, 2)
            .with_summoning_sickness(false)
            .unwrap();

        let export = export_card_instance(&card, "p1", Some(&state));
        assert_eq!(export.power, Some(2));
        assert_eq!(export.toughness, Some(2));

        let cs = export.creature_state.unwrap();
        assert_eq!(cs.power, 2);
        assert_eq!(cs.toughness, 2);
        assert!(!cs.is_tapped);
        assert!(!cs.is_attacking);
        assert!(!cs.has_summoning_sickness);
        assert_eq!(cs.damage_marked_this_turn, 0);
        assert!(cs.blocking_creature_id.is_none());
        assert!(cs.blocked_by.is_none());
    }

    #[test]
    fn export_attacking_creature_state() {
        let card = make_creature("bear", "p1", 3, 3);
        let state = PermanentState::for_creature(3, 3)
            .with_summoning_sickness(false)
            .unwrap()
            .with_attacking(true)
            .unwrap()
            .with_has_attacked_this_turn(true)
            .unwrap()
            .with_tapped(true);

        let export = export_card_instance(&card, "p1", Some(&state));
        let cs = export.creature_state.unwrap();
        assert!(cs.is_attacking);
        assert!(cs.has_attacked_this_turn);
        assert!(cs.is_tapped);
    }

    #[test]
    fn export_creature_with_damage_and_blocking() {
        let card = make_creature("bear", "p1", 2, 3);
        let state = PermanentState::for_creature(2, 3)
            .with_summoning_sickness(false)
            .unwrap()
            .with_damage(2)
            .unwrap()
            .with_blocking_creature_id(Some(CardInstanceId::new("attacker-1")))
            .unwrap();

        let export = export_card_instance(&card, "p1", Some(&state));
        let cs = export.creature_state.unwrap();
        assert_eq!(cs.damage_marked_this_turn, 2);
        assert_eq!(cs.blocking_creature_id, Some("attacker-1".to_owned()));
        assert!(cs.blocked_by.is_none());
    }

    #[test]
    fn export_creature_with_plus_counters_shows_boosted_stats() {
        let card = make_creature("bear", "p1", 2, 2);
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 2);

        let export = export_card_instance(&card, "p1", Some(&state));
        let cs = export.creature_state.unwrap();
        assert_eq!(cs.power, 4); // 2 + 2 counters
        assert_eq!(cs.toughness, 4);
    }

    // ---- export_card_instance: planeswalker --------------------------------

    #[test]
    fn export_planeswalker_sets_flag() {
        let card = make_planeswalker("gideon", "p1");
        let export = export_card_instance(&card, "p1", None);
        assert!(export.is_planeswalker);
    }

    // ---- export_creature_state directly -----------------------------------

    #[test]
    fn export_creature_state_returns_none_for_non_creature() {
        let state = PermanentState::for_non_creature();
        assert!(export_creature_state(&state).is_none());
    }

    #[test]
    fn export_creature_state_includes_counters() {
        let card = make_creature("bear", "p1", 2, 2);
        let state = PermanentState::for_creature(2, 2)
            .add_counters("PLUS_ONE_PLUS_ONE", 3)
            .add_counters("ICE", 1);

        let export = export_card_instance(&card, "p1", Some(&state));
        let cs = export.creature_state.unwrap();
        assert_eq!(cs.counters.get("PLUS_ONE_PLUS_ONE"), Some(&3));
        assert_eq!(cs.counters.get("ICE"), Some(&1));
    }

    #[test]
    fn export_creature_state_counters_empty_when_none() {
        let card = make_creature("bear", "p1", 2, 2);
        let state = PermanentState::for_creature(2, 2)
            .with_summoning_sickness(false)
            .unwrap();

        let export = export_card_instance(&card, "p1", Some(&state));
        let cs = export.creature_state.unwrap();
        assert!(cs.counters.is_empty());
    }

    // ---- Serialisation roundtrip ------------------------------------------

    #[test]
    fn mana_pool_export_serde_roundtrip() {
        let export = ManaPoolExport {
            white: 1,
            blue: 0,
            black: 2,
            red: 0,
            green: 3,
            colorless: 0,
        };
        let json = serde_json::to_string(&export).unwrap();
        let decoded: ManaPoolExport = serde_json::from_str(&json).unwrap();
        assert_eq!(export, decoded);
    }
}
