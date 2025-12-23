/**
 * Cost - Domain model for ability and spell costs
 *
 * Costs are validated before execution and paid atomically.
 * If any cost cannot be paid, nothing happens.
 *
 * @see docs/architecture.md
 */

import type { Game } from "../game/Game"

/**
 * CostContext - Context information for cost validation and payment
 *
 * Contains minimal information needed to evaluate and pay costs:
 * - playerId: The player paying the cost
 * - source: The card or ability requiring the cost
 */
export type CostContext = {
  playerId: string
  sourceId: string
}

/**
 * Cost - Interface for all cost types
 *
 * Core contract:
 * - canPay MUST NOT mutate state
 * - pay MUST assume canPay == true
 * - pay MUST mutate state atomically
 */
export type Cost = {
  /**
   * Checks if this cost can be paid given the current game state
   *
   * MUST be side-effect free (no state mutations)
   *
   * @param game - Current game state
   * @param context - Context for cost evaluation
   * @returns true if cost can be paid, false otherwise
   */
  canPay(game: Game, context: CostContext): boolean

  /**
   * Pays this cost, mutating game state
   *
   * MUST assume canPay returned true
   * MUST mutate state atomically
   *
   * @param game - Current game state (will be mutated)
   * @param context - Context for cost payment
   */
  pay(game: Game, context: CostContext): void
}

/**
 * Validates that all costs can be paid before paying any
 *
 * This ensures atomic cost payment - either all costs are paid or none are.
 *
 * @param costs - Array of costs to validate
 * @param game - Current game state
 * @param context - Context for cost evaluation
 * @returns true if all costs can be paid, false otherwise
 */
export function canPayAllCosts(
  costs: readonly Cost[],
  game: Game,
  context: CostContext,
): boolean {
  return costs.every((cost) => cost.canPay(game, context))
}

/**
 * Pays all costs atomically
 *
 * MUST only be called after canPayAllCosts returns true
 * Pays costs in order
 *
 * @param costs - Array of costs to pay
 * @param game - Current game state (will be mutated)
 * @param context - Context for cost payment
 */
export function payAllCosts(
  costs: readonly Cost[],
  game: Game,
  context: CostContext,
): void {
  // All costs must be validated before calling this
  // We pay them in order
  for (const cost of costs) {
    cost.pay(game, context)
  }
}
