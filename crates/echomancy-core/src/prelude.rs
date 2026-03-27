//! Convenience re-exports for common types used across the crate and by consumers.

pub use crate::domain::actions::Action;
pub use crate::domain::enums::{
    CardType, GameLifecycleState, GraveyardReason, ManaColor, StaticAbility, Step, ZoneName,
};
pub use crate::domain::errors::GameError;
pub use crate::domain::events::{CardInstanceSnapshot, GameEvent};
pub use crate::domain::types::{CardDefinitionId, CardInstanceId, PlayerId};
