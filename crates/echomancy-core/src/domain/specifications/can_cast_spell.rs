//! CanCastSpell specification — checks if a player has at least one spell
//! that can legally be cast at the current timing.
//!
//! This delegates timing logic to the existing `spell_timing` service to
//! avoid duplicating rules.  The spec answers "does the player have *any*
//! castable spell?", which is what the action-window query needs.
//!
//! Note: priority is NOT checked here — use `HasPriority` separately.
//!
//! Mirrors the TypeScript `CanCastSpell` class from
//! `game/specifications/CanCastSpell.ts`.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::entities::the_stack::TheStack;
use crate::domain::enums::Step;
use crate::domain::errors::GameError;
use crate::domain::services::spell_timing;

/// Context required to evaluate the `CanCastSpell` specification.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct CanCastSpellCtx<'a> {
    /// All cards currently in the player's hand.
    pub(crate) hand_cards: &'a [CardInstance],
    /// The ID of the player attempting to cast a spell.
    pub(crate) player_id: &'a str,
    /// The ID of the player whose turn it currently is.
    pub(crate) current_player_id: &'a str,
    /// The current step.
    pub(crate) current_step: Step,
    /// Reference to the current stack.
    pub(crate) stack: &'a TheStack,
}

/// Returns `Ok(())` if the player has at least one spell that can legally
/// be cast at the current timing.
///
/// # Errors
///
/// Returns `GameError::InvalidCastSpellStep` if the player has no cards
/// in hand that can be cast right now (either because the hand is empty,
/// all cards are lands, or timing rules forbid casting any of them).
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn is_satisfied(ctx: &CanCastSpellCtx<'_>) -> Result<(), GameError> {
    let has_castable = ctx.hand_cards.iter().any(|card| {
        // Lands are not spells (CR 305.1).
        if card.definition().is_land() {
            return false;
        }

        spell_timing::can_cast_at_current_timing(
            card,
            ctx.player_id,
            ctx.current_player_id,
            ctx.current_step,
            ctx.stack,
        )
    });

    if has_castable {
        Ok(())
    } else {
        Err(GameError::InvalidCastSpellStep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cards::card_definition::CardDefinition;
    use crate::domain::cards::card_instance::CardInstance;
    use crate::domain::entities::the_stack::{SpellOnStack, StackItem, TheStack};
    use crate::domain::enums::{CardType, StaticAbility, Step};

    fn make_card(types: Vec<CardType>) -> CardInstance {
        let def = CardDefinition::new("test-id", "Test", types);
        CardInstance::new("inst-1", def, "p1")
    }

    fn make_card_with_flash(types: Vec<CardType>) -> CardInstance {
        let def = CardDefinition::new("test-id", "Test", types)
            .with_static_ability(StaticAbility::Flash);
        CardInstance::new("inst-flash", def, "p1")
    }

    fn make_land() -> CardInstance {
        make_card(vec![CardType::Land])
    }

    fn make_instant() -> CardInstance {
        make_card(vec![CardType::Instant])
    }

    fn make_sorcery() -> CardInstance {
        make_card(vec![CardType::Sorcery])
    }

    fn make_creature() -> CardInstance {
        make_card(vec![CardType::Creature])
    }

    fn stack_with_spell() -> TheStack {
        let spell_card = make_instant();
        TheStack::empty().push(StackItem::Spell(SpellOnStack {
            card: spell_card,
            controller_id: "p1".to_owned(),
            targets: Vec::new(),
        }))
    }

    // --- happy paths --------------------------------------------------------

    #[test]
    fn instant_in_hand_can_cast_any_time() {
        let instant = make_instant();
        let ctx = CanCastSpellCtx {
            hand_cards: &[instant],
            player_id: "p1",
            current_player_id: "p2", // opponent's turn
            current_step: Step::DeclareAttackers,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_ok());
    }

    #[test]
    fn sorcery_can_be_cast_in_first_main_empty_stack() {
        let sorcery = make_sorcery();
        let ctx = CanCastSpellCtx {
            hand_cards: &[sorcery],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::FirstMain,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_ok());
    }

    #[test]
    fn sorcery_can_be_cast_in_second_main_empty_stack() {
        let sorcery = make_sorcery();
        let ctx = CanCastSpellCtx {
            hand_cards: &[sorcery],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::SecondMain,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_ok());
    }

    #[test]
    fn flash_creature_can_be_cast_any_time() {
        let flash = make_card_with_flash(vec![CardType::Creature]);
        let ctx = CanCastSpellCtx {
            hand_cards: &[flash],
            player_id: "p1",
            current_player_id: "p2",
            current_step: Step::DeclareAttackers,
            stack: &stack_with_spell(),
        };
        assert!(is_satisfied(&ctx).is_ok());
    }

    // --- no castable spells -------------------------------------------------

    #[test]
    fn empty_hand_returns_err() {
        let ctx = CanCastSpellCtx {
            hand_cards: &[],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::FirstMain,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_err());
    }

    #[test]
    fn only_lands_in_hand_returns_err() {
        let land = make_land();
        let ctx = CanCastSpellCtx {
            hand_cards: &[land],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::FirstMain,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_err());
    }

    #[test]
    fn sorcery_on_opponents_turn_returns_err() {
        let sorcery = make_sorcery();
        let ctx = CanCastSpellCtx {
            hand_cards: &[sorcery],
            player_id: "p1",
            current_player_id: "p2", // p2's turn
            current_step: Step::FirstMain,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_err());
    }

    #[test]
    fn sorcery_during_combat_returns_err() {
        let sorcery = make_sorcery();
        let ctx = CanCastSpellCtx {
            hand_cards: &[sorcery],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::DeclareAttackers,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_err());
    }

    #[test]
    fn sorcery_with_non_empty_stack_returns_err() {
        let sorcery = make_sorcery();
        let ctx = CanCastSpellCtx {
            hand_cards: &[sorcery],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::FirstMain,
            stack: &stack_with_spell(),
        };
        assert!(is_satisfied(&ctx).is_err());
    }

    #[test]
    fn creature_without_flash_on_opponents_turn_returns_err() {
        let creature = make_creature();
        let ctx = CanCastSpellCtx {
            hand_cards: &[creature],
            player_id: "p1",
            current_player_id: "p2",
            current_step: Step::FirstMain,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_err());
    }

    #[test]
    fn error_variant_is_invalid_cast_spell_step() {
        let ctx = CanCastSpellCtx {
            hand_cards: &[],
            player_id: "p1",
            current_player_id: "p1",
            current_step: Step::FirstMain,
            stack: &TheStack::empty(),
        };
        let err = is_satisfied(&ctx).unwrap_err();
        assert!(matches!(err, GameError::InvalidCastSpellStep));
    }

    // --- mix of cards in hand -----------------------------------------------

    #[test]
    fn one_castable_card_among_unccastable_cards_returns_ok() {
        let land = make_land();
        let instant = make_instant();
        let sorcery = make_sorcery();
        // Sorcery cannot be cast (opponent's turn), but instant can.
        let ctx = CanCastSpellCtx {
            hand_cards: &[land, sorcery, instant],
            player_id: "p1",
            current_player_id: "p2",
            current_step: Step::DeclareAttackers,
            stack: &TheStack::empty(),
        };
        assert!(is_satisfied(&ctx).is_ok());
    }
}
