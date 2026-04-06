// NOTE: Not currently used by the Game aggregate. Kept for potential future direct-zone APIs.

//! Graveyard zone entity.
//!
//! The graveyard is ordered — cards are stored bottom-to-top.
//! The most recently added card is at the end of the internal `Vec`.

use crate::domain::cards::card_instance::CardInstance;

/// A player's graveyard zone.
///
/// Ordered from oldest (index 0) to most recent (last element).
/// Mutation always produces a new `Graveyard` (value-object style).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Graveyard {
    /// Bottom-to-top order: index 0 is oldest, last element is newest.
    cards: Vec<CardInstance>,
}

impl Graveyard {
    /// Create an empty graveyard.
    pub fn empty() -> Self {
        Graveyard { cards: Vec::new() }
    }

    /// Create a graveyard from an ordered list of cards (bottom-to-top).
    pub fn from_cards(cards: Vec<CardInstance>) -> Self {
        Graveyard { cards }
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Returns `true` if the graveyard is empty.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Number of cards in the graveyard.
    pub fn count(&self) -> usize {
        self.cards.len()
    }

    /// Returns all cards in bottom-to-top order (oldest first).
    pub fn get_all(&self) -> &[CardInstance] {
        &self.cards
    }

    /// Returns the top card (most recently added), or `None` if empty.
    pub fn get_top_card(&self) -> Option<&CardInstance> {
        self.cards.last()
    }

    // -------------------------------------------------------------------------
    // Mutations (return new Graveyard)
    // -------------------------------------------------------------------------

    /// Add a card to the top of the graveyard and return a new `Graveyard`.
    pub fn add_card(&self, card: CardInstance) -> Graveyard {
        let mut cards = Vec::with_capacity(self.cards.len() + 1);
        cards.extend_from_slice(&self.cards);
        cards.push(card);
        Graveyard { cards }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_instance::test_helpers::make_creature;

    #[test]
    fn empty_graveyard() {
        let g = Graveyard::empty();
        assert!(g.is_empty());
        assert_eq!(g.count(), 0);
        assert!(g.get_top_card().is_none());
    }

    #[test]
    fn from_cards() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let g = Graveyard::from_cards(vec![c1, c2]);
        assert_eq!(g.count(), 2);
    }

    #[test]
    fn add_card_to_graveyard() {
        let g = Graveyard::empty();
        let c = make_creature("c1", "p1");
        let g2 = g.add_card(c.clone());
        assert_eq!(g2.count(), 1);
        // original unchanged
        assert_eq!(g.count(), 0);
    }

    #[test]
    fn get_top_card_returns_most_recent() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let g = Graveyard::empty().add_card(c1).add_card(c2);
        assert_eq!(g.get_top_card().map(|c| c.instance_id()), Some("c2"));
    }

    #[test]
    fn get_all_returns_bottom_to_top_order() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let g = Graveyard::from_cards(vec![c1, c2]);
        let all = g.get_all();
        assert_eq!(all[0].instance_id(), "c1");
        assert_eq!(all[1].instance_id(), "c2");
    }

    #[test]
    fn add_card_does_not_mutate_original() {
        let c = make_creature("c1", "p1");
        let g = Graveyard::from_cards(vec![c.clone()]);
        let _g2 = g.add_card(make_creature("c2", "p1"));
        assert_eq!(g.count(), 1);
    }
}
