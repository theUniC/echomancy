import type { Game } from "../game/Game"
import type { EffectContext } from "./EffectContext"

export interface Effect {
  resolve(game: Game, context: EffectContext): void
}
