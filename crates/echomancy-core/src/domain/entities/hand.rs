// NOTE: Not currently used by the Game aggregate. Kept for potential future direct-zone APIs.

//! Hand zone entity — cards held by a player.
//!
//! Mirrors the TypeScript `Hand` class from `game/entities/Hand.ts`.

use crate::domain::cards::card_instance::CardInstance;

/// A player's hand zone.
///
/// Mutation always produces a new `Hand` (value-object style).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hand {
    cards: Vec<CardInstance>,
}

impl Hand {
    /// Create an empty hand.
    pub fn empty() -> Self {
        Hand { cards: Vec::new() }
    }

    /// Create a hand from an existing collection of cards.
    pub fn from_cards(cards: Vec<CardInstance>) -> Self {
        Hand { cards }
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Returns `true` if the hand is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Number of cards in hand.
    pub fn count(&self) -> usize {
        self.cards.len()
    }

    /// Returns all cards in insertion order.
    pub fn get_all(&self) -> &[CardInstance] {
        &self.cards
    }

    /// Find a card by its instance ID.
    pub fn find_card(&self, instance_id: &str) -> Option<&CardInstance> {
        self.cards.iter().find(|c| c.instance_id() == instance_id)
    }

    // -------------------------------------------------------------------------
    // Mutations (return new Hand)
    // -------------------------------------------------------------------------

    /// Add a card to the hand and return a new `Hand`.
    pub fn add_card(&self, card: CardInstance) -> Hand {
        let mut cards = Vec::with_capacity(self.cards.len() + 1);
        cards.extend_from_slice(&self.cards);
        cards.push(card);
        Hand { cards }
    }

    /// Remove a card by instance ID and return a new `Hand`.
    ///
    /// If the card does not exist the returned hand still contains all
    /// existing cards (no error, mirrors TS behaviour).
    pub fn remove_card(&self, instance_id: &str) -> Hand {
        let cards = self
            .cards
            .iter()
            .filter(|c| c.instance_id() != instance_id)
            .cloned()
            .collect();
        Hand { cards }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_instance::test_helpers::make_creature;

    #[test]
    fn empty_hand() {
        let hand = Hand::empty();
        assert!(hand.is_empty());
        assert_eq!(hand.count(), 0);
        assert!(hand.get_all().is_empty());
    }

    #[test]
    fn from_cards() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let hand = Hand::from_cards(vec![c1, c2]);
        assert_eq!(hand.count(), 2);
    }

    #[test]
    fn add_card() {
        let hand = Hand::empty();
        let c = make_creature("c1", "p1");
        let hand2 = hand.add_card(c.clone());
        assert_eq!(hand2.count(), 1);
        // original unchanged
        assert_eq!(hand.count(), 0);
    }

    #[test]
    fn remove_card() {
        let c = make_creature("c1", "p1");
        let hand = Hand::from_cards(vec![c.clone()]);
        let hand2 = hand.remove_card("c1");
        assert!(hand2.is_empty());
        // original unchanged
        assert_eq!(hand.count(), 1);
    }

    #[test]
    fn remove_non_existent_card() {
        let c = make_creature("c1", "p1");
        let hand = Hand::from_cards(vec![c]);
        let hand2 = hand.remove_card("missing");
        assert_eq!(hand2.count(), 1);
    }

    #[test]
    fn find_card_returns_correct_card() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let hand = Hand::from_cards(vec![c1, c2]);
        assert_eq!(hand.find_card("c2").map(|c| c.instance_id()), Some("c2"));
    }

    #[test]
    fn find_card_returns_none_for_missing() {
        let hand = Hand::empty();
        assert!(hand.find_card("missing").is_none());
    }

    #[test]
    fn get_all_preserves_order() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let hand = Hand::from_cards(vec![c1, c2]);
        let all = hand.get_all();
        assert_eq!(all[0].instance_id(), "c1");
        assert_eq!(all[1].instance_id(), "c2");
    }
}
