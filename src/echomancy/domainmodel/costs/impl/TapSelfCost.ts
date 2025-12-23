/**
 * TapSelfCost - Tap the permanent that has this ability
 *
 * Validates:
 * - Permanent exists on battlefield
 * - Permanent is untapped
 * - Permanent is controlled by the player
 *
 * TODO: Support tapping other permanents (not just self)
 * TODO: Support tapping multiple permanents
 */

import type { Game } from "../../game/Game"
import {
  PermanentAlreadyTappedError,
  PermanentNotControlledError,
  PermanentNotFoundError,
} from "../../game/GameErrors"
import type { Cost, CostContext } from "../Cost"

export class TapSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    // Find the permanent
    const playerState = game.getPlayerState(context.playerId)
    const permanent = playerState.battlefield.cards.find(
      (card) => card.instanceId === context.sourceId,
    )

    if (!permanent) {
      return false
    }

    // Check if permanent is a creature (MVP: only creatures have tap state)
    // TODO: Track tap state for all permanents, not just creatures
    const isCreature = permanent.definition.types.includes("CREATURE")
    if (!isCreature) {
      // For MVP, non-creatures are assumed to be untapped
      return true
    }

    // Check if creature is untapped
    const creatureState = game.getCreatureState(permanent.instanceId)
    return !creatureState.isTapped
  }

  pay(game: Game, context: CostContext): void {
    // Find the permanent on ANY battlefield
    const playerIds = game.getPlayersInTurnOrder()
    let permanent = null

    for (const playerId of playerIds) {
      const playerState = game.getPlayerState(playerId)
      const found = playerState.battlefield.cards.find(
        (card) => card.instanceId === context.sourceId,
      )
      if (found) {
        permanent = found
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

    // Check if permanent is a creature (MVP: only creatures have tap state)
    const isCreature = permanent.definition.types.includes("CREATURE")
    if (!isCreature) {
      // For MVP, non-creatures are assumed to be untapped
      // In the future, all permanents will have tap state
      return
    }

    // Tap the creature
    const creatureState = game.getCreatureState(permanent.instanceId)
    if (creatureState.isTapped) {
      throw new PermanentAlreadyTappedError(context.sourceId)
    }

    creatureState.isTapped = true
  }
}
