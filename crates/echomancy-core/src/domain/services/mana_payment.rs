//! ManaPaymentService — auto-pay mana costs from a pool.
//!
//! Stateless service that validates and executes mana cost payments.
//! Implements auto-pay logic with smart mana usage:
//!   1. Pay colored requirements first (exact color match)
//!   2. Pay colorless requirements (C) — can only be paid with colorless mana
//!   3. Pay generic cost with remaining mana (prefer colorless, then colored)
//!
//! Mirrors `ManaPaymentService.ts`.

use crate::domain::enums::ManaColor;
use crate::domain::errors::GameError;
use crate::domain::value_objects::mana::{ManaCost, ManaPool};

/// The five colored mana types in priority order for auto-pay.
const COLORED_MANA: [ManaColor; 5] = [
    ManaColor::White,
    ManaColor::Blue,
    ManaColor::Black,
    ManaColor::Red,
    ManaColor::Green,
];

/// Returns `true` if the given pool can pay the given cost.
///
/// This is a pure predicate that does not mutate anything.
pub fn can_pay_cost(pool: &ManaPool, cost: &ManaCost) -> bool {
    pay_cost(pool.clone(), cost).is_ok()
}

/// Pays for a mana cost from the given pool using the auto-pay algorithm.
///
/// Auto-pay algorithm:
/// 1. Pay colored requirements (W, U, B, R, G) from exact color matches.
/// 2. Pay colorless requirements (C) from colorless mana only.
/// 3. Pay generic cost from remaining mana:
///    - Prefer colorless (C) first.
///    - Then use colored mana in priority order: W, U, B, R, G.
///
/// Returns the new `ManaPool` with the cost deducted.
///
/// # Errors
///
/// Returns `GameError::InsufficientManaForSpell` if the cost cannot be paid.
pub(crate) fn pay_cost(pool: ManaPool, cost: &ManaCost) -> Result<ManaPool, GameError> {
    let mut remaining = pool;

    // Step 1: Pay colored requirements (exact color match).
    for &color in &COLORED_MANA {
        let required = color_requirement(cost, color);
        if required > 0 {
            let available = remaining.get(color);
            if available < required {
                return Err(insufficient_mana_error(color, required, available));
            }
            remaining = remaining.spend(color, required).map_err(|e| {
                GameError::InsufficientManaForSpell {
                    message: e.to_string(),
                }
            })?;
        }
    }

    // Step 2: Pay colorless requirements (C can only be paid with C).
    if cost.colorless > 0 {
        let available = remaining.get(ManaColor::Colorless);
        if available < cost.colorless {
            return Err(insufficient_mana_error(
                ManaColor::Colorless,
                cost.colorless,
                available,
            ));
        }
        remaining = remaining
            .spend(ManaColor::Colorless, cost.colorless)
            .map_err(|e| GameError::InsufficientManaForSpell {
                message: e.to_string(),
            })?;
    }

    // Step 3: Pay generic cost with remaining mana.
    let mut generic_remaining = cost.generic;

    if generic_remaining > 0 {
        // Prefer colorless first.
        let colorless_available = remaining.get(ManaColor::Colorless);
        let colorless_to_spend = colorless_available.min(generic_remaining);
        if colorless_to_spend > 0 {
            remaining = remaining
                .spend(ManaColor::Colorless, colorless_to_spend)
                .map_err(|e| GameError::InsufficientManaForSpell {
                    message: e.to_string(),
                })?;
            generic_remaining -= colorless_to_spend;
        }

        // Then use colored mana in priority order.
        for &color in &COLORED_MANA {
            if generic_remaining == 0 {
                break;
            }
            let available = remaining.get(color);
            let to_spend = available.min(generic_remaining);
            if to_spend > 0 {
                remaining = remaining.spend(color, to_spend).map_err(|e| {
                    GameError::InsufficientManaForSpell {
                        message: e.to_string(),
                    }
                })?;
                generic_remaining -= to_spend;
            }
        }

        if generic_remaining > 0 {
            let total_available = remaining.total();
            return Err(GameError::InsufficientManaForSpell {
                message: format!(
                    "Insufficient mana to pay generic cost: need {generic_remaining}, available {total_available}"
                ),
            });
        }
    }

    Ok(remaining)
}

/// Returns the required amount for a specific `ManaColor` from a `ManaCost`.
fn color_requirement(cost: &ManaCost, color: ManaColor) -> u32 {
    match color {
        ManaColor::White => cost.white,
        ManaColor::Blue => cost.blue,
        ManaColor::Black => cost.black,
        ManaColor::Red => cost.red,
        ManaColor::Green => cost.green,
        ManaColor::Colorless => cost.colorless,
    }
}

fn insufficient_mana_error(color: ManaColor, required: u32, available: u32) -> GameError {
    GameError::InsufficientManaForSpell {
        message: format!(
            "Insufficient {color} mana: requested {required}, available {available}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::mana::{ManaCost, ManaPool};

    fn pool_with(color: ManaColor, amount: u32) -> ManaPool {
        ManaPool::empty().add(color, amount).unwrap()
    }

    fn pool_from(colors: &[(ManaColor, u32)]) -> ManaPool {
        let mut pool = ManaPool::empty();
        for &(color, amount) in colors {
            pool = pool.add(color, amount).unwrap();
        }
        pool
    }

    // ---- can_pay_cost -------------------------------------------------------

    #[test]
    fn can_pay_cost_true_for_exact_colored_match() {
        let pool = pool_with(ManaColor::Red, 2);
        let cost = ManaCost::parse("RR").unwrap();
        assert!(can_pay_cost(&pool, &cost));
    }

    #[test]
    fn can_pay_cost_false_when_short_of_color() {
        let pool = pool_with(ManaColor::Red, 1);
        let cost = ManaCost::parse("RR").unwrap();
        assert!(!can_pay_cost(&pool, &cost));
    }

    #[test]
    fn can_pay_cost_true_for_generic_with_any_mana() {
        let pool = pool_with(ManaColor::Green, 3);
        let cost = ManaCost::parse("3").unwrap();
        assert!(can_pay_cost(&pool, &cost));
    }

    #[test]
    fn can_pay_cost_false_for_insufficient_generic() {
        let pool = pool_with(ManaColor::Green, 2);
        let cost = ManaCost::parse("3").unwrap();
        assert!(!can_pay_cost(&pool, &cost));
    }

    #[test]
    fn can_pay_cost_true_for_zero_cost() {
        let pool = ManaPool::empty();
        let cost = ManaCost::zero();
        assert!(can_pay_cost(&pool, &cost));
    }

    // ---- pay_cost: colored requirements ------------------------------------

    #[test]
    fn pays_exact_colored_requirement() {
        let pool = pool_with(ManaColor::Blue, 2);
        let cost = ManaCost::parse("UU").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::Blue), 0);
    }

    #[test]
    fn error_on_insufficient_colored_mana() {
        let pool = pool_with(ManaColor::Blue, 1);
        let cost = ManaCost::parse("UU").unwrap();
        assert!(pay_cost(pool, &cost).is_err());
    }

    #[test]
    fn pays_multiple_colored_requirements() {
        let pool = pool_from(&[
            (ManaColor::White, 2),
            (ManaColor::Blue, 1),
        ]);
        let cost = ManaCost::parse("WWU").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::White), 0);
        assert_eq!(result.get(ManaColor::Blue), 0);
    }

    // ---- pay_cost: colorless requirement -----------------------------------

    #[test]
    fn pays_colorless_requirement_from_colorless_mana() {
        let pool = pool_with(ManaColor::Colorless, 2);
        let cost = ManaCost::parse("CC").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::Colorless), 0);
    }

    #[test]
    fn colored_mana_cannot_pay_colorless_requirement() {
        // C symbol can only be paid with colorless (C) mana
        let pool = pool_with(ManaColor::Red, 2);
        let cost = ManaCost::parse("CC").unwrap();
        assert!(pay_cost(pool, &cost).is_err());
    }

    // ---- pay_cost: generic cost -------------------------------------------

    #[test]
    fn generic_cost_prefers_colorless_mana() {
        let pool = pool_from(&[
            (ManaColor::Colorless, 2),
            (ManaColor::Red, 2),
        ]);
        let cost = ManaCost::parse("2").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        // Colorless should be spent first
        assert_eq!(result.get(ManaColor::Colorless), 0);
        // Red should be untouched
        assert_eq!(result.get(ManaColor::Red), 2);
    }

    #[test]
    fn generic_cost_falls_back_to_colored_mana() {
        let pool = pool_with(ManaColor::Green, 3);
        let cost = ManaCost::parse("3").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::Green), 0);
    }

    #[test]
    fn generic_cost_uses_mixed_mana() {
        let pool = pool_from(&[
            (ManaColor::Colorless, 1),
            (ManaColor::Red, 1),
        ]);
        let cost = ManaCost::parse("2").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::Colorless), 0);
        assert_eq!(result.get(ManaColor::Red), 0);
    }

    // ---- pay_cost: mixed colored + generic --------------------------------

    #[test]
    fn pays_mixed_cost_colored_then_generic() {
        let pool = pool_from(&[
            (ManaColor::Blue, 2),
            (ManaColor::Red, 2),
        ]);
        let cost = ManaCost::parse("2UU").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        // Blue used for UU
        assert_eq!(result.get(ManaColor::Blue), 0);
        // Red used for generic 2
        assert_eq!(result.get(ManaColor::Red), 0);
    }

    #[test]
    fn leaves_remaining_mana_after_payment() {
        // Pool: 3R + 2G = 5 total.
        // Cost: 2R = 1 generic + 1 red.
        // Step 1: spend 1R for the R requirement → pool = 2R + 2G.
        // Step 3: spend 2R for the generic 2 (colored priority: W, U, B, R, G).
        // Remaining: 2G.
        let pool = pool_from(&[
            (ManaColor::Red, 3),
            (ManaColor::Green, 2),
        ]);
        let cost = ManaCost::parse("2R").unwrap();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::Red), 0);
        assert_eq!(result.get(ManaColor::Green), 2); // green left over
        assert_eq!(result.total(), 2);
    }

    #[test]
    fn zero_cost_does_not_change_pool() {
        let pool = pool_with(ManaColor::Red, 3);
        let cost = ManaCost::zero();
        let result = pay_cost(pool, &cost).unwrap();
        assert_eq!(result.get(ManaColor::Red), 3);
    }

    #[test]
    fn error_on_insufficient_generic_mana() {
        let pool = pool_with(ManaColor::Red, 1);
        let cost = ManaCost::parse("3").unwrap();
        let err = pay_cost(pool, &cost).unwrap_err();
        assert!(matches!(err, GameError::InsufficientManaForSpell { .. }));
    }
}
