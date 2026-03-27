//! Convenience re-exports for common types used across the crate and by consumers.

pub use crate::domain::abilities::{ActivatedAbility, ActivationCost, Ability};
pub use crate::domain::actions::Action;
pub use crate::domain::cards::card_definition::CardDefinition;
pub use crate::domain::cards::card_instance::CardInstance;
pub use crate::domain::cards::{catalog, prebuilt_decks};
pub use crate::domain::costs::{Cost, CostContext};
pub use crate::domain::effects::{Effect, EffectContext};
pub use crate::domain::entities::battlefield::Battlefield;
pub use crate::domain::entities::graveyard::Graveyard;
pub use crate::domain::entities::hand::Hand;
pub use crate::domain::entities::library::Library;
pub use crate::domain::entities::player::{Player, PlayerState};
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
pub use crate::domain::value_objects::permanent_state::{
    CreatureSubState, PermanentState, PermanentStateError, PermanentStateSnapshot,
};
pub use crate::domain::value_objects::turn_state::{TurnState, TurnStateSnapshot};
