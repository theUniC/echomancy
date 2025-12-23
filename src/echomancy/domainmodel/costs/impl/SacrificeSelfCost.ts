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
import type { Cost, CostContext } from "../Cost"
import {
  assertPermanentControl,
  findControlledPermanent,
  findPermanentOnAnyBattlefield,
} from "./helpers"

export class SacrificeSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    const permanent = findControlledPermanent(game, context)
    return permanent !== undefined
  }

  pay(game: Game, context: CostContext): void {
    const { permanent, ownerState } = findPermanentOnAnyBattlefield(
      game,
      context.sourceId,
    )

    assertPermanentControl(permanent, context.playerId, context.sourceId)

    // Move from battlefield to graveyard
    const permanentIndex = ownerState.battlefield.cards.findIndex(
      (card) => card.instanceId === context.sourceId,
    )
    ownerState.battlefield.cards.splice(permanentIndex, 1)
    ownerState.graveyard.cards.push(permanent)

    // TODO: Emit ZONE_CHANGED event for sacrifice
    // TODO: Handle triggered abilities that fire on sacrifice
  }
}
