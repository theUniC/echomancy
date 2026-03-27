use serde::{Deserialize, Serialize};

use crate::domain::enums::ManaColor;

// ============================================================================
// ManaPool
// ============================================================================

/// Snapshot of a mana pool suitable for serialisation or equality checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaPoolSnapshot {
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    pub colorless: u32,
}

/// Error returned when a player tries to spend more mana than is available.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Insufficient {color} mana: requested {requested}, available {available}")]
pub struct InsufficientManaError {
    pub color: ManaColor,
    pub requested: u32,
    pub available: u32,
}

/// Error returned when spending mana from a pool fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManaSpendError {
    #[error("Cannot spend zero mana")]
    ZeroAmount,
    #[error("{0}")]
    Insufficient(InsufficientManaError),
}

/// Error returned when adding mana to a pool fails.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ManaAddError {
    #[error("Cannot add zero mana")]
    ZeroAmount,
    #[error("Mana overflow for color {0}")]
    Overflow(ManaColor),
}

/// Immutable representation of a player's mana pool.
///
/// All mutating operations return a **new** instance; the original is unchanged.
/// This mirrors the TypeScript `ManaPool` value object in `ManaPool.ts`.
///
/// # Examples
///
/// ```
/// use echomancy_core::prelude::{ManaPool, ManaColor};
///
/// let pool = ManaPool::empty()
///     .add(ManaColor::Red, 2).unwrap()
///     .add(ManaColor::Blue, 1).unwrap();
/// assert_eq!(pool.get(ManaColor::Red), 2);
/// assert_eq!(pool.total(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManaPool {
    white: u32,
    blue: u32,
    black: u32,
    red: u32,
    green: u32,
    colorless: u32,
}

impl ManaPool {
    /// Creates an empty mana pool with all colors at zero.
    pub fn empty() -> Self {
        Self {
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    /// Reconstructs a `ManaPool` from a previously captured snapshot.
    pub fn from_snapshot(snapshot: ManaPoolSnapshot) -> Self {
        Self {
            white: snapshot.white,
            blue: snapshot.blue,
            black: snapshot.black,
            red: snapshot.red,
            green: snapshot.green,
            colorless: snapshot.colorless,
        }
    }

    // ---- accessors ----------------------------------------------------------

    /// Returns the amount of mana available for the given color.
    pub fn get(&self, color: ManaColor) -> u32 {
        match color {
            ManaColor::White => self.white,
            ManaColor::Blue => self.blue,
            ManaColor::Black => self.black,
            ManaColor::Red => self.red,
            ManaColor::Green => self.green,
            ManaColor::Colorless => self.colorless,
        }
    }

    /// Returns the total mana across all colors.
    pub fn total(&self) -> u32 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    /// Returns `true` if all colors are at zero.
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }

    // ---- builders -----------------------------------------------------------

    /// Returns a new `ManaPool` with `amount` of `color` added.
    ///
    /// # Errors
    ///
    /// Returns [`ManaAddError::ZeroAmount`] if `amount` is zero.
    /// Returns [`ManaAddError::Overflow`] if `amount` would overflow the stored `u32`.
    /// (In practice overflow cannot happen in a normal game; the check prevents
    /// wrapping on pathological inputs.)
    pub fn add(
        &self,
        color: ManaColor,
        amount: u32,
    ) -> Result<Self, ManaAddError> {
        if amount == 0 {
            return Err(ManaAddError::ZeroAmount);
        }
        let mut next = self.clone();
        let slot = next.slot_mut(color);
        *slot = slot
            .checked_add(amount)
            .ok_or(ManaAddError::Overflow(color))?;
        Ok(next)
    }

    /// Returns a new `ManaPool` with `amount` of `color` removed.
    ///
    /// # Errors
    ///
    /// Returns [`ManaSpendError::ZeroAmount`] if `amount` is zero.
    /// Returns [`ManaSpendError::Insufficient`] if less than `amount` is available.
    pub fn spend(
        &self,
        color: ManaColor,
        amount: u32,
    ) -> Result<Self, ManaSpendError> {
        if amount == 0 {
            return Err(ManaSpendError::ZeroAmount);
        }
        let available = self.get(color);
        if available < amount {
            return Err(ManaSpendError::Insufficient(InsufficientManaError {
                color,
                requested: amount,
                available,
            }));
        }
        let mut next = self.clone();
        *next.slot_mut(color) -= amount;
        Ok(next)
    }

    /// Returns a new, empty `ManaPool` (discards all floating mana).
    pub fn clear(&self) -> Self {
        Self::empty()
    }

    // ---- snapshot -----------------------------------------------------------

    /// Exports the pool as a serialisable snapshot.
    pub fn to_snapshot(&self) -> ManaPoolSnapshot {
        ManaPoolSnapshot {
            white: self.white,
            blue: self.blue,
            black: self.black,
            red: self.red,
            green: self.green,
            colorless: self.colorless,
        }
    }

    // ---- private helpers ----------------------------------------------------

    fn slot_mut(&mut self, color: ManaColor) -> &mut u32 {
        match color {
            ManaColor::White => &mut self.white,
            ManaColor::Blue => &mut self.blue,
            ManaColor::Black => &mut self.black,
            ManaColor::Red => &mut self.red,
            ManaColor::Green => &mut self.green,
            ManaColor::Colorless => &mut self.colorless,
        }
    }
}

// ============================================================================
// ManaCost
// ============================================================================

/// The mana cost of a spell or ability.
///
/// Mirrors the TypeScript `ManaCost` type from `ManaCost.ts`.
///
/// A cost has:
/// - `generic`: any mana (e.g. the "2" in "2UU")
/// - Per-color amounts for W / U / B / R / G / C
///
/// # Examples
///
/// ```
/// use echomancy_core::prelude::ManaCost;
///
/// let cost = ManaCost::parse("2UU").unwrap();
/// assert_eq!(cost.generic, 2);
/// assert_eq!(cost.blue, 2);
/// assert_eq!(cost.total(), 4);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    /// Generic mana — may be paid with any color.
    pub generic: u32,
    pub white: u32,
    pub blue: u32,
    pub black: u32,
    pub red: u32,
    pub green: u32,
    /// Colorless mana — must be paid with colorless specifically.
    pub colorless: u32,
}

impl ManaCost {
    /// Creates a zero-cost `ManaCost`.
    pub fn zero() -> Self {
        Self {
            generic: 0,
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    /// Parses a mana cost string such as `"2UU"`, `"BBB"`, or `"4"`.
    ///
    /// Supported characters:
    /// - ASCII digits `0–9` — contribute to the generic component
    /// - `W`, `U`, `B`, `R`, `G`, `C` — contribute to the respective color
    ///
    /// An empty string is treated as `{0}` (free spell).
    ///
    /// # Errors
    ///
    /// Returns `Err(String)` if the input contains unrecognised characters.
    pub fn parse(cost_string: &str) -> Result<Self, String> {
        if cost_string.is_empty() {
            return Ok(Self::zero());
        }

        let mut cost = Self::zero();
        let mut generic_digits = String::new();
        let mut parsing_generic = true;

        for ch in cost_string.chars() {
            if parsing_generic && ch.is_ascii_digit() {
                generic_digits.push(ch);
            } else {
                parsing_generic = false;
                match ch {
                    'W' => cost.white += 1,
                    'U' => cost.blue += 1,
                    'B' => cost.black += 1,
                    'R' => cost.red += 1,
                    'G' => cost.green += 1,
                    'C' => cost.colorless += 1,
                    _ => {
                        return Err(format!(
                            "Invalid mana cost format: '{cost_string}'"
                        ))
                    }
                }
            }
        }

        if !generic_digits.is_empty() {
            cost.generic = generic_digits
                .parse::<u32>()
                .map_err(|_| format!("Invalid generic cost in '{cost_string}'"))?;
        }

        Ok(cost)
    }

    /// Returns the total converted mana cost (CMC).
    pub fn total(&self) -> u32 {
        self.generic + self.white + self.blue + self.black + self.red + self.green + self.colorless
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::ManaColor;

    // ---- ManaPool::empty ----------------------------------------------------

    #[test]
    fn empty_pool_has_all_zeros() {
        let pool = ManaPool::empty();
        assert_eq!(pool.get(ManaColor::White), 0);
        assert_eq!(pool.get(ManaColor::Blue), 0);
        assert_eq!(pool.get(ManaColor::Black), 0);
        assert_eq!(pool.get(ManaColor::Red), 0);
        assert_eq!(pool.get(ManaColor::Green), 0);
        assert_eq!(pool.get(ManaColor::Colorless), 0);
    }

    // ---- ManaPool::from_snapshot -------------------------------------------

    #[test]
    fn from_snapshot_restores_all_values() {
        let snap = ManaPoolSnapshot {
            white: 1,
            blue: 2,
            black: 3,
            red: 4,
            green: 5,
            colorless: 6,
        };
        let pool = ManaPool::from_snapshot(snap);
        assert_eq!(pool.get(ManaColor::White), 1);
        assert_eq!(pool.get(ManaColor::Blue), 2);
        assert_eq!(pool.get(ManaColor::Black), 3);
        assert_eq!(pool.get(ManaColor::Red), 4);
        assert_eq!(pool.get(ManaColor::Green), 5);
        assert_eq!(pool.get(ManaColor::Colorless), 6);
    }

    // ---- ManaPool::add -----------------------------------------------------

    #[test]
    fn add_returns_new_instance_original_unchanged() {
        let pool1 = ManaPool::empty();
        let pool2 = pool1.add(ManaColor::Red, 2).unwrap();

        assert_eq!(pool1.get(ManaColor::Red), 0); // original unchanged
        assert_eq!(pool2.get(ManaColor::Red), 2);
    }

    #[test]
    fn add_accumulates_mana() {
        let pool = ManaPool::empty()
            .add(ManaColor::Red, 2)
            .unwrap()
            .add(ManaColor::Red, 3)
            .unwrap();
        assert_eq!(pool.get(ManaColor::Red), 5);
    }

    // ---- ManaPool::spend ---------------------------------------------------

    #[test]
    fn spend_returns_new_instance_original_unchanged() {
        let pool1 = ManaPool::empty().add(ManaColor::Red, 3).unwrap();
        let pool2 = pool1.spend(ManaColor::Red, 2).unwrap();

        assert_eq!(pool1.get(ManaColor::Red), 3); // original unchanged
        assert_eq!(pool2.get(ManaColor::Red), 1);
    }

    #[test]
    fn spend_insufficient_mana_returns_error() {
        let pool = ManaPool::empty().add(ManaColor::Red, 1).unwrap();
        let err = pool.spend(ManaColor::Red, 2).unwrap_err();
        let ManaSpendError::Insufficient(inner) = err else {
            panic!("expected Insufficient variant");
        };
        assert_eq!(inner.color, ManaColor::Red);
        assert_eq!(inner.requested, 2);
        assert_eq!(inner.available, 1);
    }

    #[test]
    fn spend_exact_amount_leaves_zero() {
        let pool = ManaPool::empty()
            .add(ManaColor::Blue, 3)
            .unwrap()
            .spend(ManaColor::Blue, 3)
            .unwrap();
        assert_eq!(pool.get(ManaColor::Blue), 0);
    }

    // ---- ManaPool::clear ---------------------------------------------------

    #[test]
    fn clear_returns_empty_pool_original_unchanged() {
        let pool = ManaPool::empty()
            .add(ManaColor::Red, 3)
            .unwrap()
            .add(ManaColor::Blue, 2)
            .unwrap();
        let cleared = pool.clear();

        assert_eq!(pool.get(ManaColor::Red), 3); // original unchanged
        assert!(cleared.is_empty());
    }

    // ---- ManaPool::is_empty ------------------------------------------------

    #[test]
    fn empty_pool_is_empty() {
        assert!(ManaPool::empty().is_empty());
    }

    #[test]
    fn pool_with_mana_is_not_empty() {
        let pool = ManaPool::empty().add(ManaColor::Red, 1).unwrap();
        assert!(!pool.is_empty());
    }

    // ---- ManaPool equality -------------------------------------------------

    #[test]
    fn equal_pools_are_equal() {
        let pool1 = ManaPool::empty()
            .add(ManaColor::Red, 2)
            .unwrap()
            .add(ManaColor::Blue, 1)
            .unwrap();
        let pool2 = ManaPool::empty()
            .add(ManaColor::Red, 2)
            .unwrap()
            .add(ManaColor::Blue, 1)
            .unwrap();
        assert_eq!(pool1, pool2);
    }

    #[test]
    fn different_pools_are_not_equal() {
        let pool1 = ManaPool::empty().add(ManaColor::Red, 2).unwrap();
        let pool2 = ManaPool::empty().add(ManaColor::Red, 3).unwrap();
        assert_ne!(pool1, pool2);
    }

    // ---- ManaPool::to_snapshot ---------------------------------------------

    #[test]
    fn snapshot_matches_pool_state() {
        let pool = ManaPool::empty()
            .add(ManaColor::Red, 2)
            .unwrap()
            .add(ManaColor::Green, 1)
            .unwrap();
        let snap = pool.to_snapshot();
        assert_eq!(snap.white, 0);
        assert_eq!(snap.blue, 0);
        assert_eq!(snap.black, 0);
        assert_eq!(snap.red, 2);
        assert_eq!(snap.green, 1);
        assert_eq!(snap.colorless, 0);
    }

    // ---- ManaPool::total ---------------------------------------------------

    #[test]
    fn total_sums_all_colors() {
        let pool = ManaPool::empty()
            .add(ManaColor::Red, 2)
            .unwrap()
            .add(ManaColor::Blue, 3)
            .unwrap()
            .add(ManaColor::Green, 1)
            .unwrap();
        assert_eq!(pool.total(), 6);
    }

    // ---- ManaCost::parse ---------------------------------------------------

    #[test]
    fn parse_empty_string_gives_zero_cost() {
        let cost = ManaCost::parse("").unwrap();
        assert_eq!(cost, ManaCost::zero());
    }

    #[test]
    fn parse_generic_only() {
        let cost = ManaCost::parse("4").unwrap();
        assert_eq!(cost.generic, 4);
        assert_eq!(cost.blue, 0);
        assert_eq!(cost.total(), 4);
    }

    #[test]
    fn parse_colored_only() {
        let cost = ManaCost::parse("BBB").unwrap();
        assert_eq!(cost.generic, 0);
        assert_eq!(cost.black, 3);
        assert_eq!(cost.total(), 3);
    }

    #[test]
    fn parse_mixed_generic_and_color() {
        let cost = ManaCost::parse("2UU").unwrap();
        assert_eq!(cost.generic, 2);
        assert_eq!(cost.blue, 2);
        assert_eq!(cost.total(), 4);
    }

    #[test]
    fn parse_colorless_symbol() {
        let cost = ManaCost::parse("2C").unwrap();
        assert_eq!(cost.generic, 2);
        assert_eq!(cost.colorless, 1);
        assert_eq!(cost.total(), 3);
    }

    #[test]
    fn parse_all_colors() {
        let cost = ManaCost::parse("1WUBRG").unwrap();
        assert_eq!(cost.generic, 1);
        assert_eq!(cost.white, 1);
        assert_eq!(cost.blue, 1);
        assert_eq!(cost.black, 1);
        assert_eq!(cost.red, 1);
        assert_eq!(cost.green, 1);
        assert_eq!(cost.total(), 6);
    }

    #[test]
    fn parse_invalid_character_returns_err() {
        let result = ManaCost::parse("2XU");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid mana cost format"));
    }

    // ---- ManaCost::zero ----------------------------------------------------

    #[test]
    fn zero_cost_has_zero_total() {
        assert_eq!(ManaCost::zero().total(), 0);
    }

    // ---- ManaCost::parse additional cases ----------------------------------

    #[test]
    fn parse_multi_digit_generic_cost() {
        let cost = ManaCost::parse("12UU").unwrap();
        assert_eq!(cost.generic, 12);
        assert_eq!(cost.blue, 2);
        assert_eq!(cost.total(), 14);
    }

    #[test]
    fn parse_lowercase_returns_err() {
        let result = ManaCost::parse("2uu");
        assert!(result.is_err());
    }

    // ---- ManaPool::add / spend with zero amount ----------------------------

    #[test]
    fn add_zero_amount_returns_err() {
        let pool = ManaPool::empty();
        assert!(pool.add(ManaColor::Red, 0).is_err());
    }

    #[test]
    fn spend_zero_amount_returns_err() {
        let pool = ManaPool::empty().add(ManaColor::Red, 1).unwrap();
        assert!(pool.spend(ManaColor::Red, 0).is_err());
    }
}
