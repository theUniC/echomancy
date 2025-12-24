/**
 * SacrificeSelfCost - Sacrifice the permanent that has this ability
 *
 * Validates:
 * - Permanent exists on battlefield
 * - Permanent is controlled by the player
 *
 * Moves the permanent from battlefield to graveyard and triggers
 * zone change events (enabling "dies" triggers).
 *
 * TODO: Support sacrificing other permanents (not just self)
 * TODO: Support sacrificing multiple permanents
 * TODO: Support sacrificing permanents with specific properties (e.g., "sacrifice a creature")
 */

import type { Game } from "../../game/Game"
import type { Cost, CostContext } from "../Cost"
import { assertPermanentControl, findControlledPermanent } from "./helpers"

export class SacrificeSelfCost implements Cost {
  canPay(game: Game, context: CostContext): boolean {
    const permanent = findControlledPermanent(game, context)
    return permanent !== undefined
  }

  pay(game: Game, context: CostContext): void {
    // Verify the player controls the permanent
    // Note: We need to check control here because movePermanentToGraveyard
    // doesn't validate control (it can move any permanent to graveyard)
    assertPermanentControl(game, context.playerId, context.sourceId)

    // Use Game's method to handle complete zone transition with events
    // This will:
    // - Remove from battlefield
    // - Add to graveyard
    // - Clean up state
    // - Emit ZONE_CHANGED event
    // - Evaluate "dies" triggers
    game.movePermanentToGraveyard(context.sourceId, "sacrifice")
  }
}
