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
/// - 24x Forest
/// - 20x Bear (2/2)
/// - 16x Giant Growth (instant)
pub fn green_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..24 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::forest(),
            owner_id,
        ));
    }

    for _ in 0..20 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::bear(),
            owner_id,
        ));
    }

    for _ in 0..16 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::giant_growth(),
            owner_id,
        ));
    }

    deck
}

/// Creates the red starter deck (60 cards) for the given player.
///
/// Composition:
/// - 24x Mountain
/// - 20x Elite Vanguard (2/1)
/// - 16x Lightning Strike (instant)
pub fn red_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..24 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::mountain(),
            owner_id,
        ));
    }

    for _ in 0..20 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::elite_vanguard(),
            owner_id,
        ));
    }

    for _ in 0..16 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::lightning_strike(),
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

        assert_eq!(forests, 24);
        assert_eq!(bears, 20);
        assert_eq!(growths, 16);
    }

    #[test]
    fn red_deck_composition() {
        let deck = red_deck("player-1");

        let mountains = deck
            .iter()
            .filter(|c| c.definition().id() == "mountain")
            .count();
        let vanguards = deck
            .iter()
            .filter(|c| c.definition().id() == "elite-vanguard")
            .count();
        let strikes = deck
            .iter()
            .filter(|c| c.definition().id() == "lightning-strike")
            .count();

        assert_eq!(mountains, 24);
        assert_eq!(vanguards, 20);
        assert_eq!(strikes, 16);
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
