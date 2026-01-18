import { Step } from "../Steps"
import type { GameActionContext } from "./HasPriority"
import type { Specification } from "./Specification"

/**
 * Specification that checks if a player can play a land.
 *
 * A player can play a land if:
 * 1. They are the current player (their turn)
 * 2. They haven't played a land this turn
 * 3. It's a main phase (first or second)
 *
 * @example
 * const spec = new CanPlayLand()
 * if (spec.isSatisfiedBy({ game, playerId })) {
 *   // Player can play a land
 * }
 */
export class CanPlayLand implements Specification<GameActionContext> {
  isSatisfiedBy({ game, playerId }: GameActionContext): boolean {
    // Must be the current player
    if (playerId !== game.currentPlayerId) {
      return false
    }

    // Must not have played a land this turn
    if (game.playedLandsThisTurn > 0) {
      return false
    }

    // Must be in a main phase
    const isMainPhase =
      game.currentStep === Step.FIRST_MAIN ||
      game.currentStep === Step.SECOND_MAIN

    return isMainPhase
  }
}
