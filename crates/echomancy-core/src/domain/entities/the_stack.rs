//! The stack zone — spells and abilities waiting to resolve.
//!
//! The stack follows LIFO order: the most recently added item resolves first.
//! Named `TheStack` to distinguish it from Rust's standard data structures.
//!
//! Mirrors the TypeScript `TheStack` class and `StackTypes.ts` from
//! `game/entities/TheStack.ts` and `game/StackTypes.ts`.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::effects::Effect;
use crate::domain::targets::Target;

/// A spell on the stack — a card waiting to resolve.
///
/// Mirrors the TypeScript `SpellOnStack` type from `StackTypes.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellOnStack {
    /// The card being cast.
    pub card: CardInstance,
    /// The player who cast the spell.
    pub controller_id: String,
    /// Target players or permanents (MVP: usually empty).
    pub targets: Vec<Target>,
}

/// An activated ability on the stack.
///
/// Mirrors the TypeScript `AbilityOnStack` type from `StackTypes.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbilityOnStack {
    /// The permanent whose ability was activated.
    pub source_id: String,
    /// The effect to execute when the ability resolves.
    pub effect: Effect,
    /// The player who activated the ability.
    pub controller_id: String,
    /// Target players or permanents (MVP: usually empty).
    pub targets: Vec<Target>,
}

/// Items that can be on the stack.
///
/// MVP scope: only spells and activated abilities.
/// `TriggeredAbilityOnStack` is defined in the TS source but triggers execute
/// immediately in the MVP — so it is omitted here for now.
///
/// Mirrors the TypeScript `StackItem` union from `StackTypes.ts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackItem {
    /// A spell (card being cast).
    Spell(SpellOnStack),
    /// An activated ability from a permanent.
    Ability(AbilityOnStack),
}

impl StackItem {
    /// Returns the controller ID of this stack item.
    pub fn controller_id(&self) -> &str {
        match self {
            StackItem::Spell(s) => &s.controller_id,
            StackItem::Ability(a) => &a.controller_id,
        }
    }
}

/// The game's stack zone (LIFO).
///
/// Items are stored bottom-to-top: the last element is the top of the stack.
/// Mutation always produces a new `TheStack` (value-object style).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheStack {
    /// Bottom-to-top order: the last element is the top of the stack.
    items: Vec<StackItem>,
}

impl TheStack {
    /// Create an empty stack.
    pub fn empty() -> Self {
        TheStack { items: Vec::new() }
    }

    /// Create a stack from an ordered list of items (bottom-to-top).
    pub fn from_items(items: Vec<StackItem>) -> Self {
        TheStack { items }
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    /// Returns `true` if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns `true` if the stack has at least one item.
    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }

    /// Number of items on the stack.
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Returns all items in bottom-to-top order (read-only).
    pub fn get_all(&self) -> &[StackItem] {
        &self.items
    }

    /// Peek at the top item without removing it.
    pub fn peek(&self) -> Option<&StackItem> {
        self.items.last()
    }

    // -------------------------------------------------------------------------
    // Mutations (return new TheStack)
    // -------------------------------------------------------------------------

    /// Push an item onto the top of the stack.
    pub fn push(&self, item: StackItem) -> TheStack {
        let mut items = Vec::with_capacity(self.items.len() + 1);
        items.extend_from_slice(&self.items);
        items.push(item);
        TheStack { items }
    }

    /// Pop the top item and return it alongside the new stack.
    ///
    /// Returns `(None, empty_stack)` if the stack is already empty.
    pub fn pop(&self) -> (Option<StackItem>, TheStack) {
        if self.items.is_empty() {
            return (None, TheStack::empty());
        }
        let mut items = self.items.clone();
        let item = items.pop();
        (item, TheStack { items })
    }

    /// Clear all items and return an empty `TheStack`.
    pub fn clear(&self) -> TheStack {
        TheStack::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_instance::test_helpers::make_creature;

    fn make_spell(controller_id: &str) -> StackItem {
        StackItem::Spell(SpellOnStack {
            card: make_creature(&format!("card-{controller_id}"), controller_id),
            controller_id: controller_id.to_owned(),
            targets: Vec::new(),
        })
    }

    fn make_ability(controller_id: &str) -> StackItem {
        StackItem::Ability(AbilityOnStack {
            source_id: format!("source-{controller_id}"),
            effect: Effect::NoOp,
            controller_id: controller_id.to_owned(),
            targets: Vec::new(),
        })
    }

    #[test]
    fn empty_stack() {
        let stack = TheStack::empty();
        assert!(stack.is_empty());
        assert!(!stack.has_items());
        assert_eq!(stack.count(), 0);
        assert!(stack.get_all().is_empty());
    }

    #[test]
    fn from_items() {
        let s1 = make_spell("p1");
        let s2 = make_spell("p1");
        let stack = TheStack::from_items(vec![s1, s2]);
        assert_eq!(stack.count(), 2);
        assert!(!stack.is_empty());
    }

    #[test]
    fn push_adds_item_to_top() {
        let stack = TheStack::empty();
        let spell = make_spell("p1");
        let stack2 = stack.push(spell.clone());
        assert_eq!(stack2.count(), 1);
        assert_eq!(stack2.peek(), Some(&spell));
    }

    #[test]
    fn push_does_not_mutate_original() {
        let stack = TheStack::empty();
        let _stack2 = stack.push(make_spell("p1"));
        assert!(stack.is_empty());
    }

    #[test]
    fn push_is_lifo() {
        let s1 = make_spell("p1");
        let s2 = make_spell("p2");
        let stack = TheStack::empty().push(s1).push(s2.clone());
        assert_eq!(stack.peek(), Some(&s2));
    }

    #[test]
    fn pop_removes_top_item() {
        let s1 = make_spell("p1");
        let s2 = make_spell("p2");
        let stack = TheStack::from_items(vec![s1.clone(), s2.clone()]);
        let (item, remaining) = stack.pop();
        assert_eq!(item, Some(s2));
        assert_eq!(remaining.count(), 1);
        assert_eq!(remaining.peek(), Some(&s1));
    }

    #[test]
    fn pop_from_empty_returns_none() {
        let stack = TheStack::empty();
        let (item, remaining) = stack.pop();
        assert!(item.is_none());
        assert!(remaining.is_empty());
    }

    #[test]
    fn pop_does_not_mutate_original() {
        let stack = TheStack::from_items(vec![make_spell("p1")]);
        let _result = stack.pop();
        assert_eq!(stack.count(), 1);
    }

    #[test]
    fn peek_on_empty_returns_none() {
        let stack = TheStack::empty();
        assert!(stack.peek().is_none());
    }

    #[test]
    fn peek_does_not_remove_item() {
        let spell = make_spell("p1");
        let stack = TheStack::from_items(vec![spell.clone()]);
        let _ = stack.peek();
        assert_eq!(stack.count(), 1);
    }

    #[test]
    fn get_all_returns_bottom_to_top() {
        let s1 = make_spell("p1");
        let s2 = make_spell("p2");
        let stack = TheStack::from_items(vec![s1.clone(), s2.clone()]);
        let all = stack.get_all();
        // Bottom = first element, top = last
        assert_eq!(all[1], s2);
    }

    #[test]
    fn clear_returns_empty_stack() {
        let stack = TheStack::from_items(vec![make_spell("p1"), make_ability("p1")]);
        let cleared = stack.clear();
        assert!(cleared.is_empty());
        // original unchanged
        assert_eq!(stack.count(), 2);
    }

    #[test]
    fn mixed_spell_and_ability() {
        let spell = make_spell("p1");
        let ability = make_ability("p2");
        let stack = TheStack::empty().push(spell).push(ability.clone());
        let (top, _) = stack.pop();
        assert_eq!(top, Some(ability));
    }

    #[test]
    fn controller_id_from_spell() {
        let spell = make_spell("player-1");
        assert_eq!(spell.controller_id(), "player-1");
    }

    #[test]
    fn controller_id_from_ability() {
        let ability = make_ability("player-2");
        assert_eq!(ability.controller_id(), "player-2");
    }
}
