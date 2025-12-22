import type { Game } from "@/echomancy/domainmodel/game/Game"
import type { GameEvent } from "@/echomancy/domainmodel/game/GameEvents"
import type { EffectContext } from "@/echomancy/domainmodel/effects/EffectContext"

/**
 * Trigger - A declarative triggered ability
 *
 * IMPORTANT: Triggers are NOT active listeners or observers.
 * They are DECLARATIONS of "when X happens, if Y is true, do Z".
 *
 * The Game is responsible for:
 * - Detecting when events occur
 * - Evaluating which triggers match
 * - Executing the effects in the correct order
 *
 * Cards declare triggers but do NOT execute logic actively.
 *
 * ---
 *
 * Structure:
 * - eventType: Which type of event this trigger watches for
 * - condition: A predicate that determines if the trigger fires
 *              Can inspect the full game state and event details
 * - effect: The effect to execute when the trigger fires
 *           Receives the game instance and execution context
 *
 * ---
 *
 * Example (conceptual):
 *
 * "Whenever this creature attacks, draw a card"
 *
 * {
 *   eventType: "CREATURE_DECLARED_ATTACKER",
 *   condition: (game, event, sourceCard) =>
 *     event.creature.instanceId === sourceCard.instanceId,
 *   effect: (game, context) =>
 *     game.drawCards(context.controllerId, 1)
 * }
 *
 * ---
 *
 * MVP Limitations (documented with TODOs):
 * - Triggers execute immediately (no separate trigger stack)
 * - No APNAP ordering (Active Player, Non-Active Player)
 * - No complex targeting in trigger effects
 * - No intervening-if clauses
 * - No duration tracking ("until end of turn")
 */
export type Trigger = {
  /**
   * The type of event that can activate this trigger
   */
  eventType: GameEvent["type"]

  /**
   * Predicate that determines if this trigger should fire
   *
   * @param game - The current game state (read-only)
   * @param event - The event that occurred
   * @param sourceCard - The card that owns this trigger
   * @returns true if the trigger should fire, false otherwise
   *
   * The condition can inspect:
   * - The event details (which card, which player, etc.)
   * - The source card (is this MY card that triggered?)
   * - The full game state (how many creatures are on battlefield?)
   *
   * NOTE: The condition should be pure and have no side effects.
   */
  condition: (game: Game, event: GameEvent, sourceCard: any) => boolean

  /**
   * The effect to execute when the trigger fires
   *
   * @param game - The game instance (can mutate state)
   * @param context - The execution context
   *
   * The effect receives:
   * - game: full access to mutate game state
   * - context: includes source card, controller, targets
   *
   * The effect can do anything a normal Effect can do:
   * - Draw cards
   * - Deal damage (future)
   * - Create tokens (future)
   * - Modify game state
   *
   * TODO: For MVP, trigger effects do NOT support targeting.
   * The context.targets array will be empty.
   * Targeting in triggers requires additional infrastructure
   * (choosing targets when trigger goes on stack, etc.)
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
  condition: (game: Game, event: Extract<GameEvent, { type: T }>, sourceCard: any) => boolean
  effect: (game: Game, context: EffectContext) => void
}
