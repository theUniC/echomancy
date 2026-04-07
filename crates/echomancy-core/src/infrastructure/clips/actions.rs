//! Parse CLIPS `action-*` output facts into `RulesAction` variants.
//!
//! After `ClipsEngine::run()`, CLIPS working memory may contain `action-*`
//! facts asserted by card rules. This module reads those facts, validates
//! each one, and converts them into typed `RulesAction` values that Rust can
//! apply to the game.
//!
//! It also reads `awaiting-input` facts, which signal that a rule called
//! `(halt)` and needs player input before continuing.
//!
//! # Action templates
//!
//! Each `action-*` deftemplate is enumerated explicitly:
//!
//! | Template | RulesAction variant |
//! |----------|---------------------|
//! | action-draw | DrawCards |
//! | action-damage | DealDamage |
//! | action-destroy | DestroyPermanent |
//! | action-gain-life | GainLife |
//! | action-lose-life | LoseLife |
//! | action-move-zone | MoveZone |
//! | action-add-mana | AddMana |
//! | action-tap | Tap |
//! | action-untap | Untap |
//! | action-add-counter | AddCounter |
//! | action-create-token | CreateToken |
//! | action-modify-pt | ModifyPowerToughness |

use crate::domain::rules_engine::{InputRequest, RulesAction, RulesError};
use crate::infrastructure::clips::{ClipsEngine, SlotValue};

/// Parse all `action-*` facts from CLIPS working memory into `RulesAction` values.
///
/// The returned vec is sorted ascending by the `priority` slot value.
/// Actions with missing/invalid required slots are logged and skipped.
pub(crate) fn parse_action_facts(engine: &ClipsEngine) -> Result<Vec<RulesAction>, RulesError> {
    let mut actions: Vec<(i64, RulesAction)> = Vec::new();

    // action-draw
    for row in engine.collect_facts_by_template("action-draw", &["priority", "player", "amount"]) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let player = match extract_string(&row, "player") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        actions.push((priority, RulesAction::DrawCards { player, amount }));
    }

    // action-damage
    for row in engine.collect_facts_by_template(
        "action-damage",
        &["priority", "source", "target", "amount"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        actions.push((priority, RulesAction::DealDamage { source, target, amount }));
    }

    // action-destroy
    for row in engine.collect_facts_by_template("action-destroy", &["priority", "target"]) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        actions.push((priority, RulesAction::DestroyPermanent { target }));
    }

    // action-gain-life
    for row in
        engine.collect_facts_by_template("action-gain-life", &["priority", "player", "amount"])
    {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let player = match extract_string(&row, "player") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        actions.push((priority, RulesAction::GainLife { player, amount }));
    }

    // action-lose-life
    for row in
        engine.collect_facts_by_template("action-lose-life", &["priority", "player", "amount"])
    {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let player = match extract_string(&row, "player") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        actions.push((priority, RulesAction::LoseLife { player, amount }));
    }

    // action-move-zone
    for row in engine.collect_facts_by_template(
        "action-move-zone",
        &["priority", "card-id", "from-zone", "to-zone"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let card_id = match extract_string(&row, "card-id") {
            Some(s) => s,
            None => continue,
        };
        let from_zone = match extract_symbol(&row, "from-zone") {
            Some(s) => s,
            None => continue,
        };
        let to_zone = match extract_symbol(&row, "to-zone") {
            Some(s) => s,
            None => continue,
        };
        actions.push((
            priority,
            RulesAction::MoveZone { card_id, from_zone, to_zone },
        ));
    }

    // action-add-mana
    for row in engine.collect_facts_by_template(
        "action-add-mana",
        &["priority", "player", "color", "amount"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let player = match extract_string(&row, "player") {
            Some(s) => s,
            None => continue,
        };
        let color = match extract_symbol(&row, "color") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        actions.push((priority, RulesAction::AddMana { player, color, amount }));
    }

    // action-tap
    for row in engine.collect_facts_by_template("action-tap", &["priority", "permanent-id"]) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let permanent_id = match extract_string(&row, "permanent-id") {
            Some(s) => s,
            None => continue,
        };
        actions.push((priority, RulesAction::Tap { permanent_id }));
    }

    // action-untap
    for row in engine.collect_facts_by_template("action-untap", &["priority", "permanent-id"]) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let permanent_id = match extract_string(&row, "permanent-id") {
            Some(s) => s,
            None => continue,
        };
        actions.push((priority, RulesAction::Untap { permanent_id }));
    }

    // action-add-counter
    for row in engine.collect_facts_by_template(
        "action-add-counter",
        &["priority", "permanent-id", "counter-type", "amount"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let permanent_id = match extract_string(&row, "permanent-id") {
            Some(s) => s,
            None => continue,
        };
        let counter_type = match extract_string(&row, "counter-type") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        actions.push((
            priority,
            RulesAction::AddCounter { permanent_id, counter_type, amount },
        ));
    }

    // action-counter-spell
    for row in engine.collect_facts_by_template("action-counter-spell", &["priority", "target"]) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        actions.push((priority, RulesAction::CounterSpell { target }));
    }

    // action-modify-pt
    for row in engine.collect_facts_by_template(
        "action-modify-pt",
        &["priority", "source", "target", "power", "toughness", "duration"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(0);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        let power = match extract_integer(&row, "power") {
            Some(n) => n as i32,
            None => continue,
        };
        let toughness = match extract_integer(&row, "toughness") {
            Some(n) => n as i32,
            None => continue,
        };
        let duration = extract_symbol(&row, "duration")
            .unwrap_or_else(|| "until-end-of-turn".to_owned());
        actions.push((
            priority,
            RulesAction::ModifyPowerToughness { source, target, power, toughness, duration },
        ));
    }

    // action-set-pt
    for row in engine.collect_facts_by_template(
        "action-set-pt",
        &["priority", "source", "target", "power", "toughness", "duration"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(0);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        let power = match extract_integer(&row, "power") {
            Some(n) => n as i32,
            None => continue,
        };
        let toughness = match extract_integer(&row, "toughness") {
            Some(n) => n as i32,
            None => continue,
        };
        let duration = extract_symbol(&row, "duration")
            .unwrap_or_else(|| "until-end-of-turn".to_owned());
        actions.push((
            priority,
            RulesAction::SetPowerToughness { source, target, power, toughness, duration },
        ));
    }

    // action-switch-pt
    for row in engine.collect_facts_by_template(
        "action-switch-pt",
        &["priority", "source", "target", "duration"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(0);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        let duration = extract_symbol(&row, "duration")
            .unwrap_or_else(|| "until-end-of-turn".to_owned());
        actions.push((
            priority,
            RulesAction::SwitchPowerToughness { source, target, duration },
        ));
    }

    // action-remove-all-abilities
    for row in engine.collect_facts_by_template(
        "action-remove-all-abilities",
        &["priority", "source", "target", "duration"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(0);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        let duration = extract_symbol(&row, "duration")
            .unwrap_or_else(|| "until-end-of-turn".to_owned());
        actions.push((
            priority,
            RulesAction::RemoveAllAbilities { source, target, duration },
        ));
    }

    // action-create-token (multislot fields read as Void — skipped in M2)
    for row in engine.collect_facts_by_template(
        "action-create-token",
        &["priority", "controller", "name", "power", "toughness"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(100);
        let controller = match extract_string(&row, "controller") {
            Some(s) => s,
            None => continue,
        };
        let name = match extract_string(&row, "name") {
            Some(s) => s,
            None => continue,
        };
        let power = match extract_integer(&row, "power") {
            Some(n) => n as i32,
            None => continue,
        };
        let toughness = match extract_integer(&row, "toughness") {
            Some(n) => n as i32,
            None => continue,
        };
        // Multislot `types` and `keywords` not yet readable (M2 limitation).
        actions.push((
            priority,
            RulesAction::CreateToken {
                controller,
                name,
                power,
                toughness,
                types: Vec::new(),
                keywords: Vec::new(),
            },
        ));
    }

    // action-prevent-damage
    for row in engine.collect_facts_by_template(
        "action-prevent-damage",
        &["priority", "source", "target", "amount", "duration"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(0);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        let amount = match extract_integer(&row, "amount") {
            Some(n) if n >= 0 => n as u32,
            _ => continue,
        };
        let duration = extract_symbol(&row, "duration")
            .unwrap_or_else(|| "next-occurrence".to_owned());
        actions.push((
            priority,
            RulesAction::RegisterPreventionShield { source, target, amount, duration },
        ));
    }

    // action-regenerate
    for row in engine.collect_facts_by_template(
        "action-regenerate",
        &["priority", "source", "target"],
    ) {
        let priority = extract_integer(&row, "priority").unwrap_or(0);
        let source = match extract_string(&row, "source") {
            Some(s) => s,
            None => continue,
        };
        let target = match extract_string(&row, "target") {
            Some(s) => s,
            None => continue,
        };
        actions.push((priority, RulesAction::RegisterRegenerationShield { source, target }));
    }

    // Sort by priority ascending
    actions.sort_by_key(|(priority, _)| *priority);

    Ok(actions.into_iter().map(|(_, action)| action).collect())
}

/// Parse `awaiting-input` facts from CLIPS working memory.
///
/// Returns the first `awaiting-input` fact found, or `None` if none exist.
pub(crate) fn parse_awaiting_input(engine: &ClipsEngine) -> Option<InputRequest> {
    let rows =
        engine.collect_facts_by_template("awaiting-input", &["type", "player", "prompt"]);
    rows.into_iter().next().map(|row| {
        let input_type = extract_symbol(&row, "type").unwrap_or_default();
        let player = extract_string(&row, "player").unwrap_or_default();
        let prompt = extract_string(&row, "prompt").unwrap_or_default();
        InputRequest { input_type, player, prompt }
    })
}

// ============================================================================
// Private helpers
// ============================================================================

fn extract_integer(
    row: &crate::infrastructure::clips::FactRow,
    slot: &str,
) -> Option<i64> {
    match row.slots.get(slot)? {
        SlotValue::Integer(n) => Some(*n),
        _ => None,
    }
}

fn extract_string(
    row: &crate::infrastructure::clips::FactRow,
    slot: &str,
) -> Option<String> {
    match row.slots.get(slot)? {
        SlotValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn extract_symbol(
    row: &crate::infrastructure::clips::FactRow,
    slot: &str,
) -> Option<String> {
    match row.slots.get(slot)? {
        SlotValue::Symbol(s) => Some(s.clone()),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::clips::ClipsEngine;

    const TEMPLATES: &str = include_str!("../../../../../rules/core/templates.clp");

    fn engine_with_templates() -> ClipsEngine {
        let mut engine = ClipsEngine::new().expect("engine");
        engine.load_rules(TEMPLATES).expect("templates");
        engine.reset();
        engine
    }

    // ---- parse_action_facts: action-draw ------------------------------------

    #[test]
    fn parse_action_facts_empty_when_no_actions() {
        let engine = engine_with_templates();
        let actions = parse_action_facts(&engine).unwrap();
        assert!(actions.is_empty());
    }

    #[test]
    fn parse_action_facts_parses_action_draw() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-draw (player "p1") (amount 2))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::DrawCards { player, amount: 2 } if player == "p1"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_damage() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-damage (source "bolt-1") (target "p2") (amount 3))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::DealDamage { source, target, amount: 3 }
                if source == "bolt-1" && target == "p2"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_destroy() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-destroy (target "creature-1"))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::DestroyPermanent { target } if target == "creature-1"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_gain_life() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-gain-life (player "p1") (amount 5))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::GainLife { player, amount: 5 } if player == "p1"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_lose_life() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-lose-life (player "p2") (amount 3))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::LoseLife { player, amount: 3 } if player == "p2"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_move_zone() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(
                r#"(action-move-zone (card-id "creature-1") (from-zone BATTLEFIELD) (to-zone GRAVEYARD))"#,
            )
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::MoveZone { card_id, from_zone, to_zone }
                if card_id == "creature-1"
                    && from_zone == "BATTLEFIELD"
                    && to_zone == "GRAVEYARD"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_add_mana() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-add-mana (player "p1") (color GREEN) (amount 1))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::AddMana { player, color, amount: 1 }
                if player == "p1" && color == "GREEN"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_tap() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-tap (permanent-id "forest-1"))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::Tap { permanent_id } if permanent_id == "forest-1"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_untap() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-untap (permanent-id "creature-1"))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::Untap { permanent_id } if permanent_id == "creature-1"
        ));
    }

    #[test]
    fn parse_action_facts_parses_action_add_counter() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(
                r#"(action-add-counter (permanent-id "creature-1") (counter-type "+1/+1") (amount 2))"#,
            )
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            RulesAction::AddCounter { permanent_id, counter_type, amount: 2 }
                if permanent_id == "creature-1" && counter_type == "+1/+1"
        ));
    }

    #[test]
    fn parse_action_facts_sorts_by_priority_ascending() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-gain-life (priority 200) (player "p1") (amount 3))"#)
            .unwrap();
        engine
            .assert_fact(r#"(action-damage (priority 50) (source "s") (target "p2") (amount 1))"#)
            .unwrap();
        engine
            .assert_fact(r#"(action-draw (priority 100) (player "p1") (amount 1))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 3);

        // Priority 50 (damage) should come first
        assert!(matches!(&actions[0], RulesAction::DealDamage { .. }));
        // Priority 100 (draw) second
        assert!(matches!(&actions[1], RulesAction::DrawCards { .. }));
        // Priority 200 (gain-life) last
        assert!(matches!(&actions[2], RulesAction::GainLife { .. }));
    }

    // ---- parse_action_facts: action-modify-pt ---------------------------------

    #[test]
    fn parse_action_facts_parses_action_modify_pt() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(
                r#"(action-modify-pt (source "gg-1") (target "bear-1") (power 3) (toughness 3) (duration until-end-of-turn))"#,
            )
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(
                &actions[0],
                RulesAction::ModifyPowerToughness { source, target, power: 3, toughness: 3, duration }
                    if source == "gg-1" && target == "bear-1" && duration == "until-end-of-turn"
            ),
            "unexpected action: {:?}",
            actions[0]
        );
    }

    // ---- parse_action_facts: action-prevent-damage --------------------------

    #[test]
    fn parse_action_facts_parses_action_prevent_damage() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(
                r#"(action-prevent-damage (source "mend-1") (target "bear-1") (amount 3) (duration until-depleted))"#,
            )
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(
                &actions[0],
                RulesAction::RegisterPreventionShield { target, amount: 3, duration, source }
                    if target == "bear-1" && duration == "until-depleted" && source == "mend-1"
            ),
            "unexpected action: {:?}",
            actions[0]
        );
    }

    // ---- parse_action_facts: action-regenerate -------------------------------

    #[test]
    fn parse_action_facts_parses_action_regenerate() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(r#"(action-regenerate (source "troll-1") (target "bear-1"))"#)
            .unwrap();

        let actions = parse_action_facts(&engine).unwrap();
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(
                &actions[0],
                RulesAction::RegisterRegenerationShield { target, source }
                    if target == "bear-1" && source == "troll-1"
            ),
            "unexpected action: {:?}",
            actions[0]
        );
    }

    // ---- parse_awaiting_input -----------------------------------------------

    #[test]
    fn parse_awaiting_input_returns_none_when_absent() {
        let engine = engine_with_templates();
        assert!(parse_awaiting_input(&engine).is_none());
    }

    #[test]
    fn parse_awaiting_input_returns_request_when_present() {
        let mut engine = engine_with_templates();
        engine
            .assert_fact(
                r#"(awaiting-input (type sacrifice) (player "p1") (prompt "Choose a creature"))"#,
            )
            .unwrap();

        let req = parse_awaiting_input(&engine).expect("should have awaiting-input");
        assert_eq!(req.input_type, "sacrifice");
        assert_eq!(req.player, "p1");
        assert_eq!(req.prompt, "Choose a creature");
    }
}
