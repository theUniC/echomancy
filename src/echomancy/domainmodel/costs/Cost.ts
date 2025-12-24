/**
 * Cost - Domain model for ability and spell costs
 *
 * Costs are validated before execution and paid atomically.
 * If any cost cannot be paid, nothing happens.
 *
 * @see docs/architecture.md
 */

import type { CardInstance } from "../cards/CardInstance"
import type { Game } from "../game/Game"
import {
  CannotPayCostsError,
  PermanentNotControlledError,
  PermanentNotFoundError,
} from "../game/GameErrors"
import type { PlayerState } from "../game/PlayerState"

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
 * Validates all costs before paying any to ensure atomicity.
 * If any cost cannot be paid, throws CannotPayCostsError.
 *
 * @param costs - Array of costs to pay
 * @param game - Current game state (will be mutated)
 * @param context - Context for cost payment
 * @throws CannotPayCostsError if any cost cannot be paid
 */
export function payAllCosts(
  costs: readonly Cost[],
  game: Game,
  context: CostContext,
): void {
  // Validate all costs before paying any (ensures atomicity)
  if (!canPayAllCosts(costs, game, context)) {
    throw new CannotPayCostsError("One or more costs cannot be paid")
  }

  // Pay costs in order
  for (const cost of costs) {
    cost.pay(game, context)
  }
}

// ============================================================================
// HELPER FUNCTIONS FOR COST IMPLEMENTATIONS
// ============================================================================

/**
 * Result of finding a permanent across all battlefields
 */
export type PermanentSearchResult = {
  permanent: CardInstance
  controllerState: PlayerState
}

/**
 * Finds a permanent on the controller's battlefield
 *
 * Used by canPay() methods to check if a permanent exists and is controlled
 *
 * @param game - Current game state
 * @param context - Cost context with player and source IDs
 * @returns The permanent if found and controlled, undefined otherwise
 */
export function findControlledPermanent(
  game: Game,
  context: CostContext,
): CardInstance | undefined {
  const playerState = game.getPlayerState(context.playerId)
  return playerState.battlefield.cards.find(
    (card) => card.instanceId === context.sourceId,
  )
}

/**
 * Finds a permanent on ANY player's battlefield
 *
 * Used by pay() methods to locate and validate permanents before mutating state
 *
 * @param game - Current game state
 * @param sourceId - The permanent's instance ID
 * @returns Search result with permanent and controller state
 * @throws PermanentNotFoundError if permanent not on any battlefield
 */
export function findPermanentOnAnyBattlefield(
  game: Game,
  sourceId: string,
): PermanentSearchResult {
  const playerIds = game.getPlayersInTurnOrder()

  for (const playerId of playerIds) {
    const playerState = game.getPlayerState(playerId)
    const found = playerState.battlefield.cards.find(
      (card) => card.instanceId === sourceId,
    )
    if (found) {
      return { permanent: found, controllerState: playerState }
    }
  }

  throw new PermanentNotFoundError(sourceId)
}

/**
 * Validates that a player controls a specific permanent
 *
 * Control is determined by whether the permanent is present on the
 * specified player's battlefield, not by its owner.
 *
 * @param game - Current game state
 * @param playerId - The player who should control the permanent
 * @param permanentId - The permanent's instance ID
 * @throws PermanentNotFoundError if permanent doesn't exist on any battlefield
 * @throws PermanentNotControlledError if player doesn't control the permanent
 */
export function assertPermanentControl(
  game: Game,
  playerId: string,
  permanentId: string,
): void {
  // First check if permanent exists on any battlefield
  const playerIds = game.getPlayersInTurnOrder()
  let foundAnywhere = false

  for (const id of playerIds) {
    const state = game.getPlayerState(id)
    if (
      state.battlefield.cards.some((card) => card.instanceId === permanentId)
    ) {
      foundAnywhere = true
      break
    }
  }

  if (!foundAnywhere) {
    throw new PermanentNotFoundError(permanentId)
  }

  // Then check if the specific player controls it
  const playerState = game.getPlayerState(playerId)
  const controlsPermanent = playerState.battlefield.cards.some(
    (card) => card.instanceId === permanentId,
  )

  if (!controlsPermanent) {
    throw new PermanentNotControlledError(permanentId, playerId)
  }
}
