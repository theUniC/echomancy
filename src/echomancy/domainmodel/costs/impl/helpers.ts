/**
 * Helper functions for cost implementations
 *
 * These utilities reduce code duplication across cost implementations
 * by providing common operations for finding and validating permanents.
 */

import type { CardInstance } from "../../cards/CardInstance"
import type { Game } from "../../game/Game"
import {
  PermanentNotControlledError,
  PermanentNotFoundError,
} from "../../game/GameErrors"
import type { PlayerState } from "../../game/PlayerState"
import type { CostContext } from "../Cost"

/**
 * Result of finding a permanent across all battlefields
 */
type PermanentSearchResult = {
  permanent: CardInstance
  ownerState: PlayerState
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
 * @returns Search result with permanent and owner state
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
      return { permanent: found, ownerState: playerState }
    }
  }

  throw new PermanentNotFoundError(sourceId)
}

/**
 * Validates that a player controls a specific permanent
 *
 * @param permanent - The permanent to check
 * @param playerId - The player who should control it
 * @param permanentId - The permanent's ID (for error messages)
 * @throws PermanentNotControlledError if player doesn't control the permanent
 */
export function assertPermanentControl(
  permanent: CardInstance,
  playerId: string,
  permanentId: string,
): void {
  if (permanent.ownerId !== playerId) {
    throw new PermanentNotControlledError(permanentId, playerId)
  }
}
