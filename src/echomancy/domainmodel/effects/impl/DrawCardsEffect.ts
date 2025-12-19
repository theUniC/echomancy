import type { Game } from "../../game/Game"
import type { Effect } from "../Effect"
import type { EffectContext } from "../EffectContext"

export class DrawCardsEffect implements Effect {
  constructor(private readonly amount: number) {}

  resolve(game: Game, context: EffectContext): void {
    game.drawCards(context.controllerId, this.amount)
  }
}
