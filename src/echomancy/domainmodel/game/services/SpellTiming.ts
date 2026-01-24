import type { CardInstance } from "../../../cards/CardInstance"
import type { Game } from "../Game"
import { Step } from "../Steps"

/**
 * Domain service for spell timing validation.
 *
 * Determines when spells can be legally cast based on card type,
 * game phase, turn ownership, stack state, and keywords like Flash.
 *
 * Timing Categories:
 * - Sorcery Speed: Active player's main phase, stack empty
 * - Instant Speed: Any time player has priority
 *
 * @see docs/specs/active/B1-05-spell-timing.md
 */
// biome-ignore lint/complexity/noStaticOnlyClass: Domain service pattern uses static methods for stateless operations
export class SpellTimingService {
  /**
   * Checks if a card follows sorcery-speed timing rules.
   *
   * Sorcery speed applies to:
   * - Sorceries
   * - Creatures (unless they have Flash)
   * - Artifacts (unless they have Flash)
   * - Enchantments (unless they have Flash)
   * - Planeswalkers
   *
   * @param card - The card to check
   * @returns true if card requires sorcery-speed timing
   */
  static isSorcerySpeed(card: CardInstance): boolean {
    const { types, staticAbilities = [] } = card.definition

    // Instants are NEVER sorcery speed
    if (types.includes("INSTANT")) {
      return false
    }

    // Cards with Flash use instant speed
    if (staticAbilities.includes("FLASH")) {
      return false
    }

    // Everything else (sorceries, creatures, artifacts, enchantments, planeswalkers) is sorcery speed
    return true
  }

  /**
   * Checks if a card can be cast at instant speed.
   *
   * Instant speed applies to:
   * - Instants
   * - Any permanent with Flash keyword
   *
   * @param card - The card to check
   * @returns true if card can be cast at instant speed
   */
  static isInstantSpeed(card: CardInstance): boolean {
    const { types, staticAbilities = [] } = card.definition

    // Instants are always instant speed
    if (types.includes("INSTANT")) {
      return true
    }

    // Cards with Flash can be cast at instant speed
    if (staticAbilities.includes("FLASH")) {
      return true
    }

    return false
  }

  /**
   * Checks if a card can be cast at the current game timing.
   *
   * Validates:
   * - Turn ownership (for sorcery speed)
   * - Phase restrictions (for sorcery speed)
   * - Stack state (for sorcery speed)
   *
   * @param game - The current game state
   * @param playerId - The player attempting to cast
   * @param card - The card to cast
   * @returns true if timing is legal for casting this card
   */
  static canCastAtCurrentTiming(
    game: Game,
    playerId: string,
    card: CardInstance,
  ): boolean {
    // Instant speed: can be cast any time player has priority
    if (SpellTimingService.isInstantSpeed(card)) {
      return true
    }

    // Sorcery speed requires:
    // 1. Active player's turn
    if (game.currentPlayerId !== playerId) {
      return false
    }

    // 2. Main phase (FIRST_MAIN or SECOND_MAIN)
    const isMainPhase =
      game.currentStep === Step.FIRST_MAIN ||
      game.currentStep === Step.SECOND_MAIN

    if (!isMainPhase) {
      return false
    }

    // 3. Stack must be empty
    if (game.getStack().length > 0) {
      return false
    }

    return true
  }
}
