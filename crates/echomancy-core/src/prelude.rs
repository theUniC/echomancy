//! Convenience re-exports for common types used across the crate and by consumers.

pub use crate::domain::actions::Action;
pub use crate::domain::enums::{
    CardType, GameLifecycleState, GraveyardReason, ManaColor, StaticAbility, Step, ZoneName,
};
pub use crate::domain::errors::GameError;
pub use crate::domain::events::{CardInstanceSnapshot, GameEvent};
pub use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
pub use crate::domain::value_objects::mana::{
    InsufficientManaError, ManaCost, ManaAddError, ManaPool, ManaPoolSnapshot, ManaSpendError,
};
pub use crate::domain::value_objects::permanent_state::{
    CreatureSubState, PermanentState, PermanentStateError, PermanentStateSnapshot,
};
pub use crate::domain::value_objects::turn_state::{TurnState, TurnStateSnapshot};
