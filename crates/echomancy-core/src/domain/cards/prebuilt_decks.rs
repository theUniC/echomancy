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
/// - 16x Forest
/// - 2x Thornwood Tapland (enters tapped — showcases K8 EntersTapped)
/// - 16x Bear (2/2)
/// - 2x Oakshield Troll (3/3 Hexproof — showcases K4 Hexproof)
/// - 2x Ancient Guardian (4/5 Indestructible — showcases K3 Indestructible)
/// - 2x Ironbark Wall (0/4 CannotAttack — showcases K9 CannotAttack)
/// - 2x Thalia, Forest Keeper (2/2 Legendary First Strike — showcases R3 Legendary)
/// - 12x Giant Growth (instant)
/// - 4x Sol Ring (artifact)
/// - 2x Wild Bounty (enchantment — showcases R10 triggered abilities)
pub fn green_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..16 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::forest(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::thornwood_tapland(),
            owner_id,
        ));
    }

    for _ in 0..16 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::bear(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::oakshield_troll(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::ancient_guardian(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::ironbark_wall(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::thalia_forest_keeper(),
            owner_id,
        ));
    }

    for _ in 0..12 {
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
            catalog::wild_bounty(),
            owner_id,
        ));
    }

    deck
}

/// Creates the red starter deck (60 cards) for the given player.
///
/// Composition:
/// - 18x Mountain
/// - 16x Goblin (1/1)
/// - 2x Reckless Berserker (2/1 Menace + MustAttack — showcases K2, K10)
/// - 4x Frozen Sentinel (3/3 Haste + DoesNotUntap — showcases K7 DoesNotUntap)
/// - 14x Lightning Strike (instant)
/// - 4x Sol Ring (artifact)
/// - 2x Wild Bounty (enchantment — showcases R10 triggered abilities)
pub fn red_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..18 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::mountain(),
            owner_id,
        ));
    }

    for _ in 0..16 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::goblin(),
            owner_id,
        ));
    }

    for _ in 0..2 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::reckless_berserker(),
            owner_id,
        ));
    }

    for _ in 0..4 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::frozen_sentinel(),
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
            catalog::wild_bounty(),
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

        assert_eq!(forests, 16, "green deck should have 16 Forests after showcase additions");
        assert_eq!(bears, 16, "green deck should have 16 Bears after showcase additions");
        assert_eq!(growths, 12, "green deck should have 12 Giant Growths after showcase additions");

        let sol_rings = deck.iter().filter(|c| c.definition().id() == "sol-ring").count();
        let bounties = deck.iter().filter(|c| c.definition().id() == "wild-bounty").count();
        assert_eq!(sol_rings, 4);
        assert_eq!(bounties, 2);

        // Showcase cards: 2x each
        let trolls = deck.iter().filter(|c| c.definition().id() == "oakshield-troll").count();
        let guardians = deck.iter().filter(|c| c.definition().id() == "ancient-guardian").count();
        let walls = deck.iter().filter(|c| c.definition().id() == "ironbark-wall").count();
        let thalias = deck.iter().filter(|c| c.definition().id() == "thalia-forest-keeper").count();
        let taplands = deck.iter().filter(|c| c.definition().id() == "thornwood-tapland").count();
        assert_eq!(trolls, 2, "green deck should have 2x Oakshield Troll");
        assert_eq!(guardians, 2, "green deck should have 2x Ancient Guardian");
        assert_eq!(walls, 2, "green deck should have 2x Ironbark Wall");
        assert_eq!(thalias, 2, "green deck should have 2x Thalia, Forest Keeper");
        assert_eq!(taplands, 2, "green deck should have 2x Thornwood Tapland");
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

        assert_eq!(mountains, 18, "red deck should have 18 Mountains after showcase additions");
        assert_eq!(goblins, 16, "red deck should have 16 Goblins after showcase additions");
        assert_eq!(strikes, 14);

        let sol_rings = deck.iter().filter(|c| c.definition().id() == "sol-ring").count();
        let bounties = deck.iter().filter(|c| c.definition().id() == "wild-bounty").count();
        assert_eq!(sol_rings, 4);
        assert_eq!(bounties, 2);

        // Showcase cards: 2x each
        let berserkers = deck.iter().filter(|c| c.definition().id() == "reckless-berserker").count();
        let sentinels = deck.iter().filter(|c| c.definition().id() == "frozen-sentinel").count();
        assert_eq!(berserkers, 2, "red deck should have 2x Reckless Berserker");
        assert_eq!(sentinels, 4, "red deck should have 4x Frozen Sentinel");
        // Sanity: total is still 60
        assert_eq!(deck.len(), 60, "red deck total should remain 60");
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
