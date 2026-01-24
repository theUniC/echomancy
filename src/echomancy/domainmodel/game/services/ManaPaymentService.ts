/**
 * ManaPaymentService
 *
 * Domain service for validating and executing mana cost payments.
 * Implements auto-pay logic with smart mana usage:
 * 1. Pay colored requirements first (exact color match)
 * 2. Pay generic cost with remaining mana (prefer colorless C, then colored)
 * 3. Colorless (C) requirements can only be paid with colorless mana
 *
 * This service is stateless and operates on ManaPool value objects.
 */

import type { ManaCost } from "../valueobjects/ManaCost"
import type { ManaColor, ManaPoolSnapshot } from "../valueobjects/ManaPool"
import { InsufficientManaError, ManaPool } from "../valueobjects/ManaPool"

export class ManaPaymentService {
  /**
   * Checks if a mana pool can pay for a given cost.
   *
   * @param pool - Current mana pool snapshot
   * @param cost - Mana cost to check
   * @returns true if the cost can be paid, false otherwise
   */
  static canPayCost(pool: ManaPoolSnapshot, cost: ManaCost): boolean {
    try {
      // Try to pay the cost - if it succeeds, return true
      ManaPaymentService.payForCost(ManaPool.fromSnapshot(pool), cost)
      return true
    } catch {
      return false
    }
  }

  /**
   * Pays for a mana cost from the given mana pool.
   * Mutates the pool by spending mana according to auto-pay rules.
   *
   * Auto-pay algorithm:
   * 1. Pay colored requirements (W, U, B, R, G) from exact color matches
   * 2. Pay colorless requirements (C) from colorless mana only
   * 3. Pay generic cost from remaining mana:
   *    - Prefer colorless (C) first
   *    - Then use colored mana in priority order: W, U, B, R, G
   *
   * @param pool - Mutable mana pool to spend from
   * @param cost - Mana cost to pay
   * @returns New ManaPool instance with mana spent
   * @throws InsufficientManaError if the cost cannot be paid
   */
  static payForCost(pool: ManaPool, cost: ManaCost): ManaPool {
    let remainingPool = pool

    // Step 1: Pay colored requirements first (exact color match)
    const coloredMana: ManaColor[] = ["W", "U", "B", "R", "G"]
    for (const color of coloredMana) {
      const required = cost[color] || 0
      if (required > 0) {
        const available = remainingPool.get(color)
        if (available < required) {
          throw new InsufficientManaError(color, required, available)
        }
        remainingPool = remainingPool.spend(color, required)
      }
    }

    // Step 2: Pay colorless requirements (C can only be paid with C)
    const colorlessRequired = cost.C || 0
    if (colorlessRequired > 0) {
      const available = remainingPool.get("C")
      if (available < colorlessRequired) {
        throw new InsufficientManaError("C", colorlessRequired, available)
      }
      remainingPool = remainingPool.spend("C", colorlessRequired)
    }

    // Step 3: Pay generic cost with remaining mana
    let genericRemaining = cost.generic

    if (genericRemaining > 0) {
      // Prefer colorless (C) first
      const colorlessAvailable = remainingPool.get("C")
      const colorlessToSpend = Math.min(colorlessAvailable, genericRemaining)
      if (colorlessToSpend > 0) {
        remainingPool = remainingPool.spend("C", colorlessToSpend)
        genericRemaining -= colorlessToSpend
      }

      // Then use colored mana in priority order: W, U, B, R, G
      for (const color of coloredMana) {
        if (genericRemaining === 0) break

        const available = remainingPool.get(color)
        const toSpend = Math.min(available, genericRemaining)
        if (toSpend > 0) {
          remainingPool = remainingPool.spend(color, toSpend)
          genericRemaining -= toSpend
        }
      }

      // If we still have generic cost remaining, we don't have enough mana
      if (genericRemaining > 0) {
        const totalAvailable = remainingPool.total()
        throw new Error(
          `Insufficient mana to pay generic cost: need ${genericRemaining}, available ${totalAvailable}`,
        )
      }
    }

    return remainingPool
  }
}
