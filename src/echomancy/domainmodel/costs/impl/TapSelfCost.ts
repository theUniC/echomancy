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
import { PermanentAlreadyTappedError } from "../../game/GameErrors"
import {
  assertPermanentControl,
  type Cost,
  type CostContext,
  findControlledPermanent,
  findPermanentOnAnyBattlefield,
} from "../Cost"

export class TapSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    const permanent = findControlledPermanent(game, context)

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
    const { permanent } = findPermanentOnAnyBattlefield(game, context.sourceId)

    assertPermanentControl(game, context.playerId, context.sourceId)

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

    game.tapPermanent(permanent.instanceId)
  }
}
