//! Prebuilt 60-card starter decks.
//!
//! Each factory function generates a new set of `CardInstance` objects with
//! unique UUIDs for the given player. Mirrors the TypeScript `PrebuiltDecks`
//! from `cards/PrebuiltDecks.ts`.

use uuid::Uuid;

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::cards::catalog;

/// Creates the green starter deck (60 cards) for the given player.
///
/// Composition:
/// - 22x Forest
/// - 18x Bear (2/2)
/// - 14x Giant Growth (instant)
/// - 4x Sol Ring (artifact)
/// - 2x Arcane Sanctum (enchantment)
pub fn green_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..22 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::forest(),
            owner_id,
        ));
    }

    for _ in 0..18 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::bear(),
            owner_id,
        ));
    }

    for _ in 0..14 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::giant_growth(),
            owner_id,
        ));
    }

    for _ in 0..4 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::sol_ring(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::arcane_sanctum(),
            owner_id,
        ));
    }

    deck
}

/// Creates the red starter deck (60 cards) for the given player.
///
/// Composition:
/// - 22x Mountain
/// - 18x Goblin (1/1)
/// - 14x Lightning Strike (instant)
/// - 4x Sol Ring (artifact)
/// - 2x Arcane Sanctum (enchantment)
pub fn red_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..22 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::mountain(),
            owner_id,
        ));
    }

    for _ in 0..18 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::goblin(),
            owner_id,
        ));
    }

    for _ in 0..14 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::lightning_strike(),
            owner_id,
        ));
    }

    for _ in 0..4 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::sol_ring(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::arcane_sanctum(),
            owner_id,
        ));
    }

    deck
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn green_deck_has_60_cards() {
        let deck = green_deck("player-1");
        assert_eq!(deck.len(), 60);
    }

    #[test]
    fn red_deck_has_60_cards() {
        let deck = red_deck("player-1");
        assert_eq!(deck.len(), 60);
    }

    #[test]
    fn green_deck_composition() {
        let deck = green_deck("player-1");

        let forests = deck
            .iter()
            .filter(|c| c.definition().id() == "forest")
            .count();
        let bears = deck
            .iter()
            .filter(|c| c.definition().id() == "bear")
            .count();
        let growths = deck
            .iter()
            .filter(|c| c.definition().id() == "giant-growth")
            .count();

        assert_eq!(forests, 22);
        assert_eq!(bears, 18);
        assert_eq!(growths, 14);

        let sol_rings = deck.iter().filter(|c| c.definition().id() == "sol-ring").count();
        let sanctums = deck.iter().filter(|c| c.definition().id() == "arcane-sanctum").count();
        assert_eq!(sol_rings, 4);
        assert_eq!(sanctums, 2);
    }

    #[test]
    fn red_deck_composition() {
        let deck = red_deck("player-1");

        let mountains = deck
            .iter()
            .filter(|c| c.definition().id() == "mountain")
            .count();
        let goblins = deck
            .iter()
            .filter(|c| c.definition().id() == "goblin")
            .count();
        let strikes = deck
            .iter()
            .filter(|c| c.definition().id() == "lightning-strike")
            .count();

        assert_eq!(mountains, 22);
        assert_eq!(goblins, 18);
        assert_eq!(strikes, 14);

        let sol_rings = deck.iter().filter(|c| c.definition().id() == "sol-ring").count();
        let sanctums = deck.iter().filter(|c| c.definition().id() == "arcane-sanctum").count();
        assert_eq!(sol_rings, 4);
        assert_eq!(sanctums, 2);
    }

    #[test]
    fn all_instance_ids_are_unique() {
        let deck = green_deck("player-1");
        let ids: HashSet<&str> = deck.iter().map(|c| c.instance_id()).collect();
        assert_eq!(ids.len(), 60, "Each card must have a unique instance ID");
    }

    #[test]
    fn all_cards_have_correct_owner() {
        let deck = green_deck("player-42");
        assert!(deck.iter().all(|c| c.owner_id() == "player-42"));
    }

    #[test]
    fn two_decks_for_same_player_have_different_ids() {
        let d1 = green_deck("player-1");
        let d2 = green_deck("player-1");
        let ids1: HashSet<&str> = d1.iter().map(|c| c.instance_id()).collect();
        let ids2: HashSet<&str> = d2.iter().map(|c| c.instance_id()).collect();
        // No overlap expected (UUIDs are unique)
        let overlap: HashSet<_> = ids1.intersection(&ids2).collect();
        assert!(
            overlap.is_empty(),
            "Two generated decks should not share instance IDs"
        );
    }
}
