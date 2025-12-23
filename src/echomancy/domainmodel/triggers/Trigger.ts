import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import type { EffectContext } from "@/echomancy/domainmodel/effects/EffectContext"
import type { Game } from "@/echomancy/domainmodel/game/Game"
import type { GameEvent } from "@/echomancy/domainmodel/game/GameEvents"

/**
 * Trigger - Declarative triggered abilities ("When X, if Y, do Z")
 *
 * Triggers are declarations evaluated by Game at specific points, NOT active
 * listeners. Cards declare triggers; Game evaluates them.
 *
 * Example: "Whenever this creature attacks, draw a card"
 * {
 *   eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
 *   condition: (game, event, source) =>
 *     event.creature.instanceId === source.instanceId,
 *   effect: (game, context) => game.drawCards(context.controllerId, 1)
 * }
 *
 * CRITICAL MVP LIMITATION:
 * - Triggers execute immediately (don't go on stack yet)
 * - TODO(stack): Create TriggeredAbilityOnStack instead
 * - TODO(stack): Allow players to respond before trigger resolves
 * - TODO(apnap): Implement APNAP ordering for simultaneous triggers
 *
 * Other limitations:
 * - No targeting, no optional triggers, no intervening-if clauses
 *
 * See ABILITY_CONTRACT_MVP.md for complete contract and evaluation rules.
 */
export type Trigger = {
  /** The type of event that activates this trigger */
  eventType: GameEvent["type"]

  /**
   * Pure predicate (no side effects) that determines if trigger fires
   * Can inspect game state, event details, and source card
   */
  condition: (game: Game, event: GameEvent, sourceCard: CardInstance) => boolean

  /**
   * Effect to execute when trigger fires
   * MVP: context.targets is always empty (no targeting yet)
   * TODO(targeting): Add target selection when trigger goes on stack
   */
  effect: (game: Game, context: EffectContext) => void
}

/**
 * Helper type for creating triggers with better type inference
 *
 * Usage:
 * const etbTrigger: TriggerDefinition<"ZONE_CHANGED"> = {
 *   eventType: "ZONE_CHANGED",
 *   condition: (game, event, source) => event.toZone === "BATTLEFIELD",
 *   effect: (game, context) => game.drawCards(context.controllerId, 1)
 * }
 */
export type TriggerDefinition<T extends GameEvent["type"]> = {
  eventType: T
  condition: (
    game: Game,
    event: Extract<GameEvent, { type: T }>,
    sourceCard: CardInstance,
  ) => boolean
  effect: (game: Game, context: EffectContext) => void
}
