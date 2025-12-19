import type { CardInstance } from "../../cards/CardInstance"
import type { Game } from "../../game/Game"
import type { Effect } from "../Effect"

export class NoOpEffect implements Effect {
  resolve(_game: Game, _source: CardInstance): void {}
}
