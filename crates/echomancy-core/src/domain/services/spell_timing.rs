//! SpellTiming — determine when a spell can legally be cast.
//!
//! Stateless service that checks timing rules without mutating game state.
//! Mirrors `SpellTiming.ts`.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::entities::the_stack::TheStack;
use crate::domain::enums::{CardType, StaticAbility, Step};

/// Returns `true` if the card must follow sorcery-speed timing.
///
/// Sorcery speed applies to everything that is NOT an instant and does NOT
/// have the Flash keyword:
/// - Sorceries
/// - Creatures (unless Flash)
/// - Artifacts (unless Flash)
/// - Enchantments (unless Flash)
/// - Planeswalkers (unless Flash)
#[allow(dead_code)]
pub(crate) fn is_sorcery_speed(card: &CardInstance) -> bool {
    !is_instant_speed(card)
}

/// Returns `true` if the card can be cast at instant speed.
///
/// - Instants are always instant speed.
/// - Any permanent with the Flash keyword is instant speed.
pub(crate) fn is_instant_speed(card: &CardInstance) -> bool {
    let def = card.definition();

    if def.types().contains(&CardType::Instant) {
        return true;
    }

    if def.has_static_ability(StaticAbility::Flash) {
        return true;
    }

    false
}

/// Returns `true` if the card can be cast at the current timing.
///
/// Lands are not spells (CR 305.1) and are never castable via this function;
/// they are played through a separate action. This function returns `false`
/// immediately for any card of type `Land`.
///
/// - Instant-speed cards can always be cast (caller is responsible for
///   priority checks at higher levels).
/// - Sorcery-speed cards require:
///   1. `casting_player_id == current_player_id` (active player's turn)
///   2. Current step is `FirstMain` or `SecondMain`
///   3. The stack is empty
pub(crate) fn can_cast_at_current_timing(
    card: &CardInstance,
    casting_player_id: &str,
    current_player_id: &str,
    current_step: Step,
    stack: &TheStack,
) -> bool {
    // CR 305.1: Lands are not spells. They are never cast; they are played
    // as a special action and must not pass through spell-timing checks.
    if card.definition().types().contains(&CardType::Land) {
        return false;
    }

    if is_instant_speed(card) {
        return true;
    }

    // Sorcery-speed checks:

    // 1. Must be the active player.
    if casting_player_id != current_player_id {
        return false;
    }

    // 2. Must be a main phase.
    let is_main_phase = current_step == Step::FirstMain || current_step == Step::SecondMain;
    if !is_main_phase {
        return false;
    }

    // 3. Stack must be empty.
    if stack.has_items() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::entities::the_stack::{SpellOnStack, StackItem, TheStack};
    use crate::domain::enums::{CardType, StaticAbility, Step};

    fn make_card(types: Vec<CardType>) -> CardInstance {
        let def = CardDefinition::new("test", "Test", types);
        CardInstance::new("test-id", def, "p1")
    }

    fn make_card_with_flash(types: Vec<CardType>) -> CardInstance {
        let def = CardDefinition::new("test", "Test", types)
            .with_static_ability(StaticAbility::Flash);
        CardInstance::new("test-id", def, "p1")
    }

    fn stack_with_spell() -> TheStack {
        let spell = make_card(vec![CardType::Instant]);
        TheStack::empty().push(StackItem::Spell(SpellOnStack {
            card: spell,
            controller_id: "p1".to_owned(),
            targets: Vec::new(),
        }))
    }

    // ---- is_instant_speed --------------------------------------------------

    // ---- land guard --------------------------------------------------------

    /// CR 305.1: Lands are not spells and must never pass timing checks.
    #[test]
    fn land_cannot_be_cast_at_any_timing() {
        let card = make_card(vec![CardType::Land]);
        // Even during active player's main phase with empty stack.
        assert!(!can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::FirstMain,
            &TheStack::empty()
        ));
    }

    #[test]
    fn land_cannot_be_cast_at_instant_speed() {
        // A land with Flash would still not be castable (lands are not spells).
        let card = make_card_with_flash(vec![CardType::Land]);
        assert!(!can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::FirstMain,
            &TheStack::empty()
        ));
    }

    #[test]
    fn instant_is_instant_speed() {
        let card = make_card(vec![CardType::Instant]);
        assert!(is_instant_speed(&card));
        assert!(!is_sorcery_speed(&card));
    }

    #[test]
    fn sorcery_is_sorcery_speed() {
        let card = make_card(vec![CardType::Sorcery]);
        assert!(is_sorcery_speed(&card));
        assert!(!is_instant_speed(&card));
    }

    #[test]
    fn creature_is_sorcery_speed() {
        let card = make_card(vec![CardType::Creature]);
        assert!(is_sorcery_speed(&card));
        assert!(!is_instant_speed(&card));
    }

    #[test]
    fn artifact_is_sorcery_speed() {
        let card = make_card(vec![CardType::Artifact]);
        assert!(is_sorcery_speed(&card));
    }

    #[test]
    fn enchantment_is_sorcery_speed() {
        let card = make_card(vec![CardType::Enchantment]);
        assert!(is_sorcery_speed(&card));
    }

    #[test]
    fn planeswalker_is_sorcery_speed() {
        let card = make_card(vec![CardType::Planeswalker]);
        assert!(is_sorcery_speed(&card));
    }

    #[test]
    fn creature_with_flash_is_instant_speed() {
        let card = make_card_with_flash(vec![CardType::Creature]);
        assert!(is_instant_speed(&card));
        assert!(!is_sorcery_speed(&card));
    }

    #[test]
    fn artifact_with_flash_is_instant_speed() {
        let card = make_card_with_flash(vec![CardType::Artifact]);
        assert!(is_instant_speed(&card));
    }

    // ---- can_cast_at_current_timing: instant speed -------------------------

    #[test]
    fn instant_can_be_cast_any_time() {
        let card = make_card(vec![CardType::Instant]);
        // Even during opponent's turn, non-main phase, non-empty stack
        let stack = stack_with_spell();
        assert!(can_cast_at_current_timing(
            &card,
            "p1",
            "p2", // opponent's turn
            Step::DeclareAttackers,
            &stack
        ));
    }

    // ---- can_cast_at_current_timing: sorcery speed -------------------------

    #[test]
    fn sorcery_can_be_cast_during_first_main_empty_stack() {
        let card = make_card(vec![CardType::Sorcery]);
        assert!(can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::FirstMain,
            &TheStack::empty()
        ));
    }

    #[test]
    fn sorcery_can_be_cast_during_second_main_empty_stack() {
        let card = make_card(vec![CardType::Sorcery]);
        assert!(can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::SecondMain,
            &TheStack::empty()
        ));
    }

    #[test]
    fn sorcery_cannot_be_cast_on_opponents_turn() {
        let card = make_card(vec![CardType::Sorcery]);
        assert!(!can_cast_at_current_timing(
            &card,
            "p1",
            "p2", // p2 is active player
            Step::FirstMain,
            &TheStack::empty()
        ));
    }

    #[test]
    fn sorcery_cannot_be_cast_during_combat() {
        let card = make_card(vec![CardType::Sorcery]);
        assert!(!can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::DeclareAttackers,
            &TheStack::empty()
        ));
    }

    #[test]
    fn sorcery_cannot_be_cast_when_stack_not_empty() {
        let card = make_card(vec![CardType::Sorcery]);
        let stack = stack_with_spell();
        assert!(!can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::FirstMain,
            &stack
        ));
    }

    #[test]
    fn creature_with_flash_can_be_cast_anytime() {
        let card = make_card_with_flash(vec![CardType::Creature]);
        let stack = stack_with_spell();
        assert!(can_cast_at_current_timing(
            &card,
            "p1",
            "p2",
            Step::DeclareAttackers,
            &stack
        ));
    }

    #[test]
    fn creature_without_flash_follows_sorcery_timing() {
        let card = make_card(vec![CardType::Creature]);
        assert!(!can_cast_at_current_timing(
            &card,
            "p1",
            "p1",
            Step::DeclareAttackers,
            &TheStack::empty()
        ));
    }
}
