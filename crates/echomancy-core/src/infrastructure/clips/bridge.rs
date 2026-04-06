//! Bridge: serialize `Game` state into CLIPS fact strings.
//!
//! This module converts the Rust domain model into CLIPS fact assertion strings
//! ready to be passed to `ClipsEngine::assert_fact()`. It covers the full
//! game state: players, permanents, mana pools, the stack, and turn state.
//!
//! # Escaping
//!
//! CLIPS string slots (type STRING) require double-quote delimiters. Any
//! double-quote characters inside the string value are escaped with a backslash.
//! Symbol slots (type SYMBOL) are unquoted — the value is written verbatim.
//!
//! # No attached facts in M2
//!
//! The `attached` template is defined in `templates.clp` but not serialized
//! here — that is deferred to M4+ (enchantments/equipment system).

use crate::domain::enums::{CardType, ManaColor, StaticAbility};
use crate::domain::entities::the_stack::StackItem;
use crate::domain::events::GameEvent;
use crate::domain::game::Game;

// ============================================================================
// Public API
// ============================================================================

/// Serialize the complete game state into CLIPS fact assertion strings.
///
/// Returns a `Vec` of fact strings, each ready to be passed to
/// `ClipsEngine::assert_fact()`. The order is: players, permanents per player,
/// mana pools, stack items, turn state.
pub(crate) fn serialize_game_state(game: &Game) -> Vec<String> {
    let mut facts = Vec::new();

    let active_player = game.current_player_id();
    let priority_player = game.priority_player_id();

    // One `player` fact per player
    for player_id in game.turn_order() {
        let life = game.player_life_total(player_id).unwrap_or(0);
        let is_active = if player_id == active_player { "TRUE" } else { "FALSE" };
        let has_priority = if priority_player == Some(player_id) { "TRUE" } else { "FALSE" };
        facts.push(format!(
            r#"(player (id {}) (life {}) (is-active {}) (has-priority {}))"#,
            clips_string(player_id),
            life,
            is_active,
            has_priority,
        ));
    }

    // One `permanent` fact per permanent on each player's battlefield
    for player_id in game.turn_order() {
        if let Ok(battlefield) = game.battlefield(player_id) {
            for card in battlefield {
                let instance_id = card.instance_id();
                let card_id = card.definition().id();
                let card_name = card.definition().name();
                let card_type = primary_card_type(card.definition().types());
                let owner_id = card.owner_id();

                // Retrieve tapped / creature state from PermanentState
                let (tapped, summoning_sick, power, toughness, damage) =
                    if let Some(pstate) = game.permanent_state(instance_id) {
                        let tapped = pstate.is_tapped();
                        let (sick, pw, th, dmg) =
                            if let Some(cs) = pstate.creature_state() {
                                (
                                    cs.has_summoning_sickness(),
                                    cs.base_power(),
                                    cs.base_toughness(),
                                    cs.damage_marked_this_turn(),
                                )
                            } else {
                                (false, 0, 0, 0)
                            };
                        (tapped, sick, pw, th, dmg)
                    } else {
                        (false, false, 0, 0, 0)
                    };

                // Keyword list (static abilities as CLIPS symbols).
                // Use the layer pipeline so Layer 6 effects (e.g. RemoveAllAbilities) are
                // reflected in the CLIPS fact rather than the raw card definition.
                let effective_ability_list = game.effective_abilities(instance_id);
                let keywords = if let Some(ref abilities) = effective_ability_list {
                    keyword_symbols(abilities)
                } else {
                    keyword_symbols(card.definition().static_abilities())
                };

                // Counter list: "type count type count ..." (multislot)
                let counters = if let Some(pstate) = game.permanent_state(instance_id) {
                    counter_multislot(pstate)
                } else {
                    String::new()
                };

                facts.push(format!(
                    "(permanent \
                     (instance-id {instance_id_s}) \
                     (card-id {card_id_s}) \
                     (card-name {card_name_s}) \
                     (controller {controller_s}) \
                     (owner {owner_s}) \
                     (card-type {card_type}) \
                     (tapped {tapped}) \
                     (summoning-sick {summoning_sick}) \
                     (power {power}) \
                     (toughness {toughness}) \
                     (damage {damage}) \
                     (keywords{keywords}) \
                     (counters{counters}))",
                    instance_id_s = clips_string(instance_id),
                    card_id_s = clips_string(card_id),
                    card_name_s = clips_string(card_name),
                    controller_s = clips_string(player_id),
                    owner_s = clips_string(owner_id),
                    card_type = card_type,
                    tapped = if tapped { "TRUE" } else { "FALSE" },
                    summoning_sick = if summoning_sick { "TRUE" } else { "FALSE" },
                    power = power,
                    toughness = toughness,
                    damage = damage,
                    keywords = if keywords.is_empty() {
                        String::new()
                    } else {
                        format!(" {keywords}")
                    },
                    counters = if counters.is_empty() {
                        String::new()
                    } else {
                        format!(" {counters}")
                    },
                ));
            }
        }
    }

    // One `mana-pool` fact per player
    for player_id in game.turn_order() {
        if let Ok(pool) = game.mana_pool(player_id) {
            facts.push(format!(
                "(mana-pool \
                 (player-id {player_id_s}) \
                 (white {white}) \
                 (blue {blue}) \
                 (black {black}) \
                 (red {red}) \
                 (green {green}) \
                 (colorless {colorless}))",
                player_id_s = clips_string(player_id),
                white = pool.get(ManaColor::White),
                blue = pool.get(ManaColor::Blue),
                black = pool.get(ManaColor::Black),
                red = pool.get(ManaColor::Red),
                green = pool.get(ManaColor::Green),
                colorless = pool.get(ManaColor::Colorless),
            ));
        }
    }

    // One `stack-item` fact per item on the stack
    for item in game.stack() {
        facts.push(serialize_stack_item(item));
    }

    // One `turn-state` fact
    facts.push(format!(
        "(turn-state \
         (current-step {step}) \
         (active-player {player_s}) \
         (turn-number {turn}))",
        step = game.current_step(),
        player_s = clips_string(game.current_player_id()),
        turn = game.turn_number(),
    ));

    facts
}

/// Serialize a `GameEvent` into a CLIPS `game-event` fact string.
///
/// The `type` slot receives a CLIPS symbol derived from the event discriminant.
/// String data (IDs, zone names) go into the `source-id`, `controller`,
/// `target-id`, and `data` slots as quoted strings.
pub(crate) fn serialize_game_event(event: &GameEvent) -> String {
    match event {
        GameEvent::ZoneChanged { card, from_zone, to_zone, controller_id } => {
            format!(
                "(game-event \
                 (type ZONE_CHANGED) \
                 (source-id {card_id}) \
                 (controller {controller}) \
                 (data {data}))",
                card_id = clips_string(card.instance_id.as_str()),
                controller = clips_string(controller_id.as_str()),
                data = clips_string(&format!("{from_zone}_TO_{to_zone}")),
            )
        }

        GameEvent::StepStarted { step, active_player_id } => {
            format!(
                "(game-event \
                 (type STEP_STARTED) \
                 (controller {player}) \
                 (data {step}))",
                player = clips_string(active_player_id.as_str()),
                step = clips_string(&step.to_string()),
            )
        }

        GameEvent::CreatureDeclaredAttacker { creature, controller_id } => {
            format!(
                "(game-event \
                 (type CREATURE_DECLARED_ATTACKER) \
                 (source-id {card_id}) \
                 (controller {controller}))",
                card_id = clips_string(creature.instance_id.as_str()),
                controller = clips_string(controller_id.as_str()),
            )
        }

        GameEvent::CreatureDeclaredBlocker { creature, controller_id, blocking } => {
            format!(
                "(game-event \
                 (type CREATURE_DECLARED_BLOCKER) \
                 (source-id {card_id}) \
                 (controller {controller}) \
                 (target-id {attacker_id}))",
                card_id = clips_string(creature.instance_id.as_str()),
                controller = clips_string(controller_id.as_str()),
                attacker_id = clips_string(blocking.instance_id.as_str()),
            )
        }

        GameEvent::CombatEnded { active_player_id } => {
            format!(
                "(game-event \
                 (type COMBAT_ENDED) \
                 (controller {player}))",
                player = clips_string(active_player_id.as_str()),
            )
        }

        GameEvent::SpellResolved { card, controller_id, targets } => {
            let target_id = targets.first().map(|t| t.target_id()).unwrap_or("");
            format!(
                "(game-event \
                 (type SPELL_RESOLVING) \
                 (source-id {card_id}) \
                 (controller {controller}) \
                 (target-id {target_id_s}) \
                 (data {def_id}))",
                card_id = clips_string(card.instance_id.as_str()),
                controller = clips_string(controller_id.as_str()),
                target_id_s = clips_string(target_id),
                def_id = clips_string(card.definition_id.as_str()),
            )
        }

        GameEvent::ManaAdded { player_id, color, amount } => {
            format!(
                "(game-event \
                 (type MANA_ADDED) \
                 (controller {player}) \
                 (data {data}))",
                player = clips_string(player_id.as_str()),
                data = clips_string(&format!("{color}:{amount}")),
            )
        }

        GameEvent::TriggeredAbilityFires { source, controller_id, trigger_type } => {
            format!(
                "(game-event \
                 (type TRIGGERED_ABILITY_FIRES) \
                 (source-id {source_id}) \
                 (controller {controller}) \
                 (data {data}) \
                 (target-id {definition_id}))",
                source_id = clips_string(source.instance_id.as_str()),
                controller = clips_string(controller_id.as_str()),
                data = clips_string(trigger_type),
                definition_id = clips_string(source.definition_id.as_str()),
            )
        }
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Wrap a string value in CLIPS double-quote delimiters, escaping any internal
/// double-quote characters with a backslash.
fn clips_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Derive the primary CLIPS card-type symbol from a card's type list.
/// Uses the first type in the list; defaults to `UNKNOWN` if empty.
fn primary_card_type(types: &[CardType]) -> &'static str {
    match types.first() {
        Some(CardType::Creature) => "CREATURE",
        Some(CardType::Instant) => "INSTANT",
        Some(CardType::Sorcery) => "SORCERY",
        Some(CardType::Artifact) => "ARTIFACT",
        Some(CardType::Enchantment) => "ENCHANTMENT",
        Some(CardType::Planeswalker) => "PLANESWALKER",
        Some(CardType::Land) => "LAND",
        Some(CardType::Kindred) => "KINDRED",
        None => "UNKNOWN",
    }
}

/// Build a space-separated list of CLIPS keyword symbols from static abilities.
/// Returns an empty string when there are no abilities.
fn keyword_symbols(abilities: &[StaticAbility]) -> String {
    abilities
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build a space-separated list of `"type" count` pairs for the `counters` multislot.
/// Counter types are serialized as CLIPS strings; counts as integers.
fn counter_multislot(pstate: &crate::domain::value_objects::permanent_state::PermanentState) -> String {
    // We read counters via to_snapshot() to access the HashMap.
    let snap = pstate.to_snapshot();
    let mut parts: Vec<String> = snap
        .counters
        .iter()
        .map(|(k, v)| format!("{} {v}", clips_string(k)))
        .collect();
    // Sort for deterministic output (important for tests).
    parts.sort();
    parts.join(" ")
}

/// Serialize a `StackItem` into a CLIPS `stack-item` fact string.
fn serialize_stack_item(item: &StackItem) -> String {
    match item {
        StackItem::Spell(spell) => {
            let card_id = spell.card.definition().id();
            let instance_id = spell.card.instance_id();
            let target = spell.targets.first().map(target_id).unwrap_or_default();
            format!(
                "(stack-item \
                 (id {id}) \
                 (card-id {card_id_s}) \
                 (controller {controller}) \
                 (status WAITING) \
                 (target {target}))",
                id = clips_string(instance_id),
                card_id_s = clips_string(card_id),
                controller = clips_string(&spell.controller_id),
                target = clips_string(&target),
            )
        }
        StackItem::Ability(ability) => {
            let target = ability.targets.first().map(target_id).unwrap_or_default();
            format!(
                "(stack-item \
                 (id {id}) \
                 (card-id {card_id_s}) \
                 (controller {controller}) \
                 (status WAITING) \
                 (target {target}))",
                id = clips_string(&ability.source_id),
                card_id_s = clips_string(&ability.source_id),
                controller = clips_string(&ability.controller_id),
                target = clips_string(&target),
            )
        }
    }
}

/// Extract an ID string from a `Target`.
fn target_id(target: &crate::domain::targets::Target) -> String {
    target.target_id().to_owned()
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    
    use crate::domain::enums::{CardType, ManaColor, StaticAbility, Step};
    use crate::domain::events::{CardInstanceSnapshot, GameEvent};
    use crate::domain::game::test_helpers::{
        add_permanent_to_battlefield, make_creature_card, make_creature_with_ability,
        make_land_card, make_started_game,
    };
    use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};

    // ---- clips_string -------------------------------------------------------

    #[test]
    fn clips_string_wraps_in_double_quotes() {
        assert_eq!(clips_string("hello"), r#""hello""#);
    }

    #[test]
    fn clips_string_escapes_inner_double_quotes() {
        assert_eq!(clips_string(r#"say "hi""#), r#""say \"hi\"""#);
    }

    #[test]
    fn clips_string_escapes_backslash() {
        assert_eq!(clips_string(r#"a\b"#), r#""a\\b""#);
    }

    // ---- primary_card_type --------------------------------------------------

    #[test]
    fn primary_card_type_creature() {
        assert_eq!(primary_card_type(&[CardType::Creature]), "CREATURE");
    }

    #[test]
    fn primary_card_type_land() {
        assert_eq!(primary_card_type(&[CardType::Land]), "LAND");
    }

    #[test]
    fn primary_card_type_empty_returns_unknown() {
        assert_eq!(primary_card_type(&[]), "UNKNOWN");
    }

    // ---- keyword_symbols ----------------------------------------------------

    #[test]
    fn keyword_symbols_empty_when_no_abilities() {
        assert_eq!(keyword_symbols(&[]), "");
    }

    #[test]
    fn keyword_symbols_single_ability() {
        assert_eq!(keyword_symbols(&[StaticAbility::Flying]), "FLYING");
    }

    #[test]
    fn keyword_symbols_multiple_abilities() {
        let result = keyword_symbols(&[StaticAbility::Flying, StaticAbility::Reach]);
        assert_eq!(result, "FLYING REACH");
    }

    // ---- serialize_game_state: player facts ---------------------------------

    #[test]
    fn serialize_game_state_emits_one_player_fact_per_player() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let player_facts: Vec<_> = facts.iter().filter(|f| f.starts_with("(player")).collect();
        assert_eq!(player_facts.len(), 2, "should have exactly 2 player facts");
    }

    #[test]
    fn serialize_game_state_player_fact_contains_life_total() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let p1_fact = facts
            .iter()
            .find(|f| f.contains(r#"(id "p1")"#))
            .expect("p1 player fact not found");
        assert!(
            p1_fact.contains("(life 20)"),
            "p1 should start with 20 life, got: {p1_fact}"
        );
    }

    #[test]
    fn serialize_game_state_active_player_marked() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let p1_fact = facts
            .iter()
            .find(|f| f.contains(r#"(id "p1")"#))
            .expect("p1 player fact not found");
        // p1 is the active player in make_started_game
        assert!(p1_fact.contains("(is-active TRUE)"), "p1 should be active");

        let p2_fact = facts
            .iter()
            .find(|f| f.contains(r#"(id "p2")"#))
            .expect("p2 player fact not found");
        assert!(p2_fact.contains("(is-active FALSE)"), "p2 should not be active");
    }

    #[test]
    fn serialize_game_state_priority_player_marked() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let p1_fact = facts.iter().find(|f| f.contains(r#"(id "p1")"#)).unwrap();
        assert!(p1_fact.contains("(has-priority TRUE)"), "p1 starts with priority");
    }

    // ---- serialize_game_state: permanent facts ------------------------------

    #[test]
    fn serialize_game_state_emits_no_permanent_facts_for_empty_battlefield() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let perm_facts: Vec<_> = facts.iter().filter(|f| f.starts_with("(permanent")).collect();
        assert!(perm_facts.is_empty(), "no permanents should be on the battlefield");
    }

    #[test]
    fn serialize_game_state_emits_permanent_fact_for_land() {
        let (mut game, p1, _p2) = make_started_game();
        let land = make_land_card("land-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);

        let facts = serialize_game_state(&game);
        let perm_facts: Vec<_> = facts.iter().filter(|f| f.starts_with("(permanent")).collect();
        assert_eq!(perm_facts.len(), 1, "should have one permanent fact");

        let perm = &perm_facts[0];
        assert!(perm.contains(r#"(instance-id "land-1")"#), "instance-id mismatch");
        assert!(perm.contains("(card-type LAND)"), "card-type should be LAND");
        assert!(perm.contains("(tapped FALSE)"), "land should start untapped");
    }

    #[test]
    fn serialize_game_state_creature_permanent_has_stats() {
        let (mut game, p1, _p2) = make_started_game();
        let creature = make_creature_card("bear-1", &p1, 2, 3);
        add_permanent_to_battlefield(&mut game, &p1, creature);

        let facts = serialize_game_state(&game);
        let perm = facts
            .iter()
            .find(|f| f.contains(r#"(instance-id "bear-1")"#))
            .expect("bear permanent fact not found");

        assert!(perm.contains("(power 2)"), "power should be 2");
        assert!(perm.contains("(toughness 3)"), "toughness should be 3");
        assert!(perm.contains("(summoning-sick TRUE)"), "new creature should have summoning sickness");
        assert!(perm.contains("(card-type CREATURE)"), "card-type should be CREATURE");
    }

    #[test]
    fn serialize_game_state_creature_with_keyword_emits_keyword_in_multislot() {
        let (mut game, p1, _p2) = make_started_game();
        let flyer = make_creature_with_ability("flyer-1", &p1, 2, 2, StaticAbility::Flying);
        add_permanent_to_battlefield(&mut game, &p1, flyer);

        let facts = serialize_game_state(&game);
        let perm = facts
            .iter()
            .find(|f| f.contains(r#"(instance-id "flyer-1")"#))
            .expect("flyer permanent fact not found");

        assert!(perm.contains("(keywords FLYING)"), "keywords should contain FLYING, got: {perm}");
    }

    // ---- serialize_game_state: mana-pool facts -------------------------------

    #[test]
    fn serialize_game_state_emits_one_mana_pool_fact_per_player() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let pool_facts: Vec<_> = facts.iter().filter(|f| f.starts_with("(mana-pool")).collect();
        assert_eq!(pool_facts.len(), 2, "should have 2 mana-pool facts");
    }

    #[test]
    fn serialize_game_state_empty_mana_pool_has_all_zeros() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let p1_pool = facts
            .iter()
            .find(|f| f.starts_with("(mana-pool") && f.contains(r#"(player-id "p1")"#))
            .expect("p1 mana-pool fact not found");

        assert!(p1_pool.contains("(white 0)"));
        assert!(p1_pool.contains("(blue 0)"));
        assert!(p1_pool.contains("(red 0)"));
        assert!(p1_pool.contains("(green 0)"));
        assert!(p1_pool.contains("(colorless 0)"));
    }

    // ---- serialize_game_state: turn-state fact -------------------------------

    #[test]
    fn serialize_game_state_emits_turn_state_fact() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let ts_facts: Vec<_> = facts.iter().filter(|f| f.starts_with("(turn-state")).collect();
        assert_eq!(ts_facts.len(), 1, "should have exactly one turn-state fact");
    }

    #[test]
    fn serialize_game_state_turn_state_has_correct_step_and_player() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let ts = facts
            .iter()
            .find(|f| f.starts_with("(turn-state"))
            .expect("turn-state fact not found");

        // make_started_game() starts at Untap step
        assert!(ts.contains("(current-step UNTAP)"), "step should be UNTAP, got: {ts}");
        assert!(ts.contains(r#"(active-player "p1")"#), "p1 is active player");
        assert!(ts.contains("(turn-number 1)"), "first turn");
    }

    // ---- serialize_game_state: stack facts ----------------------------------

    #[test]
    fn serialize_game_state_emits_no_stack_facts_when_empty() {
        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);
        let stack_facts: Vec<_> = facts.iter().filter(|f| f.starts_with("(stack-item")).collect();
        assert!(stack_facts.is_empty(), "stack should be empty");
    }

    // ---- serialize_game_event -----------------------------------------------

    #[test]
    fn serialize_zone_changed_event() {
        use crate::domain::enums::ZoneName;
        let event = GameEvent::ZoneChanged {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("land-1"),
                definition_id: CardDefinitionId::new("forest"),
                owner_id: PlayerId::new("p1"),
            },
            from_zone: ZoneName::Hand,
            to_zone: ZoneName::Battlefield,
            controller_id: PlayerId::new("p1"),
        };
        let fact = serialize_game_event(&event);
        assert!(fact.starts_with("(game-event"), "should be a game-event fact");
        assert!(fact.contains("(type ZONE_CHANGED)"), "should have ZONE_CHANGED type");
        assert!(fact.contains(r#"(source-id "land-1")"#), "should have card instance id");
        assert!(fact.contains(r#"(controller "p1")"#), "should have controller");
    }

    #[test]
    fn serialize_step_started_event() {
        let event = GameEvent::StepStarted {
            step: Step::FirstMain,
            active_player_id: PlayerId::new("p1"),
        };
        let fact = serialize_game_event(&event);
        assert!(fact.contains("(type STEP_STARTED)"), "should have STEP_STARTED type");
        assert!(fact.contains(r#"(controller "p1")"#), "should have active player as controller");
    }

    #[test]
    fn serialize_spell_resolved_event() {
        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("bolt-1"),
                definition_id: CardDefinitionId::new("lightning-bolt"),
                owner_id: PlayerId::new("p1"),
            },
            controller_id: PlayerId::new("p1"),
            targets: vec![],
        };
        let fact = serialize_game_event(&event);
        assert!(fact.contains("(type SPELL_RESOLVING)"), "should use SPELL_RESOLVING type");
        assert!(fact.contains(r#"(source-id "bolt-1")"#), "should have spell instance id");
        assert!(
            fact.contains(r#"(data "lightning-bolt")"#),
            "should include card definition id in data slot, got: {fact}"
        );
    }

    #[test]
    fn serialize_spell_resolved_event_with_player_target() {
        use crate::domain::targets::Target;
        let event = GameEvent::SpellResolved {
            card: CardInstanceSnapshot {
                instance_id: CardInstanceId::new("strike-1"),
                definition_id: CardDefinitionId::new("lightning-strike"),
                owner_id: PlayerId::new("p1"),
            },
            controller_id: PlayerId::new("p1"),
            targets: vec![Target::player("p2")],
        };
        let fact = serialize_game_event(&event);
        assert!(fact.contains("(type SPELL_RESOLVING)"), "should use SPELL_RESOLVING type");
        assert!(
            fact.contains(r#"(target-id "p2")"#),
            "should include target-id in fact, got: {fact}"
        );
    }

    #[test]
    fn serialize_combat_ended_event() {
        let event = GameEvent::CombatEnded {
            active_player_id: PlayerId::new("p1"),
        };
        let fact = serialize_game_event(&event);
        assert!(fact.contains("(type COMBAT_ENDED)"));
        assert!(fact.contains(r#"(controller "p1")"#));
    }

    #[test]
    fn serialize_mana_added_event() {
        let event = GameEvent::ManaAdded {
            player_id: PlayerId::new("p1"),
            color: ManaColor::Green,
            amount: 1,
        };
        let fact = serialize_game_event(&event);
        assert!(fact.contains("(type MANA_ADDED)"));
        assert!(fact.contains(r#"(controller "p1")"#));
    }

    // ---- round-trip: assert game-state facts into CLIPS ----------------------

    #[test]
    fn game_state_facts_can_be_asserted_into_clips_engine() {
        use crate::infrastructure::clips::ClipsEngine;

        let templates = include_str!("../../../../../rules/core/templates.clp");
        let mut engine = ClipsEngine::new().expect("engine creation");
        engine.load_rules(templates).expect("templates should load");
        engine.reset();

        let (game, _p1, _p2) = make_started_game();
        let facts = serialize_game_state(&game);

        for fact in &facts {
            engine
                .assert_fact(fact)
                .unwrap_or_else(|e| panic!("failed to assert fact: {fact}\nerror: {e}"));
        }

        // Verify player facts were asserted
        let player_rows = engine.collect_facts_by_template(
            "player",
            &["id", "life", "is-active", "has-priority"],
        );
        assert_eq!(player_rows.len(), 2, "should have 2 player facts in CLIPS");
    }

    #[test]
    fn permanent_facts_can_be_asserted_into_clips_engine() {
        use crate::infrastructure::clips::ClipsEngine;

        let templates = include_str!("../../../../../rules/core/templates.clp");
        let mut engine = ClipsEngine::new().expect("engine creation");
        engine.load_rules(templates).expect("templates should load");
        engine.reset();

        let (mut game, p1, _p2) = make_started_game();
        let land = make_land_card("land-1", &p1);
        add_permanent_to_battlefield(&mut game, &p1, land);

        let facts = serialize_game_state(&game);
        for fact in &facts {
            engine
                .assert_fact(fact)
                .unwrap_or_else(|e| panic!("failed to assert fact: {fact}\nerror: {e}"));
        }

        let perm_rows = engine.collect_facts_by_template(
            "permanent",
            &["instance-id", "card-type", "tapped"],
        );
        assert_eq!(perm_rows.len(), 1, "should have 1 permanent fact in CLIPS");
    }

    #[test]
    fn game_event_fact_can_be_asserted_into_clips_engine() {
        use crate::infrastructure::clips::ClipsEngine;

        let templates = include_str!("../../../../../rules/core/templates.clp");
        let mut engine = ClipsEngine::new().expect("engine creation");
        engine.load_rules(templates).expect("templates should load");
        engine.reset();

        let event = GameEvent::StepStarted {
            step: Step::FirstMain,
            active_player_id: PlayerId::new("p1"),
        };
        let fact = serialize_game_event(&event);
        engine
            .assert_fact(&fact)
            .unwrap_or_else(|e| panic!("failed to assert event fact: {fact}\nerror: {e}"));

        let rows = engine.collect_facts_by_template("game-event", &["type", "controller"]);
        assert_eq!(rows.len(), 1, "should have 1 game-event fact in CLIPS");
    }
}
