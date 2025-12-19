import type { Game } from "../../game/Game"
import { InvalidEffectTargetError } from "../../game/GameErrors"
import type { Effect } from "../Effect"
import type { EffectContext } from "../EffectContext"

export class DrawTargetPlayerEffect implements Effect {
  constructor(private readonly amount: number) {}

  resolve(game: Game, context: EffectContext): void {
    const target = context.targets[0]
    if (!target || target.kind !== "PLAYER") {
      throw new InvalidEffectTargetError(
        "DrawTargetPlayerEffect",
        "Missing PLAYER target",
      )
    }

    game.drawCards(target.playerId, this.amount)
  }
}
