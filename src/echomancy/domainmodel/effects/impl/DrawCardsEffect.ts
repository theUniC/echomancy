import type { CardInstance } from "../../cards/CardInstance"
import type { Game } from "../../game/Game"
import type { Effect } from "../Effect"

export class DrawCardsEffect implements Effect {
  constructor(private readonly amount: number) {}

  resolve(game: Game, source: CardInstance): void {
    game.drawCards(source.ownerId, this.amount)
  }
}
