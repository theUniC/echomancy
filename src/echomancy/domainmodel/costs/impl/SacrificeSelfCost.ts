/**
 * SacrificeSelfCost - Sacrifice the permanent that has this ability
 *
 * Validates:
 * - Permanent exists on battlefield
 * - Permanent is controlled by the player
 *
 * Moves the permanent from battlefield to graveyard
 *
 * TODO: Support sacrificing other permanents (not just self)
 * TODO: Support sacrificing multiple permanents
 * TODO: Support sacrificing permanents with specific properties (e.g., "sacrifice a creature")
 */

import type { Game } from "../../game/Game"
import {
  PermanentNotControlledError,
  PermanentNotFoundError,
} from "../../game/GameErrors"
import type { Cost, CostContext } from "../Cost"

export class SacrificeSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    // Find the permanent
    const playerState = game.getPlayerState(context.playerId)
    const permanent = playerState.battlefield.cards.find(
      (card) => card.instanceId === context.sourceId,
    )

    return permanent !== undefined
  }

  pay(game: Game, context: CostContext): void {
    // Find the permanent on ANY battlefield
    const playerIds = game.getPlayersInTurnOrder()
    let permanent = null
    let permanentOwnerState = null

    for (const playerId of playerIds) {
      const playerState = game.getPlayerState(playerId)
      const found = playerState.battlefield.cards.find(
        (card) => card.instanceId === context.sourceId,
      )
      if (found) {
        permanent = found
        permanentOwnerState = playerState
        break
      }
    }

    if (!permanent) {
      throw new PermanentNotFoundError(context.sourceId)
    }

    // Verify control
    if (permanent.ownerId !== context.playerId) {
      throw new PermanentNotControlledError(context.sourceId, context.playerId)
    }

    // Move from battlefield to graveyard
    const permanentIndex = permanentOwnerState?.battlefield.cards.findIndex(
      (card) => card.instanceId === context.sourceId,
    )
    permanentOwnerState?.battlefield.cards.splice(permanentIndex, 1)
    permanentOwnerState?.graveyard.cards.push(permanent)

    // TODO: Emit ZONE_CHANGED event for sacrifice
    // TODO: Handle triggered abilities that fire on sacrifice
  }
}
