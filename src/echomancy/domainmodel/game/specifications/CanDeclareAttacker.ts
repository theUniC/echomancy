import { Step } from "../Steps"
import type { GameActionContext } from "./HasPriority"
import type { Specification } from "./Specification"

/**
 * Specification that checks if a player can declare attackers.
 *
 * A player can declare attackers if:
 * 1. It's the DECLARE_ATTACKERS step
 * 2. They are the current player (active player)
 * 3. They have at least one creature that can attack (untapped, hasn't attacked this turn)
 *
 * @example
 * const spec = new CanDeclareAttacker()
 * if (spec.isSatisfiedBy({ game, playerId })) {
 *   // Player can declare attackers
 * }
 */
export class CanDeclareAttacker implements Specification<GameActionContext> {
  isSatisfiedBy({ game, playerId }: GameActionContext): boolean {
    // Must be DECLARE_ATTACKERS step
    if (game.currentStep !== Step.DECLARE_ATTACKERS) {
      return false
    }

    // Must be the current player (active player)
    if (playerId !== game.currentPlayerId) {
      return false
    }

    // Must have at least one attackable creature
    const playerState = game.getPlayerState(playerId)
    const hasAttackableCreature = playerState.battlefield.cards.some((card) => {
      // Must be a creature
      if (!card.definition.types.includes("CREATURE")) {
        return false
      }

      // Get creature state from Game
      const creatureState = game.getCreatureState(card.instanceId)

      // Must be untapped and not have attacked this turn
      return !creatureState.isTapped && !creatureState.hasAttackedThisTurn
    })

    return hasAttackableCreature
  }
}
