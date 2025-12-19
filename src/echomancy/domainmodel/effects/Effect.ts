import type { CardInstance } from "../cards/CardInstance"
import type { Game } from "../game/Game"

export interface Effect {
  resolve(game: Game, source: CardInstance): void
}
