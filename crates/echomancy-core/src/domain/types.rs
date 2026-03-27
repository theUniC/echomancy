use serde::{Deserialize, Serialize};
use std::fmt;

/// Newtype wrapper for a player's unique identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(String);

impl PlayerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PlayerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PlayerId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Newtype wrapper for a card instance's unique identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardInstanceId(String);

impl CardInstanceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CardInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CardInstanceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CardInstanceId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Newtype wrapper for a card definition's unique identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardDefinitionId(String);

impl CardDefinitionId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CardDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CardDefinitionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CardDefinitionId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_display() {
        let id = PlayerId::new("player-1");
        assert_eq!(id.to_string(), "player-1");
    }

    #[test]
    fn card_instance_id_display() {
        let id = CardInstanceId::new("instance-abc");
        assert_eq!(id.to_string(), "instance-abc");
    }

    #[test]
    fn card_definition_id_display() {
        let id = CardDefinitionId::new("lightning-bolt");
        assert_eq!(id.to_string(), "lightning-bolt");
    }

    #[test]
    fn player_id_equality() {
        let a = PlayerId::new("x");
        let b = PlayerId::new("x");
        let c = PlayerId::new("y");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn player_id_from_str() {
        let id: PlayerId = "player-1".into();
        assert_eq!(id.as_str(), "player-1");
    }

    #[test]
    fn card_instance_id_from_string() {
        let id: CardInstanceId = String::from("inst-1").into();
        assert_eq!(id.as_str(), "inst-1");
    }

    #[test]
    fn card_definition_id_roundtrip_serde() {
        let id = CardDefinitionId::new("plains");
        let json = serde_json::to_string(&id).unwrap();
        let decoded: CardDefinitionId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, decoded);
    }
}
