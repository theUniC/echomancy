/**
 * Trigger - Declarative triggered abilities.
 * @see docs/ability-system.md
 */

import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import type { EffectContext } from "@/echomancy/domainmodel/effects/EffectContext"
import type { Game } from "@/echomancy/domainmodel/game/Game"
import type { GameEvent } from "@/echomancy/domainmodel/game/GameEvents"

export type Trigger = {
  eventType: GameEvent["type"]
  condition: (game: Game, event: GameEvent, sourceCard: CardInstance) => boolean
  effect: (game: Game, context: EffectContext) => void
}

/** Helper type for better type inference on the event parameter */
export type TriggerDefinition<T extends GameEvent["type"]> = {
  eventType: T
  condition: (
    game: Game,
    event: Extract<GameEvent, { type: T }>,
    sourceCard: CardInstance,
  ) => boolean
  effect: (game: Game, context: EffectContext) => void
}
