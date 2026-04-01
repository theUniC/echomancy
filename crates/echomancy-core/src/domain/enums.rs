use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Card Types
// ============================================================================

/// The type of a Magic: The Gathering card.
///
/// Mirrors the TypeScript `CardType` union from `CardDefinition.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardType {
    Creature,
    Instant,
    Sorcery,
    Artifact,
    Enchantment,
    Planeswalker,
    Land,
}

impl fmt::Display for CardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CardType::Creature => "CREATURE",
            CardType::Instant => "INSTANT",
            CardType::Sorcery => "SORCERY",
            CardType::Artifact => "ARTIFACT",
            CardType::Enchantment => "ENCHANTMENT",
            CardType::Planeswalker => "PLANESWALKER",
            CardType::Land => "LAND",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Mana Colors
// ============================================================================

/// The six mana colors (including colorless) used in mana costs.
///
/// Derived from the `ManaCost` value object in `ManaCost.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManaColor {
    /// White mana (W)
    White,
    /// Blue mana (U)
    Blue,
    /// Black mana (B)
    Black,
    /// Red mana (R)
    Red,
    /// Green mana (G)
    Green,
    /// Colorless mana (C) — must be paid with colorless specifically
    Colorless,
}

impl fmt::Display for ManaColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ManaColor::White => "W",
            ManaColor::Blue => "U",
            ManaColor::Black => "B",
            ManaColor::Red => "R",
            ManaColor::Green => "G",
            ManaColor::Colorless => "C",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Static Abilities
// ============================================================================

/// Static ability keywords supported by the game engine.
///
/// Always-on abilities that affect rule checks only. Does not go on the stack.
/// Mirrors the TypeScript `StaticAbility` type from `CardDefinition.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StaticAbility {
    Flying,
    Reach,
    Vigilance,
    Haste,
    Flash,
    FirstStrike,
    DoubleStrike,
    Trample,
    Deathtouch,
    Lifelink,
    /// CR 702.11 — can't be the target of spells or abilities opponents control.
    Hexproof,
    /// CR 702.18 — can't be the target of spells or abilities at all.
    Shroud,
    /// CR 702.12 — this permanent can't be destroyed by lethal damage or effects.
    Indestructible,
    /// CR 302.6 — this permanent does not untap during its controller's untap step.
    DoesNotUntap,
    /// This permanent enters the battlefield tapped.
    EntersTapped,
}

impl fmt::Display for StaticAbility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            StaticAbility::Flying => "FLYING",
            StaticAbility::Reach => "REACH",
            StaticAbility::Vigilance => "VIGILANCE",
            StaticAbility::Haste => "HASTE",
            StaticAbility::Flash => "FLASH",
            StaticAbility::FirstStrike => "FIRST_STRIKE",
            StaticAbility::DoubleStrike => "DOUBLE_STRIKE",
            StaticAbility::Trample => "TRAMPLE",
            StaticAbility::Deathtouch => "DEATHTOUCH",
            StaticAbility::Lifelink => "LIFELINK",
            StaticAbility::Hexproof => "HEXPROOF",
            StaticAbility::Shroud => "SHROUD",
            StaticAbility::Indestructible => "INDESTRUCTIBLE",
            StaticAbility::DoesNotUntap => "DOES_NOT_UNTAP",
            StaticAbility::EntersTapped => "ENTERS_TAPPED",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Zone Names
// ============================================================================

/// The named zones in a Magic: The Gathering game.
///
/// Mirrors the TypeScript `ZoneName` type from `Zone.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZoneName {
    Hand,
    Battlefield,
    Graveyard,
    Stack,
    Library,
    Exile,
}

impl fmt::Display for ZoneName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ZoneName::Hand => "HAND",
            ZoneName::Battlefield => "BATTLEFIELD",
            ZoneName::Graveyard => "GRAVEYARD",
            ZoneName::Stack => "STACK",
            ZoneName::Library => "LIBRARY",
            ZoneName::Exile => "EXILE",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Steps / Phases
// ============================================================================

/// All phases and steps in a Magic: The Gathering turn.
///
/// Mirrors the TypeScript `Step` const object from `Steps.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Step {
    Untap,
    Upkeep,
    Draw,
    FirstMain,
    BeginningOfCombat,
    DeclareAttackers,
    DeclareBlockers,
    /// First combat damage step — only creatures with First Strike deal damage.
    /// Inserted between DeclareBlockers and CombatDamage.
    FirstStrikeDamage,
    CombatDamage,
    EndOfCombat,
    SecondMain,
    EndStep,
    Cleanup,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Step::Untap => "UNTAP",
            Step::Upkeep => "UPKEEP",
            Step::Draw => "DRAW",
            Step::FirstMain => "FIRST_MAIN",
            Step::BeginningOfCombat => "BEGINNING_OF_COMBAT",
            Step::DeclareAttackers => "DECLARE_ATTACKERS",
            Step::DeclareBlockers => "DECLARE_BLOCKERS",
            Step::FirstStrikeDamage => "FIRST_STRIKE_DAMAGE",
            Step::CombatDamage => "COMBAT_DAMAGE",
            Step::EndOfCombat => "END_OF_COMBAT",
            Step::SecondMain => "SECOND_MAIN",
            Step::EndStep => "END_STEP",
            Step::Cleanup => "CLEANUP",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Game Lifecycle State
// ============================================================================

/// Describes the current lifecycle phase of a game instance.
///
/// Mirrors the TypeScript `GameLifecycleState` enum from `Game.ts`.
///
/// - `Created`: Game instance exists; rules engine is not yet active.
/// - `Started`: Rules engine is active; game is in progress.
/// - `Finished`: Game has concluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameLifecycleState {
    Created,
    Started,
    Finished,
}

impl fmt::Display for GameLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameLifecycleState::Created => "CREATED",
            GameLifecycleState::Started => "STARTED",
            GameLifecycleState::Finished => "FINISHED",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Graveyard Reason
// ============================================================================

/// The reason a permanent was moved to the graveyard.
///
/// Mirrors the TypeScript `GraveyardReason` enum from `Game.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GraveyardReason {
    Sacrifice,
    Destroy,
    StateBased,
}

impl fmt::Display for GraveyardReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GraveyardReason::Sacrifice => "sacrifice",
            GraveyardReason::Destroy => "destroy",
            GraveyardReason::StateBased => "state-based",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CardType ---

    #[test]
    fn card_type_display() {
        assert_eq!(CardType::Creature.to_string(), "CREATURE");
        assert_eq!(CardType::Instant.to_string(), "INSTANT");
        assert_eq!(CardType::Sorcery.to_string(), "SORCERY");
        assert_eq!(CardType::Artifact.to_string(), "ARTIFACT");
        assert_eq!(CardType::Enchantment.to_string(), "ENCHANTMENT");
        assert_eq!(CardType::Planeswalker.to_string(), "PLANESWALKER");
        assert_eq!(CardType::Land.to_string(), "LAND");
    }

    #[test]
    fn card_type_serde_roundtrip() {
        let original = CardType::Creature;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"CREATURE\"");
        let decoded: CardType = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    // --- ManaColor ---

    #[test]
    fn mana_color_display() {
        assert_eq!(ManaColor::White.to_string(), "W");
        assert_eq!(ManaColor::Blue.to_string(), "U");
        assert_eq!(ManaColor::Black.to_string(), "B");
        assert_eq!(ManaColor::Red.to_string(), "R");
        assert_eq!(ManaColor::Green.to_string(), "G");
        assert_eq!(ManaColor::Colorless.to_string(), "C");
    }

    // --- StaticAbility ---

    #[test]
    fn static_ability_display() {
        assert_eq!(StaticAbility::Flying.to_string(), "FLYING");
        assert_eq!(StaticAbility::Reach.to_string(), "REACH");
        assert_eq!(StaticAbility::Vigilance.to_string(), "VIGILANCE");
        assert_eq!(StaticAbility::Haste.to_string(), "HASTE");
        assert_eq!(StaticAbility::Flash.to_string(), "FLASH");
        assert_eq!(StaticAbility::FirstStrike.to_string(), "FIRST_STRIKE");
        assert_eq!(StaticAbility::Trample.to_string(), "TRAMPLE");
        assert_eq!(StaticAbility::Deathtouch.to_string(), "DEATHTOUCH");
        assert_eq!(StaticAbility::Lifelink.to_string(), "LIFELINK");
    }

    #[test]
    fn static_ability_serde_roundtrip() {
        let original = StaticAbility::Flying;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"FLYING\"");
        let decoded: StaticAbility = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    // --- ZoneName ---

    #[test]
    fn zone_name_display() {
        assert_eq!(ZoneName::Hand.to_string(), "HAND");
        assert_eq!(ZoneName::Battlefield.to_string(), "BATTLEFIELD");
        assert_eq!(ZoneName::Graveyard.to_string(), "GRAVEYARD");
        assert_eq!(ZoneName::Stack.to_string(), "STACK");
        assert_eq!(ZoneName::Library.to_string(), "LIBRARY");
        assert_eq!(ZoneName::Exile.to_string(), "EXILE");
    }

    #[test]
    fn zone_name_serde_roundtrip() {
        let original = ZoneName::Battlefield;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"BATTLEFIELD\"");
        let decoded: ZoneName = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    // --- Step ---

    #[test]
    fn step_display() {
        assert_eq!(Step::Untap.to_string(), "UNTAP");
        assert_eq!(Step::Upkeep.to_string(), "UPKEEP");
        assert_eq!(Step::Draw.to_string(), "DRAW");
        assert_eq!(Step::FirstMain.to_string(), "FIRST_MAIN");
        assert_eq!(Step::BeginningOfCombat.to_string(), "BEGINNING_OF_COMBAT");
        assert_eq!(Step::DeclareAttackers.to_string(), "DECLARE_ATTACKERS");
        assert_eq!(Step::DeclareBlockers.to_string(), "DECLARE_BLOCKERS");
        assert_eq!(Step::FirstStrikeDamage.to_string(), "FIRST_STRIKE_DAMAGE");
        assert_eq!(Step::CombatDamage.to_string(), "COMBAT_DAMAGE");
        assert_eq!(Step::EndOfCombat.to_string(), "END_OF_COMBAT");
        assert_eq!(Step::SecondMain.to_string(), "SECOND_MAIN");
        assert_eq!(Step::EndStep.to_string(), "END_STEP");
        assert_eq!(Step::Cleanup.to_string(), "CLEANUP");
    }

    #[test]
    fn step_serde_roundtrip() {
        let original = Step::FirstMain;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"FIRST_MAIN\"");
        let decoded: Step = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    // --- GameLifecycleState ---

    #[test]
    fn game_lifecycle_state_display() {
        assert_eq!(GameLifecycleState::Created.to_string(), "CREATED");
        assert_eq!(GameLifecycleState::Started.to_string(), "STARTED");
        assert_eq!(GameLifecycleState::Finished.to_string(), "FINISHED");
    }

    #[test]
    fn game_lifecycle_state_serde_roundtrip() {
        let original = GameLifecycleState::Started;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"STARTED\"");
        let decoded: GameLifecycleState = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }

    // --- GraveyardReason ---

    #[test]
    fn graveyard_reason_display() {
        assert_eq!(GraveyardReason::Sacrifice.to_string(), "sacrifice");
        assert_eq!(GraveyardReason::Destroy.to_string(), "destroy");
        assert_eq!(GraveyardReason::StateBased.to_string(), "state-based");
    }

    #[test]
    fn graveyard_reason_serde_roundtrip() {
        let original = GraveyardReason::StateBased;
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"state-based\"");
        let decoded: GraveyardReason = serde_json::from_str(&json).unwrap();
        assert_eq!(original, decoded);
    }
}
