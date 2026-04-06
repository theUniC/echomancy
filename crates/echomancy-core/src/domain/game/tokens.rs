//! Token creation helpers for the `Game` aggregate.
//!
//! All functions in this module create tokens and put them onto the battlefield
//! under a controller's control (CR 111.1).
//!
//! Each token creation function follows the same pattern:
//! 1. Build a `CardDefinition` for the token.
//! 2. Wrap it in a `CardInstance` with a unique `"token-N"` instance ID.
//! 3. Delegate to `enter_battlefield` to place it on the battlefield.

use super::Game;
use crate::domain::enums::{CardType, StaticAbility, ZoneName};
use crate::domain::events::GameEvent;

impl Game {
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
    ) -> Vec<GameEvent> {
        use crate::domain::cards::card_definition::CardDefinition;
        use crate::domain::cards::card_instance::CardInstance;

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
    ) -> Vec<GameEvent> {
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
    ) -> Vec<GameEvent> {
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
    ) -> Vec<GameEvent> {
        self.create_token(
            controller_id,
            "Food",
            0,
            0,
            &["Artifact".to_owned()],
            &[],
        )
    }
}

// ============================================================================
// Private helpers for token creation
// ============================================================================

/// Parse a type string into a `CardType`, used when creating tokens from
/// `RulesAction::CreateToken` which carries type names as strings.
fn parse_card_type(s: &str) -> Option<CardType> {
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
fn parse_static_ability(s: &str) -> Option<StaticAbility> {
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use crate::domain::game::test_helpers::*;

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
}
