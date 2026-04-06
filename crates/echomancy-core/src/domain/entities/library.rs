// NOTE: Not currently used by the Game aggregate. Kept for potential future direct-zone APIs.

//! Library (deck) zone entity.
//!
//! The library is an ordered zone. Cards are drawn from the top (index 0).
//! Shuffling uses `rand::SeedableRng` for deterministic tests.

use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::domain::cards::card_instance::CardInstance;

/// A player's library (deck) zone.
///
/// Cards are ordered top-to-bottom: index 0 is the top card.
/// Mutation always produces a new `Library` (value-object style).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Library {
    /// Cards in top-to-bottom order (index 0 = top).
    cards: Vec<CardInstance>,
}

impl Library {
    /// Create an empty library.
    pub fn empty() -> Self {
        Library { cards: Vec::new() }
    }

    /// Create a library from an ordered list of card instances.
    ///
    /// Index 0 is treated as the top of the library.
    pub fn from_cards(cards: Vec<CardInstance>) -> Self {
        Library { cards }
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Returns `true` if the library has no cards.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Number of cards in the library.
    pub fn count(&self) -> usize {
        self.cards.len()
    }

    /// Returns all cards in top-to-bottom order (read-only).
    pub fn get_all(&self) -> &[CardInstance] {
        &self.cards
    }

    /// Peek at the top `n` cards without removing them.
    ///
    /// Returns fewer cards than requested if the library is smaller.
    pub fn peek_top(&self, n: usize) -> &[CardInstance] {
        let end = n.min(self.cards.len());
        &self.cards[..end]
    }

    // -------------------------------------------------------------------------
    // Mutations (return new Library)
    // -------------------------------------------------------------------------

    /// Draw the top card, returning it alongside the remaining library.
    ///
    /// Returns `(None, self_unchanged)` if the library is empty.
    pub fn draw_from_top(&self) -> (Option<CardInstance>, Library) {
        if self.cards.is_empty() {
            return (None, self.clone());
        }
        let card = self.cards[0].clone();
        let remaining = Library {
            cards: self.cards[1..].to_vec(),
        };
        (Some(card), remaining)
    }

    /// Add a card to the top of the library.
    pub fn add_to_top(&self, card: CardInstance) -> Library {
        let mut cards = Vec::with_capacity(self.cards.len() + 1);
        cards.push(card);
        cards.extend_from_slice(&self.cards);
        Library { cards }
    }

    /// Add a card to the bottom of the library.
    pub fn add_to_bottom(&self, card: CardInstance) -> Library {
        let mut cards = self.cards.clone();
        cards.push(card);
        Library { cards }
    }

    /// Shuffle the library using the Fisher-Yates algorithm.
    ///
    /// Providing `seed` gives a deterministic result (useful for tests).
    /// `None` draws entropy from the OS.
    ///
    /// # MTG Rule 103.1
    ///
    /// Each player shuffles their deck before the game begins. `shuffle()` must
    /// be called during game initialization to comply with this rule.
    pub fn shuffle(&self, seed: Option<u64>) -> Library {
        let mut cards = self.cards.clone();
        match seed {
            Some(s) => {
                let mut rng = rand::rngs::SmallRng::seed_from_u64(s);
                cards.shuffle(&mut rng);
            }
            None => {
                let mut rng = rand::rngs::SmallRng::from_os_rng();
                cards.shuffle(&mut rng);
            }
        }
        Library { cards }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_instance::test_helpers::make_creature;
    use std::collections::HashSet;

    fn make_deck(n: usize, owner: &str) -> Vec<CardInstance> {
        (1..=n)
            .map(|i| make_creature(&format!("card-{i}"), owner))
            .collect()
    }

    #[test]
    fn empty_library() {
        let lib = Library::empty();
        assert!(lib.is_empty());
        assert_eq!(lib.count(), 0);
    }

    #[test]
    fn from_cards_correct_count() {
        let lib = Library::from_cards(make_deck(3, "p1"));
        assert_eq!(lib.count(), 3);
        assert!(!lib.is_empty());
    }

    #[test]
    fn preserves_card_order() {
        let cards = make_deck(3, "p1");
        let lib = Library::from_cards(cards.clone());
        assert_eq!(lib.peek_top(1)[0].instance_id(), "card-1");
    }

    #[test]
    fn draw_from_top_returns_top_card() {
        let lib = Library::from_cards(make_deck(2, "p1"));
        let (card, remaining) = lib.draw_from_top();
        assert_eq!(card.unwrap().instance_id(), "card-1");
        assert_eq!(remaining.count(), 1);
        // original unchanged
        assert_eq!(lib.count(), 2);
    }

    #[test]
    fn draw_from_empty_returns_none() {
        let lib = Library::empty();
        let (card, remaining) = lib.draw_from_top();
        assert!(card.is_none());
        assert!(remaining.is_empty());
    }

    #[test]
    fn draw_preserves_order_of_remaining() {
        let lib = Library::from_cards(make_deck(3, "p1"));
        let (first, lib2) = lib.draw_from_top();
        assert_eq!(first.unwrap().instance_id(), "card-1");
        let (second, lib3) = lib2.draw_from_top();
        assert_eq!(second.unwrap().instance_id(), "card-2");
        let (third, lib4) = lib3.draw_from_top();
        assert_eq!(third.unwrap().instance_id(), "card-3");
        assert!(lib4.is_empty());
    }

    #[test]
    fn peek_top_returns_n_cards() {
        let lib = Library::from_cards(make_deck(3, "p1"));
        let peeked = lib.peek_top(2);
        assert_eq!(peeked.len(), 2);
        assert_eq!(peeked[0].instance_id(), "card-1");
        assert_eq!(peeked[1].instance_id(), "card-2");
        // library unchanged
        assert_eq!(lib.count(), 3);
    }

    #[test]
    fn peek_top_returns_all_if_n_exceeds_size() {
        let lib = Library::from_cards(make_deck(1, "p1"));
        let peeked = lib.peek_top(10);
        assert_eq!(peeked.len(), 1);
    }

    #[test]
    fn peek_top_on_empty_library() {
        let lib = Library::empty();
        assert_eq!(lib.peek_top(3).len(), 0);
    }

    #[test]
    fn add_to_top() {
        let lib = Library::from_cards(make_deck(2, "p1"));
        let extra = make_creature("new-top", "p1");
        let lib2 = lib.add_to_top(extra);
        assert_eq!(lib2.peek_top(1)[0].instance_id(), "new-top");
        assert_eq!(lib2.count(), 3);
    }

    #[test]
    fn add_to_bottom() {
        let lib = Library::from_cards(make_deck(2, "p1"));
        let extra = make_creature("new-bottom", "p1");
        let lib2 = lib.add_to_bottom(extra);
        assert_eq!(lib2.count(), 3);
        let all = lib2.get_all();
        assert_eq!(all.last().unwrap().instance_id(), "new-bottom");
    }

    #[test]
    fn shuffle_with_seed_is_deterministic() {
        let cards = make_deck(20, "p1");
        let lib = Library::from_cards(cards);
        let s1 = lib.shuffle(Some(12345));
        let s2 = lib.shuffle(Some(12345));
        let ids1: Vec<&str> = s1.get_all().iter().map(|c| c.instance_id()).collect();
        let ids2: Vec<&str> = s2.get_all().iter().map(|c| c.instance_id()).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn shuffle_with_different_seeds_produces_different_orders() {
        let cards = make_deck(20, "p1");
        let lib = Library::from_cards(cards);
        let s1 = lib.shuffle(Some(12345));
        let s2 = lib.shuffle(Some(54321));
        let ids1: Vec<&str> = s1.get_all().iter().map(|c| c.instance_id()).collect();
        let ids2: Vec<&str> = s2.get_all().iter().map(|c| c.instance_id()).collect();
        assert_ne!(ids1, ids2);
    }

    #[test]
    fn shuffle_preserves_all_cards() {
        let cards = make_deck(20, "p1");
        let lib = Library::from_cards(cards.clone());
        let shuffled = lib.shuffle(Some(42));
        let original_ids: HashSet<&str> = cards.iter().map(|c| c.instance_id()).collect();
        let shuffled_ids: HashSet<&str> = shuffled.get_all().iter().map(|c| c.instance_id()).collect();
        assert_eq!(original_ids, shuffled_ids);
    }

    #[test]
    fn shuffle_empty_library() {
        let lib = Library::empty();
        let shuffled = lib.shuffle(Some(0));
        assert!(shuffled.is_empty());
    }

    #[test]
    fn shuffle_single_card() {
        let lib = Library::from_cards(make_deck(1, "p1"));
        let shuffled = lib.shuffle(None);
        assert_eq!(shuffled.count(), 1);
        assert_eq!(shuffled.peek_top(1)[0].instance_id(), "card-1");
    }

    #[test]
    fn shuffle_does_not_mutate_original() {
        let lib = Library::from_cards(make_deck(5, "p1"));
        let _shuffled = lib.shuffle(Some(1));
        // original top card still first
        assert_eq!(lib.peek_top(1)[0].instance_id(), "card-1");
    }

    #[test]
    fn draw_after_shuffle_reduces_count() {
        let lib = Library::from_cards(make_deck(3, "p1"));
        let shuffled = lib.shuffle(Some(42));
        let (card, remaining) = shuffled.draw_from_top();
        assert!(card.is_some());
        assert_eq!(remaining.count(), 2);
    }
}
