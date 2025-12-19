import type { Game } from "../../game/Game"
import type { Effect } from "../Effect"
import type { EffectContext } from "../EffectContext"

export class DrawTargetPlayerEffect implements Effect {
  constructor(private readonly amount: number) {}

  resolve(game: Game, context: EffectContext): void {
    const target = context.targets[0]
    if (!target || target.kind !== "PLAYER") {
      throw new Error("Missing PLAYER target")
    }

    game.drawCards(target.playerId, this.amount)
  }
}
