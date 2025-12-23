/**
 * Effect - The executable part of an ability.
 * @see docs/effect-system.md
 */

import type { Game } from "../game/Game"
import type { EffectContext } from "./EffectContext"

export interface Effect {
  resolve(game: Game, context: EffectContext): void
}
