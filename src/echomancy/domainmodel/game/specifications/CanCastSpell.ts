import { SpellTimingService } from "../services/SpellTiming"
import type { GameActionContext } from "./HasPriority"
import type { Specification } from "./Specification"

/**
 * Specification that checks if a player can cast a spell.
 *
 * A player can cast a spell if they have at least one card in hand
 * that can be cast at the current timing (considering instant vs sorcery speed,
 * Flash keyword, game phase, turn ownership, and stack state).
 *
 * Note: This doesn't check priority - that's handled separately by HasPriority.
 *
 * @example
 * const spec = new CanCastSpell()
 * if (spec.isSatisfiedBy({ game, playerId })) {
 *   // Player has spells they could cast at current timing
 * }
 */
export class CanCastSpell implements Specification<GameActionContext> {
  isSatisfiedBy({ game, playerId }: GameActionContext): boolean {
    const playerState = game.getPlayerState(playerId)

    // Check if player has at least one non-land card they can cast at current timing
    return playerState.hand.cards.some((card) => {
      // Lands are not spells
      if (card.definition.types.includes("LAND")) {
        return false
      }

      // Check if timing is legal for this card
      return SpellTimingService.canCastAtCurrentTiming(game, playerId, card)
    })
  }
}
