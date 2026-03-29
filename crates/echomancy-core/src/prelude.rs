//! Convenience re-exports for common types used across the crate and by consumers.

// Infrastructure layer
pub use crate::infrastructure::allowed_actions::AllowedActionsResult;
pub use crate::infrastructure::legal_actions::compute_legal_actions;
pub use crate::infrastructure::create_rules_engine;
pub use crate::infrastructure::game_snapshot::{
    CardRegistry, CardSnapshot, CombatStateSnapshot, CombatSummary, GameSnapshot,
    OpponentState, PrivatePlayerState, PublicGameState, SnapshotError, StackItemSnapshot,
    StackSnapshot, UiHints, create_game_snapshot,
};

// Domain — game aggregate
pub use crate::domain::game::Game;

// Domain — export types
pub use crate::infrastructure::game_state_export::{
    CardInstanceExport, CreatureStateExport, DrawOutcomeExport, GameOutcomeExport,
    GameStateExport, ManaPoolExport, PlayerStateExport, StackItemExport, StackItemKind,
    WinOutcomeExport,
};

pub use crate::domain::abilities::{ActivatedAbility, ActivationCost, Ability};
pub use crate::domain::actions::Action;
pub use crate::domain::services::mana_payment::can_pay_cost;
pub use crate::domain::cards::card_definition::CardDefinition;
pub use crate::domain::cards::card_instance::CardInstance;
pub use crate::domain::cards::{catalog, prebuilt_decks};
pub use crate::domain::costs::{Cost, CostContext};
pub use crate::domain::effects::{Effect, EffectContext};
// NOTE: Battlefield, Graveyard, Hand, Library, Player, PlayerState are not
// exported here — they are not used by the Game aggregate (which uses
// GamePlayerState with Vec<CardInstance> directly). Kept in their modules for
// potential future direct-zone APIs.
pub use crate::domain::entities::the_stack::{AbilityOnStack, SpellOnStack, StackItem, TheStack};
pub use crate::domain::enums::{
    CardType, GameLifecycleState, GraveyardReason, ManaColor, StaticAbility, Step, ZoneName,
};
pub use crate::domain::errors::GameError;
pub use crate::domain::events::{CardInstanceSnapshot, GameEvent};
pub use crate::domain::targets::Target;
pub use crate::domain::triggers::{Trigger, TriggerCondition, TriggerEventType};
pub use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
pub use crate::domain::value_objects::mana::{
    InsufficientManaError, ManaCost, ManaAddError, ManaPool, ManaPoolSnapshot, ManaSpendError,
};
pub use crate::domain::value_objects::creature_state::CreatureSubState;
pub use crate::domain::value_objects::permanent_state::{
    PermanentState, PermanentStateError, PermanentStateSnapshot,
};
pub use crate::domain::value_objects::turn_state::{TurnState, TurnStateSnapshot};
