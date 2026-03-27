//! Battlefield zone entity.
//!
//! The battlefield holds all permanents currently in play.
//! Ownership semantics mirror the TS immutable-class pattern:
//! every mutating method returns a new `Battlefield`.
//!
//! Mirrors the TypeScript `Battlefield` class from
//! `game/entities/Battlefield.ts`.

use crate::domain::cards::card_instance::CardInstance;

/// Represents the battlefield zone — all permanents currently in play.
///
/// `Battlefield` owns its cards and provides query/mutation methods.
/// Mutation always produces a new `Battlefield` (value-object style).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Battlefield {
    cards: Vec<CardInstance>,
}

impl Battlefield {
    /// Create an empty battlefield.
    pub fn empty() -> Self {
        Battlefield { cards: Vec::new() }
    }

    /// Create a battlefield from an existing collection of card instances.
    pub fn from_cards(cards: Vec<CardInstance>) -> Self {
        Battlefield { cards }
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Returns `true` if no permanents are on the battlefield.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Number of permanents on the battlefield.
    pub fn count(&self) -> usize {
        self.cards.len()
    }

    /// Returns all permanents in insertion order.
    ///
    /// The returned slice is read-only; call `from_cards` to construct a new
    /// battlefield from a modified collection.
    pub fn get_all(&self) -> &[CardInstance] {
        &self.cards
    }

    /// Find a permanent by its instance ID.
    pub fn find_permanent(&self, instance_id: &str) -> Option<&CardInstance> {
        self.cards.iter().find(|c| c.instance_id() == instance_id)
    }

    /// Find all permanents owned by the given player.
    pub fn find_permanents_by_owner(&self, owner_id: &str) -> Vec<&CardInstance> {
        self.cards
            .iter()
            .filter(|c| c.owner_id() == owner_id)
            .collect()
    }

    // -------------------------------------------------------------------------
    // Mutations (return new Battlefield)
    // -------------------------------------------------------------------------

    /// Add a permanent and return a new `Battlefield`.
    pub fn add_permanent(&self, permanent: CardInstance) -> Battlefield {
        let mut cards = Vec::with_capacity(self.cards.len() + 1);
        cards.extend_from_slice(&self.cards);
        cards.push(permanent);
        Battlefield { cards }
    }

    /// Remove a permanent by instance ID and return a new `Battlefield`.
    ///
    /// If the permanent does not exist the returned battlefield still contains
    /// all existing permanents (no error, same behaviour as the TS version).
    pub fn remove_permanent(&self, instance_id: &str) -> Battlefield {
        let cards = self
            .cards
            .iter()
            .filter(|c| c.instance_id() != instance_id)
            .cloned()
            .collect();
        Battlefield { cards }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_instance::test_helpers::{make_creature, make_land};

    #[test]
    fn empty_battlefield() {
        let bf = Battlefield::empty();
        assert!(bf.is_empty());
        assert_eq!(bf.count(), 0);
        assert!(bf.get_all().is_empty());
    }

    #[test]
    fn from_cards_initialises_correctly() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone(), c2.clone()]);
        assert_eq!(bf.count(), 2);
        assert!(!bf.is_empty());
    }

    #[test]
    fn add_permanent_to_empty_battlefield() {
        let bf = Battlefield::empty();
        let creature = make_creature("c1", "p1");
        let bf2 = bf.add_permanent(creature.clone());
        assert_eq!(bf2.count(), 1);
        assert_eq!(
            bf2.find_permanent("c1").map(|c| c.instance_id()),
            Some("c1")
        );
    }

    #[test]
    fn add_permanent_does_not_mutate_original() {
        let bf = Battlefield::empty();
        let creature = make_creature("c1", "p1");
        let _bf2 = bf.add_permanent(creature);
        // Original unchanged
        assert_eq!(bf.count(), 0);
    }

    #[test]
    fn add_multiple_permanents() {
        let bf = Battlefield::empty();
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let bf2 = bf.add_permanent(c1.clone()).add_permanent(c2.clone());
        assert_eq!(bf2.count(), 2);
    }

    #[test]
    fn remove_permanent_by_id() {
        let c1 = make_creature("c1", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone()]);
        let bf2 = bf.remove_permanent("c1");
        assert!(bf2.is_empty());
        assert!(bf2.find_permanent("c1").is_none());
    }

    #[test]
    fn remove_does_not_mutate_original() {
        let c1 = make_creature("c1", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone()]);
        let _bf2 = bf.remove_permanent("c1");
        assert_eq!(bf.count(), 1);
    }

    #[test]
    fn remove_correct_card_when_multiple_exist() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let c3 = make_creature("c3", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone(), c2.clone(), c3.clone()]);
        let bf2 = bf.remove_permanent("c2");
        assert_eq!(bf2.count(), 2);
        assert!(bf2.find_permanent("c1").is_some());
        assert!(bf2.find_permanent("c2").is_none());
        assert!(bf2.find_permanent("c3").is_some());
    }

    #[test]
    fn remove_non_existent_card_keeps_all() {
        let c1 = make_creature("c1", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone()]);
        let bf2 = bf.remove_permanent("non-existent");
        assert_eq!(bf2.count(), 1);
    }

    #[test]
    fn find_permanent_returns_correct_card() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone(), c2.clone()]);
        assert_eq!(
            bf.find_permanent("c2").map(|c| c.instance_id()),
            Some("c2")
        );
    }

    #[test]
    fn find_permanent_returns_none_for_missing() {
        let bf = Battlefield::empty();
        assert!(bf.find_permanent("missing").is_none());
    }

    #[test]
    fn find_permanents_by_owner() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_creature("c2", "p2");
        let c3 = make_creature("c3", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone(), c2.clone(), c3.clone()]);
        let p1_permanents = bf.find_permanents_by_owner("p1");
        assert_eq!(p1_permanents.len(), 2);
        let ids: Vec<&str> = p1_permanents.iter().map(|c| c.instance_id()).collect();
        assert!(ids.contains(&"c1"));
        assert!(ids.contains(&"c3"));
    }

    #[test]
    fn find_permanents_by_owner_empty_for_unknown_player() {
        let c1 = make_creature("c1", "p1");
        let bf = Battlefield::from_cards(vec![c1]);
        assert!(bf.find_permanents_by_owner("p99").is_empty());
    }

    #[test]
    fn find_permanents_by_owner_on_empty_battlefield() {
        let bf = Battlefield::empty();
        assert!(bf.find_permanents_by_owner("p1").is_empty());
    }

    #[test]
    fn get_all_returns_cards_in_order() {
        let c1 = make_creature("c1", "p1");
        let c2 = make_land("c2", "p1");
        let c3 = make_creature("c3", "p1");
        let bf = Battlefield::from_cards(vec![c1.clone(), c2.clone(), c3.clone()]);
        let all = bf.get_all();
        assert_eq!(all[0].instance_id(), "c1");
        assert_eq!(all[1].instance_id(), "c2");
        assert_eq!(all[2].instance_id(), "c3");
    }
}
