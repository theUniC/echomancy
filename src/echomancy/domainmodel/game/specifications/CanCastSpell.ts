import { Step } from "../Steps"
import type { GameActionContext } from "./HasPriority"
import type { Specification } from "./Specification"

/**
 * Specification that checks if a player can cast a spell.
 *
 * A player can cast a spell if:
 * 1. It's a main phase (for sorcery-speed spells)
 * 2. They have at least one castable card in hand
 *
 * Note: This doesn't check priority - that's handled separately by HasPriority.
 *
 * @example
 * const spec = new CanCastSpell()
 * if (spec.isSatisfiedBy({ game, playerId })) {
 *   // Player has spells they could cast
 * }
 */
export class CanCastSpell implements Specification<GameActionContext> {
  isSatisfiedBy({ game, playerId }: GameActionContext): boolean {
    // Must be in a main phase (for sorcery-speed)
    const isMainPhase =
      game.currentStep === Step.FIRST_MAIN ||
      game.currentStep === Step.SECOND_MAIN

    if (!isMainPhase) {
      return false
    }

    // Must have at least one castable card in hand
    const playerState = game.getPlayerState(playerId)
    const hasSpellInHand = playerState.hand.cards.some(
      (card) => !card.definition.types.includes("LAND"),
    )

    return hasSpellInHand
  }
}
