//! MTGJSON AtomicCards.json loader.
//!
//! Parses MTGJSON's AtomicCards format and converts card data into our
//! [`CardDefinition`] domain type.
//!
//! # Format
//!
//! AtomicCards.json is a JSON object where each key is a card name and the
//! value is an array of printings. We use only the first printing (index 0)
//! since AtomicCards guarantees all printings share the same oracle text.
//!
//! This module provides infrastructure for loading card data; not all public
//! items are called from the rest of the codebase yet.
#![allow(dead_code)]

use std::collections::HashMap;

use serde::Deserialize;

use crate::domain::cards::card_definition::CardDefinition;
use crate::domain::enums::{CardType, StaticAbility};
use crate::domain::value_objects::mana::ManaCost;

// ============================================================================
// Error type
// ============================================================================

/// Errors produced when loading or parsing MTGJSON data.
#[derive(Debug, thiserror::Error)]
pub(crate) enum MtgJsonError {
    #[error("failed to parse MTGJSON: {0}")]
    ParseError(String),
}

// ============================================================================
// Raw serde structs
// ============================================================================

/// One card face as represented in AtomicCards.json.
#[derive(Debug, Deserialize)]
struct MtgJsonCard {
    name: String,
    #[serde(rename = "manaCost", default)]
    mana_cost: Option<String>,
    types: Vec<String>,
    #[serde(default)]
    subtypes: Vec<String>,
    #[serde(default)]
    power: Option<String>,
    #[serde(default)]
    toughness: Option<String>,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "colorIdentity", default)]
    color_identity: Vec<String>,
}

// ============================================================================
// Public bulk loader
// ============================================================================

/// Parse an AtomicCards.json string into a map of `card_id → CardDefinition`.
///
/// `card_id` is derived from the card name by lowercasing and replacing spaces
/// with hyphens (e.g. `"Lightning Bolt"` → `"lightning-bolt"`).
///
/// Cards that cannot be parsed (unknown types, star P/T, etc.) are silently
/// skipped. The returned map contains everything that was successfully loaded.
///
/// # Errors
///
/// Returns [`MtgJsonError::ParseError`] only if the top-level JSON structure
/// itself is invalid (i.e. the file is corrupt). Individual card failures are
/// not errors.
pub(crate) fn load_atomic_cards(
    json: &str,
) -> Result<HashMap<String, CardDefinition>, MtgJsonError> {
    // AtomicCards.json wraps the card map inside a `"data"` key.
    // Support both the bare map format (used in tests) and the wrapped format.
    let top: serde_json::Value =
        serde_json::from_str(json).map_err(|e| MtgJsonError::ParseError(e.to_string()))?;

    let card_map = if let Some(data) = top.get("data") {
        data
    } else {
        &top
    };

    let card_map = card_map
        .as_object()
        .ok_or_else(|| MtgJsonError::ParseError("expected JSON object at root".to_string()))?;

    let mut result = HashMap::new();

    for (_card_name, printings_value) in card_map {
        // Each value is an array of printings; take the first one.
        let first = match printings_value.as_array().and_then(|arr| arr.first()) {
            Some(v) => v,
            None => continue,
        };

        let raw: MtgJsonCard = match serde_json::from_value(first.clone()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some(def) = parse_mtgjson_card(&raw) {
            let id = card_name_to_id(&raw.name);
            result.insert(id, def);
        }
    }

    Ok(result)
}

// ============================================================================
// Card conversion
// ============================================================================

/// Convert a single raw MTGJSON card into a [`CardDefinition`].
///
/// Returns `None` for cards that cannot be represented in our domain model
/// (unsupported types, star P/T values, etc.).
fn parse_mtgjson_card(raw: &MtgJsonCard) -> Option<CardDefinition> {
    // Determine card types — skip if none of them map to our CardType enum.
    let types: Vec<CardType> = raw.types.iter().filter_map(|t| map_card_type(t)).collect();
    if types.is_empty() {
        return None;
    }

    let id = card_name_to_id(&raw.name);
    let mut def = CardDefinition::new(id, &raw.name, types.clone());

    // Mana cost
    if let Some(raw_cost) = &raw.mana_cost {
        let converted = convert_mana_cost(raw_cost);
        match ManaCost::parse(&converted) {
            Ok(cost) => def = def.with_mana_cost(cost),
            Err(_) => return None,
        }
    }

    // Power / toughness — only for creatures, skip cards with non-numeric P/T.
    let is_creature = types.contains(&CardType::Creature);
    if is_creature {
        match (&raw.power, &raw.toughness) {
            (Some(p), Some(t)) => match (parse_pt(p), parse_pt(t)) {
                (Some(power), Some(toughness)) => {
                    def = def.with_power_toughness(power, toughness);
                }
                // Non-numeric P/T (e.g. "*") — skip the card.
                _ => return None,
            },
            // Creature with no P/T listed — skip rather than default to 0/0.
            _ => return None,
        }
    }

    // Keywords → static abilities.
    for kw in &raw.keywords {
        if let Some(ability) = map_keyword(kw) {
            def = def.with_static_ability(ability);
        }
    }

    // Suppress unused-field warnings from the compiler — the fields are part
    // of the parsed struct but not yet used in conversion.
    let _ = &raw.subtypes;
    let _ = &raw.text;
    let _ = &raw.color_identity;

    Some(def)
}

// ============================================================================
// Helper functions
// ============================================================================

/// Convert a card name to a stable card ID.
///
/// Rules: lowercase, spaces → hyphens, strip everything except `[a-z0-9-]`.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(card_name_to_id("Lightning Bolt"), "lightning-bolt");
/// assert_eq!(card_name_to_id("Grizzly Bears"), "grizzly-bears");
/// assert_eq!(card_name_to_id("Serra Angel"), "serra-angel");
/// assert_eq!(card_name_to_id("Jace, the Mind Sculptor"), "jace-the-mind-sculptor");
/// ```
pub(crate) fn card_name_to_id(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect()
}

/// Convert an MTGJSON mana cost string to the format expected by [`ManaCost::parse`].
///
/// MTGJSON uses `"{1}{G}"` notation; our parser expects `"1G"`.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(convert_mana_cost("{1}{G}"), "1G");
/// assert_eq!(convert_mana_cost("{R}"), "R");
/// assert_eq!(convert_mana_cost("{3}{W}{W}"), "3WW");
/// ```
pub(crate) fn convert_mana_cost(mtgjson_cost: &str) -> String {
    mtgjson_cost.replace(['{', '}'], "")
}

/// Map an MTGJSON type string to our [`CardType`] enum.
///
/// Returns `None` for types we don't support (Tribal, Conspiracy, etc.).
fn map_card_type(s: &str) -> Option<CardType> {
    match s {
        "Creature" => Some(CardType::Creature),
        "Instant" => Some(CardType::Instant),
        "Sorcery" => Some(CardType::Sorcery),
        "Artifact" => Some(CardType::Artifact),
        "Enchantment" => Some(CardType::Enchantment),
        "Planeswalker" => Some(CardType::Planeswalker),
        "Land" => Some(CardType::Land),
        _ => None,
    }
}

/// Map an MTGJSON keyword string to our [`StaticAbility`] enum.
///
/// Returns `None` for keywords we don't yet model (Trample, Deathtouch, etc.).
fn map_keyword(s: &str) -> Option<StaticAbility> {
    match s.to_lowercase().as_str() {
        "flying" => Some(StaticAbility::Flying),
        "reach" => Some(StaticAbility::Reach),
        "vigilance" => Some(StaticAbility::Vigilance),
        "haste" => Some(StaticAbility::Haste),
        "flash" => Some(StaticAbility::Flash),
        _ => None,
    }
}

/// Parse a power or toughness string into a `u32`.
///
/// Returns `None` for non-numeric values such as `"*"` or `"1+*"`.
fn parse_pt(s: &str) -> Option<u32> {
    s.parse::<u32>().ok()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- card_name_to_id ----------------------------------------------------

    #[test]
    fn card_name_to_id_lightning_bolt() {
        assert_eq!(card_name_to_id("Lightning Bolt"), "lightning-bolt");
    }

    #[test]
    fn card_name_to_id_grizzly_bears() {
        assert_eq!(card_name_to_id("Grizzly Bears"), "grizzly-bears");
    }

    #[test]
    fn card_name_to_id_serra_angel() {
        assert_eq!(card_name_to_id("Serra Angel"), "serra-angel");
    }

    #[test]
    fn card_name_to_id_strips_punctuation() {
        // Commas and apostrophes are removed.
        assert_eq!(
            card_name_to_id("Jace, the Mind Sculptor"),
            "jace-the-mind-sculptor"
        );
    }

    // ---- convert_mana_cost --------------------------------------------------

    #[test]
    fn convert_mana_cost_single_color() {
        assert_eq!(convert_mana_cost("{R}"), "R");
    }

    #[test]
    fn convert_mana_cost_generic_and_color() {
        assert_eq!(convert_mana_cost("{1}{G}"), "1G");
    }

    #[test]
    fn convert_mana_cost_double_white() {
        assert_eq!(convert_mana_cost("{3}{W}{W}"), "3WW");
    }

    #[test]
    fn convert_mana_cost_already_stripped() {
        // Should be a no-op if braces are absent.
        assert_eq!(convert_mana_cost("2UU"), "2UU");
    }

    // ---- parse_mtgjson_card — Lightning Bolt --------------------------------

    fn lightning_bolt_raw() -> MtgJsonCard {
        MtgJsonCard {
            name: "Lightning Bolt".to_string(),
            mana_cost: Some("{R}".to_string()),
            types: vec!["Instant".to_string()],
            subtypes: vec![],
            power: None,
            toughness: None,
            keywords: vec![],
            text: Some("Lightning Bolt deals 3 damage to any target.".to_string()),
            color_identity: vec!["R".to_string()],
        }
    }

    #[test]
    fn lightning_bolt_parses_as_instant() {
        let def = parse_mtgjson_card(&lightning_bolt_raw()).expect("should parse");
        assert!(def.is_instant());
    }

    #[test]
    fn lightning_bolt_has_red_mana_cost() {
        let def = parse_mtgjson_card(&lightning_bolt_raw()).expect("should parse");
        let cost = def.mana_cost().expect("should have mana cost");
        assert_eq!(cost.red, 1);
        assert_eq!(cost.total(), 1);
    }

    #[test]
    fn lightning_bolt_has_no_power_toughness() {
        let def = parse_mtgjson_card(&lightning_bolt_raw()).expect("should parse");
        assert!(def.power().is_none());
        assert!(def.toughness().is_none());
    }

    #[test]
    fn lightning_bolt_id_is_hyphenated() {
        let def = parse_mtgjson_card(&lightning_bolt_raw()).expect("should parse");
        assert_eq!(def.id(), "lightning-bolt");
    }

    // ---- parse_mtgjson_card — Grizzly Bears ---------------------------------

    fn grizzly_bears_raw() -> MtgJsonCard {
        MtgJsonCard {
            name: "Grizzly Bears".to_string(),
            mana_cost: Some("{1}{G}".to_string()),
            types: vec!["Creature".to_string()],
            subtypes: vec!["Bear".to_string()],
            power: Some("2".to_string()),
            toughness: Some("2".to_string()),
            keywords: vec![],
            text: Some(String::new()),
            color_identity: vec!["G".to_string()],
        }
    }

    #[test]
    fn grizzly_bears_parses_as_creature() {
        let def = parse_mtgjson_card(&grizzly_bears_raw()).expect("should parse");
        assert!(def.is_creature());
    }

    #[test]
    fn grizzly_bears_has_correct_mana_cost() {
        let def = parse_mtgjson_card(&grizzly_bears_raw()).expect("should parse");
        let cost = def.mana_cost().expect("should have mana cost");
        assert_eq!(cost.generic, 1);
        assert_eq!(cost.green, 1);
        assert_eq!(cost.total(), 2);
    }

    #[test]
    fn grizzly_bears_has_2_2_stats() {
        let def = parse_mtgjson_card(&grizzly_bears_raw()).expect("should parse");
        assert_eq!(def.power(), Some(2));
        assert_eq!(def.toughness(), Some(2));
    }

    #[test]
    fn grizzly_bears_has_no_static_abilities() {
        let def = parse_mtgjson_card(&grizzly_bears_raw()).expect("should parse");
        assert!(def.static_abilities().is_empty());
    }

    // ---- parse_mtgjson_card — Serra Angel -----------------------------------

    fn serra_angel_raw() -> MtgJsonCard {
        MtgJsonCard {
            name: "Serra Angel".to_string(),
            mana_cost: Some("{3}{W}{W}".to_string()),
            types: vec!["Creature".to_string()],
            subtypes: vec!["Angel".to_string()],
            power: Some("4".to_string()),
            toughness: Some("4".to_string()),
            keywords: vec!["Flying".to_string(), "Vigilance".to_string()],
            text: Some("Flying, vigilance".to_string()),
            color_identity: vec!["W".to_string()],
        }
    }

    #[test]
    fn serra_angel_has_flying_and_vigilance() {
        let def = parse_mtgjson_card(&serra_angel_raw()).expect("should parse");
        assert!(def.has_static_ability(StaticAbility::Flying));
        assert!(def.has_static_ability(StaticAbility::Vigilance));
    }

    #[test]
    fn serra_angel_has_4_4_stats() {
        let def = parse_mtgjson_card(&serra_angel_raw()).expect("should parse");
        assert_eq!(def.power(), Some(4));
        assert_eq!(def.toughness(), Some(4));
    }

    #[test]
    fn serra_angel_mana_cost_total_is_5() {
        let def = parse_mtgjson_card(&serra_angel_raw()).expect("should parse");
        let cost = def.mana_cost().expect("should have mana cost");
        assert_eq!(cost.generic, 3);
        assert_eq!(cost.white, 2);
        assert_eq!(cost.total(), 5);
    }

    // ---- parse_mtgjson_card — Basic land ------------------------------------

    fn forest_raw() -> MtgJsonCard {
        MtgJsonCard {
            name: "Forest".to_string(),
            mana_cost: None,
            types: vec!["Land".to_string()],
            subtypes: vec!["Forest".to_string()],
            power: None,
            toughness: None,
            keywords: vec![],
            text: Some("({T}: Add {G}.)".to_string()),
            color_identity: vec!["G".to_string()],
        }
    }

    #[test]
    fn forest_parses_as_land() {
        let def = parse_mtgjson_card(&forest_raw()).expect("should parse");
        assert!(def.is_land());
    }

    #[test]
    fn forest_has_no_mana_cost() {
        let def = parse_mtgjson_card(&forest_raw()).expect("should parse");
        assert!(def.mana_cost().is_none());
    }

    #[test]
    fn forest_has_no_power_toughness() {
        let def = parse_mtgjson_card(&forest_raw()).expect("should parse");
        assert!(def.power().is_none());
        assert!(def.toughness().is_none());
    }

    // ---- parse_mtgjson_card — Unknown type skipped --------------------------

    #[test]
    fn unknown_card_type_returns_none() {
        let raw = MtgJsonCard {
            name: "Some Conspiracy".to_string(),
            mana_cost: None,
            types: vec!["Conspiracy".to_string()],
            subtypes: vec![],
            power: None,
            toughness: None,
            keywords: vec![],
            text: None,
            color_identity: vec![],
        };
        assert!(parse_mtgjson_card(&raw).is_none());
    }

    // ---- parse_mtgjson_card — Star P/T skipped ------------------------------

    #[test]
    fn star_power_toughness_returns_none() {
        let raw = MtgJsonCard {
            name: "Unbound Flourishing".to_string(),
            mana_cost: Some("{2}{G}".to_string()),
            types: vec!["Creature".to_string()],
            subtypes: vec!["Hydra".to_string()],
            power: Some("*".to_string()),
            toughness: Some("*".to_string()),
            keywords: vec![],
            text: Some("".to_string()),
            color_identity: vec!["G".to_string()],
        };
        assert!(parse_mtgjson_card(&raw).is_none());
    }

    // ---- load_atomic_cards — bulk load --------------------------------------

    fn three_card_json() -> &'static str {
        r#"{
            "Lightning Bolt": [
                {
                    "name": "Lightning Bolt",
                    "manaCost": "{R}",
                    "types": ["Instant"],
                    "subtypes": [],
                    "supertypes": [],
                    "power": null,
                    "toughness": null,
                    "keywords": [],
                    "text": "Lightning Bolt deals 3 damage to any target.",
                    "colorIdentity": ["R"],
                    "type": "Instant"
                }
            ],
            "Grizzly Bears": [
                {
                    "name": "Grizzly Bears",
                    "manaCost": "{1}{G}",
                    "types": ["Creature"],
                    "subtypes": ["Bear"],
                    "supertypes": [],
                    "power": "2",
                    "toughness": "2",
                    "keywords": [],
                    "text": "",
                    "colorIdentity": ["G"],
                    "type": "Creature — Bear"
                }
            ],
            "Serra Angel": [
                {
                    "name": "Serra Angel",
                    "manaCost": "{3}{W}{W}",
                    "types": ["Creature"],
                    "subtypes": ["Angel"],
                    "supertypes": [],
                    "power": "4",
                    "toughness": "4",
                    "keywords": ["Flying", "Vigilance"],
                    "text": "Flying, vigilance",
                    "colorIdentity": ["W"],
                    "type": "Creature — Angel"
                }
            ]
        }"#
    }

    #[test]
    fn bulk_load_returns_correct_card_count() {
        let cards = load_atomic_cards(three_card_json()).expect("should parse");
        assert_eq!(cards.len(), 3);
    }

    #[test]
    fn bulk_load_contains_lightning_bolt() {
        let cards = load_atomic_cards(three_card_json()).expect("should parse");
        let bolt = cards.get("lightning-bolt").expect("should contain bolt");
        assert!(bolt.is_instant());
    }

    #[test]
    fn bulk_load_contains_grizzly_bears() {
        let cards = load_atomic_cards(three_card_json()).expect("should parse");
        let bears = cards.get("grizzly-bears").expect("should contain bears");
        assert!(bears.is_creature());
        assert_eq!(bears.power(), Some(2));
        assert_eq!(bears.toughness(), Some(2));
    }

    #[test]
    fn bulk_load_contains_serra_angel_with_abilities() {
        let cards = load_atomic_cards(three_card_json()).expect("should parse");
        let angel = cards.get("serra-angel").expect("should contain angel");
        assert!(angel.has_static_ability(StaticAbility::Flying));
        assert!(angel.has_static_ability(StaticAbility::Vigilance));
    }

    #[test]
    fn bulk_load_skips_unsupported_cards() {
        let json = r#"{
            "Grizzly Bears": [
                {
                    "name": "Grizzly Bears",
                    "manaCost": "{1}{G}",
                    "types": ["Creature"],
                    "subtypes": ["Bear"],
                    "power": "2",
                    "toughness": "2",
                    "keywords": [],
                    "text": "",
                    "colorIdentity": ["G"]
                }
            ],
            "Some Conspiracy": [
                {
                    "name": "Some Conspiracy",
                    "types": ["Conspiracy"],
                    "subtypes": [],
                    "keywords": [],
                    "text": ""
                }
            ]
        }"#;
        let cards = load_atomic_cards(json).expect("should parse");
        // Only Grizzly Bears should load; the conspiracy is skipped.
        assert_eq!(cards.len(), 1);
        assert!(cards.contains_key("grizzly-bears"));
    }

    #[test]
    fn bulk_load_invalid_json_returns_error() {
        let result = load_atomic_cards("not json at all {{{");
        assert!(result.is_err());
    }

    #[test]
    fn bulk_load_wrapped_in_data_key() {
        let json = format!(r#"{{"data": {}}}"#, three_card_json());
        let cards = load_atomic_cards(&json).expect("should parse wrapped format");
        assert_eq!(cards.len(), 3);
    }
}
