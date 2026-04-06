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

/// Creates a test deck with only Ancient Guardians and Forests.
/// Used for K3 (Indestructible) testing. Not a valid Magic deck.
pub fn ancient_guardian_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..24 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::forest(),
            owner_id,
        ));
    }

    for _ in 0..36 {
        deck.push(CardInstance::new(
            Uuid::new_v4().to_string(),
            catalog::ancient_guardian(),
            owner_id,
        ));
    }

    deck
}

/// Test deck: Oakshield Trolls + Forests. K4 (Hexproof).
pub fn hexproof_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..24 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::forest(), owner_id));
    }
    for _ in 0..36 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::oakshield_troll(), owner_id));
    }
    deck
}

/// Test deck: Ironbark Walls + Forests. K9 (CannotAttack/Defender).
pub fn defender_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..24 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::forest(), owner_id));
    }
    for _ in 0..36 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::ironbark_wall(), owner_id));
    }
    deck
}

/// Test deck: Thalia + Forests. R3 (Legendary).
pub fn legendary_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..24 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::forest(), owner_id));
    }
    for _ in 0..36 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::thalia_forest_keeper(), owner_id));
    }
    deck
}

/// Test deck: Sol Ring + Forests + Bears. R5/R15 (Mana ability).
pub fn sol_ring_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..20 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::forest(), owner_id));
    }
    for _ in 0..20 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::sol_ring(), owner_id));
    }
    for _ in 0..20 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::bear(), owner_id));
    }
    deck
}

/// Test deck: Wild Bounty + Sol Ring + Forests. R10 (Triggered ability).
pub fn wild_bounty_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..20 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::forest(), owner_id));
    }
    for _ in 0..20 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::sol_ring(), owner_id));
    }
    for _ in 0..20 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::wild_bounty(), owner_id));
    }
    deck
}

/// Test deck for P2: Reckless Berserkers + Mountains. K2 (Menace) + K10 (MustAttack).
pub fn berserker_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..24 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::mountain(), owner_id));
    }
    for _ in 0..36 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::reckless_berserker(), owner_id));
    }
    deck
}

/// Test deck for P2: Frozen Sentinels + Mountains. K7 (DoesNotUntap).
pub fn frozen_sentinel_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);
    for _ in 0..24 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::mountain(), owner_id));
    }
    for _ in 0..36 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::frozen_sentinel(), owner_id));
    }
    deck
}

/// Layer System showcase deck (60 cards) for the given player.
///
/// Composition:
/// - 14x Island (blue mana for Turn to Frog and Twisted Image)
/// - 10x Forest (green mana for Titanic Growth)
/// - 4x Sol Ring (artifact mana acceleration)
/// - 8x Bear (2/2 — basic targets)
/// - 4x Ironbark Wall (0/4 — excellent target for Twisted Image: becomes 4/0!)
/// - 4x Ancient Guardian (4/5 Indestructible — good target for Turn to Frog)
/// - 4x Turn to Frog (Layer 6 + Layer 7b multi-layer effect)
/// - 4x Twisted Image (Layer 7d switch)
/// - 4x Titanic Growth (Layer 7c pump, +4/+4)
/// - 4x Giant Growth (Layer 7c pump, +3/+3, for layer interaction demo)
pub fn layer_system_test_deck(owner_id: &str) -> Vec<CardInstance> {
    let mut deck = Vec::with_capacity(60);

    for _ in 0..14 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::island(), owner_id));
    }
    for _ in 0..10 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::forest(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::sol_ring(), owner_id));
    }
    for _ in 0..8 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::bear(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::ironbark_wall(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::ancient_guardian(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::turn_to_frog(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::twisted_image(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::titanic_growth(), owner_id));
    }
    for _ in 0..4 {
        deck.push(CardInstance::new(Uuid::new_v4().to_string(), catalog::giant_growth(), owner_id));
    }

    deck
}

/// Selects a P1 test deck by feature name (from TEST_DECK env var).
pub fn p1_test_deck(feature: &str, owner_id: &str) -> Vec<CardInstance> {
    match feature {
        "k3" => ancient_guardian_test_deck(owner_id),
        "k4" => hexproof_test_deck(owner_id),
        "k9" => defender_test_deck(owner_id),
        "r3" => legendary_test_deck(owner_id),
        "r5" => sol_ring_test_deck(owner_id),
        "r10" => wild_bounty_test_deck(owner_id),
        "ls1" => layer_system_test_deck(owner_id),
        _ => green_deck(owner_id),
    }
}

/// Selects a P2 test deck by feature name (from TEST_DECK env var).
pub fn p2_test_deck(feature: &str, owner_id: &str) -> Vec<CardInstance> {
    match feature {
        "k2" | "k10" => berserker_test_deck(owner_id),
        "k7" => frozen_sentinel_test_deck(owner_id),
        _ => red_deck(owner_id),
    }
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
    fn layer_system_test_deck_has_60_cards() {
        let deck = layer_system_test_deck("player-1");
        assert_eq!(deck.len(), 60, "LS1 test deck should have exactly 60 cards");
    }

    #[test]
    fn layer_system_test_deck_composition() {
        let deck = layer_system_test_deck("player-1");

        let islands = deck.iter().filter(|c| c.definition().id() == "island").count();
        let forests = deck.iter().filter(|c| c.definition().id() == "forest").count();
        let sol_rings = deck.iter().filter(|c| c.definition().id() == "sol-ring").count();
        let bears = deck.iter().filter(|c| c.definition().id() == "bear").count();
        let walls = deck.iter().filter(|c| c.definition().id() == "ironbark-wall").count();
        let guardians = deck.iter().filter(|c| c.definition().id() == "ancient-guardian").count();
        let frogs = deck.iter().filter(|c| c.definition().id() == "turn-to-frog").count();
        let images = deck.iter().filter(|c| c.definition().id() == "twisted-image").count();
        let titanic = deck.iter().filter(|c| c.definition().id() == "titanic-growth").count();
        let giant = deck.iter().filter(|c| c.definition().id() == "giant-growth").count();

        assert_eq!(islands, 14, "LS1 deck should have 14 Islands");
        assert_eq!(forests, 10, "LS1 deck should have 10 Forests");
        assert_eq!(sol_rings, 4, "LS1 deck should have 4 Sol Rings");
        assert_eq!(bears, 8, "LS1 deck should have 8 Bears");
        assert_eq!(walls, 4, "LS1 deck should have 4 Ironbark Walls");
        assert_eq!(guardians, 4, "LS1 deck should have 4 Ancient Guardians");
        assert_eq!(frogs, 4, "LS1 deck should have 4 Turn to Frogs");
        assert_eq!(images, 4, "LS1 deck should have 4 Twisted Images");
        assert_eq!(titanic, 4, "LS1 deck should have 4 Titanic Growths");
        assert_eq!(giant, 4, "LS1 deck should have 4 Giant Growths");
    }

    #[test]
    fn p1_test_deck_routes_ls1() {
        let deck = p1_test_deck("ls1", "player-1");
        assert_eq!(deck.len(), 60, "ls1 p1 test deck should have 60 cards");
        // Verify it selected the LS1 deck by checking for a LS1-specific card
        let frogs = deck.iter().filter(|c| c.definition().id() == "turn-to-frog").count();
        assert_eq!(frogs, 4, "ls1 p1 deck should have 4 Turn to Frogs");
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
