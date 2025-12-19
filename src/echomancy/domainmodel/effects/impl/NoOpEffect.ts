import type { Game } from "../../game/Game"
import type { Effect } from "../Effect"
import type { EffectContext } from "../EffectContext"

export class NoOpEffect implements Effect {
  resolve(_game: Game, _context: EffectContext): void {}
}
