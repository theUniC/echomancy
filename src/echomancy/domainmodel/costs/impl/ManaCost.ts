/**
 * ManaCost - Pay mana from the player's mana pool
 *
 * Example: { G: 2, W: 1 } means "pay 2 green and 1 white mana"
 *
 * TODO: Support generic mana costs (colorless that can be paid with any color)
 * TODO: Support hybrid mana costs
 * TODO: Support Phyrexian mana costs
 * TODO: Support X costs
 */

import type { Game, ManaColor } from "../../game/Game"
import type { Cost, CostContext } from "../Cost"

/**
 * Type guard to verify a string is a valid ManaColor.
 * Safe because amounts can only contain ManaColor keys.
 */
function isManaColor(key: string): key is ManaColor {
  return ["W", "U", "B", "R", "G", "C"].includes(key)
}

export class ManaCost implements Cost {
  constructor(private readonly amounts: Partial<Record<ManaColor, number>>) {}

  canPay(game: Game, context: CostContext): boolean {
    const pool = game.getManaPool(context.playerId)

    // Check if player has sufficient mana of each required color
    for (const [color, amount] of Object.entries(this.amounts)) {
      if (amount === undefined || amount <= 0) continue

      // Type guard ensures color is ManaColor
      if (!isManaColor(color)) continue

      const available = pool[color] ?? 0
      if (available < amount) {
        return false
      }
    }

    return true
  }

  pay(game: Game, context: CostContext): void {
    // Spend mana for each color
    for (const [color, amount] of Object.entries(this.amounts)) {
      if (amount === undefined || amount <= 0) continue

      // Type guard ensures color is ManaColor
      if (!isManaColor(color)) continue

      game.spendMana(context.playerId, color, amount)
    }
  }
}
