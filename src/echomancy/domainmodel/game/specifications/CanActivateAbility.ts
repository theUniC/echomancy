import type { GameActionContext } from "./HasPriority"
import type { Specification } from "./Specification"

/**
 * Specification that checks if a player can activate an ability.
 *
 * A player can activate an ability if:
 * 1. They have a permanent with an activated ability
 * 2. The ability's cost can be paid (e.g., not tapped for tap abilities)
 *
 * @example
 * const spec = new CanActivateAbility()
 * if (spec.isSatisfiedBy({ game, playerId })) {
 *   // Player has abilities they could activate
 * }
 */
export class CanActivateAbility implements Specification<GameActionContext> {
  isSatisfiedBy({ game, playerId }: GameActionContext): boolean {
    const playerState = game.getPlayerState(playerId)

    return playerState.battlefield.cards.some((card) => {
      // Check if card has an activated ability
      if (!card.definition.activatedAbility) {
        return false
      }

      // Check if the cost can be paid
      const cost = card.definition.activatedAbility.cost
      if (cost.type === "TAP") {
        const state = game.getCreatureState(card.instanceId)
        // Can activate if not tapped
        return !state.isTapped
      }

      return false
    })
  }
}
