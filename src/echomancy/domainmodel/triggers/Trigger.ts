import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import type { EffectContext } from "@/echomancy/domainmodel/effects/EffectContext"
import type { Game } from "@/echomancy/domainmodel/game/Game"
import type { GameEvent } from "@/echomancy/domainmodel/game/GameEvents"

/**
 * TRIGGERED ABILITY – MVP Implementation
 *
 * See abilities/Ability.ts for the full Ability contract.
 *
 * ============================================================================
 * DEFINITION
 * ============================================================================
 *
 * A Trigger is a declarative triggered ability that represents:
 * "When X happens, if Y is true, do Z"
 *
 * IMPORTANT: Triggers are NOT active listeners or observers.
 * They are DECLARATIONS that the Game evaluates at specific points.
 *
 * The Game is responsible for:
 * - Detecting when events occur
 * - Evaluating which triggers match
 * - Executing the effects in the correct order
 *
 * Cards declare triggers but do NOT execute logic actively.
 *
 * ============================================================================
 * STRUCTURE
 * ============================================================================
 *
 * - eventType: Which type of event this trigger watches for
 *              Must be one of GameEvent["type"]
 * - condition: A predicate that determines if the trigger fires
 *              Can inspect the full game state and event details
 *              MUST be pure (no side effects)
 * - effect: The effect to execute when the trigger fires
 *           Receives the game instance and execution context
 *           Can mutate game state via game.apply() and other Game methods
 *
 * ============================================================================
 * EXAMPLE
 * ============================================================================
 *
 * "Whenever this creature attacks, draw a card"
 *
 * {
 *   eventType: GameEventTypes.CREATURE_DECLARED_ATTACKER,
 *   condition: (game, event, sourceCard) =>
 *     event.creature.instanceId === sourceCard.instanceId,
 *   effect: (game, context) =>
 *     game.drawCards(context.controllerId, 1)
 * }
 *
 * ============================================================================
 * WHEN TRIGGERS ARE EVALUATED
 * ============================================================================
 *
 * Triggers are evaluated ONLY at these specific points:
 * 1. After enterBattlefield() → ZONE_CHANGED event
 * 2. After declareAttacker() → CREATURE_DECLARED_ATTACKER event
 * 3. After resolveSpell() → SPELL_RESOLVED event
 * 4. On step transition → STEP_STARTED, COMBAT_ENDED events
 *
 * The Game calls evaluateTriggers(event) at these points.
 * Triggers are NEVER evaluated continuously or reactively.
 *
 * See abilities/Ability.ts section 4 for complete evaluation rules.
 *
 * ============================================================================
 * MVP LIMITATIONS
 * ============================================================================
 *
 * The following are NOT supported yet:
 *
 * CRITICAL LIMITATION - Triggers execute immediately:
 * - TODO: Triggers should create StackItem and go on the stack
 * - TODO: Players should be able to respond to triggered abilities
 * - TODO: Implement APNAP ordering for simultaneous triggers
 *
 * Other limitations:
 * - No targeting in trigger effects (targets array always empty)
 * - No intervening-if clauses ("whenever X, if Y, ..." where Y is checked on resolution)
 * - No duration tracking ("until end of turn")
 * - No delayed triggered abilities ("at the beginning of the next end step")
 * - No optional triggers ("you may...")
 * - No replacement effects (not technically triggers)
 * - No state-based triggers (triggers based on continuous state)
 *
 * See abilities/Ability.ts section 7 for complete non-goals list.
 *
 * ============================================================================
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
  condition: (game: Game, event: GameEvent, sourceCard: CardInstance) => boolean

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
  condition: (
    game: Game,
    event: Extract<GameEvent, { type: T }>,
    sourceCard: CardInstance,
  ) => boolean
  effect: (game: Game, context: EffectContext) => void
}
