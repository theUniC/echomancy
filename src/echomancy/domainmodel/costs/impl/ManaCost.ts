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

export class ManaCost implements Cost {
  constructor(private readonly amounts: Partial<Record<ManaColor, number>>) {}

  canPay(game: Game, context: CostContext): boolean {
    const pool = game.getManaPool(context.playerId)

    // Check if player has sufficient mana of each required color
    for (const [color, amount] of Object.entries(this.amounts)) {
      if (amount === undefined || amount <= 0) continue

      const available = pool[color as ManaColor] ?? 0
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

      game.spendMana(context.playerId, color as ManaColor, amount)
    }
  }
}
