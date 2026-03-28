//! Trait implementations for the `Game` aggregate.
//!
//! Contains `ExportableGameContext` and `CombatValidationContext` impls.
//! Kept in a separate file because these are large blocks of boilerplate that
//! adapt domain state to the interfaces expected by other layers.

use crate::domain::cards::card_instance::CardInstance;
use crate::domain::entities::the_stack::StackItem;
use crate::domain::enums::{GameLifecycleState, StaticAbility, Step};
use crate::domain::services::combat_declarations::CombatValidationContext;
use crate::domain::services::game_state_export::{
    DrawOutcomeExport, ExportableGameContext, GameOutcomeExport, StackItemExport, StackItemKind,
    WinOutcomeExport,
};
use crate::domain::types::PlayerId;
use crate::domain::value_objects::mana::ManaPool;
use crate::domain::value_objects::permanent_state::PermanentState;

use super::{Game, GameOutcome};

impl ExportableGameContext for Game {
    fn game_id(&self) -> &str {
        &self.id
    }

    fn lifecycle_state(&self) -> GameLifecycleState {
        self.lifecycle
    }

    fn game_outcome(&self) -> Option<GameOutcomeExport> {
        self.outcome.as_ref().map(|o| match o {
            GameOutcome::Win { winner_id, reason } => {
                GameOutcomeExport::Win(WinOutcomeExport {
                    winner_id: winner_id.to_string(),
                    reason: format!("{reason:?}"),
                })
            }
            GameOutcome::Draw { reason } => {
                GameOutcomeExport::Draw(DrawOutcomeExport {
                    reason: format!("{reason:?}"),
                })
            }
        })
    }

    fn current_turn_number(&self) -> u32 {
        self.turn_state.turn_number()
    }

    fn current_player_id(&self) -> &str {
        self.turn_state.current_player_id().as_str()
    }

    fn current_step(&self) -> Step {
        self.turn_state.current_step()
    }

    fn priority_player_id(&self) -> Option<&str> {
        self.priority_player_id.as_ref().map(PlayerId::as_str)
    }

    fn turn_order(&self) -> &[String] {
        &self.turn_order_ids
    }

    fn player_life_total(&self, player_id: &str) -> i32 {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.life_total)
            .unwrap_or(0)
    }

    fn played_lands_this_turn(&self, _player_id: &str) -> u32 {
        self.turn_state.played_lands()
    }

    fn player_mana_pool(&self, player_id: &str) -> &ManaPool {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| &p.mana_pool)
            .unwrap_or_else(|| {
                // Static empty pool for missing players
                static EMPTY: std::sync::OnceLock<ManaPool> = std::sync::OnceLock::new();
                EMPTY.get_or_init(ManaPool::empty)
            })
    }

    fn hand_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.hand.as_slice())
            .unwrap_or(&[])
    }

    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.battlefield.as_slice())
            .unwrap_or(&[])
    }

    fn graveyard_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.graveyard.as_slice())
            .unwrap_or(&[])
    }

    fn library_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.library.as_slice())
            .unwrap_or(&[])
    }

    fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
        self.permanent_states.get(instance_id)
    }

    fn stack_items(&self) -> Vec<StackItemExport> {
        self.stack
            .iter()
            .map(|item| match item {
                StackItem::Spell(spell) => StackItemExport {
                    kind: StackItemKind::Spell,
                    source_card_instance_id: spell.card.instance_id().to_owned(),
                    source_card_definition_id: spell.card.definition().id().to_owned(),
                    controller_id: spell.controller_id.clone(),
                    targets: Vec::new(),
                },
                StackItem::Ability(ability) => StackItemExport {
                    kind: StackItemKind::ActivatedAbility,
                    source_card_instance_id: ability.source_id.clone(),
                    source_card_definition_id: String::new(),
                    controller_id: ability.controller_id.clone(),
                    targets: Vec::new(),
                },
            })
            .collect()
    }
}

// ============================================================================
// CombatValidationContext implementation
// ============================================================================

impl CombatValidationContext for Game {
    fn current_step(&self) -> Step {
        self.turn_state.current_step()
    }

    fn current_player_id(&self) -> &str {
        self.turn_state.current_player_id().as_str()
    }

    fn opponent_of(&self, player_id: &str) -> &str {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() != player_id)
            .map(|p| p.player_id.as_str())
            .unwrap_or("")
    }

    fn battlefield_cards(&self, player_id: &str) -> &[CardInstance] {
        self.players
            .iter()
            .find(|p| p.player_id.as_str() == player_id)
            .map(|p| p.battlefield.as_slice())
            .unwrap_or(&[])
    }

    fn is_creature(&self, card: &CardInstance) -> bool {
        card.definition().is_creature()
    }

    fn has_static_ability(&self, card: &CardInstance, ability: StaticAbility) -> bool {
        card.definition().has_static_ability(ability)
    }

    fn permanent_state(&self, instance_id: &str) -> Option<&PermanentState> {
        self.permanent_states.get(instance_id)
    }
}
