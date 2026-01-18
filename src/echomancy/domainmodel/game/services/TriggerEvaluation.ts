/**
 * TriggerEvaluation Domain Service
 *
 * Stateless service that evaluates triggered abilities.
 * Identifies which triggers should fire in response to game events.
 *
 * Current implementation (MVP):
 * - Scans all permanents on all battlefields
 * - Checks trigger conditions against the event
 * - Returns triggered abilities ready to execute
 *
 * MVP Limitations:
 * - Triggers execute immediately (not on stack)
 * - No APNAP ordering (active player, non-active player)
 * - No targeting in trigger effects
 *
 * @example
 * const triggers = TriggerEvaluation.findMatchingTriggers(game, event)
 * for (const trigger of triggers) {
 *   trigger.effect(game, { source: trigger.source, controllerId: trigger.controllerId, targets: [] })
 * }
 */

import type { CardInstance } from "../../cards/CardInstance"
import type { EffectContext } from "../../effects/EffectContext"
import type { Game } from "../Game"
import type { GameEvent } from "../GameEvents"

/**
 * Represents a permanent on the battlefield with its controller.
 */
export type PermanentOnBattlefield = {
  permanent: CardInstance
  controllerId: string
}

/**
 * Represents a triggered ability ready to execute.
 */
export type TriggeredAbilityInfo = {
  /** The effect function to execute */
  effect: (game: Game, context: EffectContext) => void
  /** The player who controls the triggered ability */
  controllerId: string
  /** The permanent that is the source of the trigger */
  source: CardInstance
}

/**
 * Collects all permanents from all players' battlefields.
 *
 * @param game - The game to collect permanents from
 * @returns Array of permanents with their controller IDs
 */
export function collectPermanentsFromBattlefield(
  game: Game,
): PermanentOnBattlefield[] {
  const permanents: PermanentOnBattlefield[] = []

  for (const playerId of game.getPlayersInTurnOrder()) {
    const playerState = game.getPlayerState(playerId)
    for (const permanent of playerState.battlefield.cards) {
      permanents.push({ permanent, controllerId: playerId })
    }
  }

  return permanents
}

/**
 * Finds all triggers that match the given event.
 *
 * This is a pure function that queries the game state and returns
 * a list of triggered abilities. It does NOT execute the triggers.
 *
 * Filters permanents to find triggers that:
 * 1. Watch for this event type
 * 2. Have their condition met
 *
 * @param game - The game to evaluate triggers in
 * @param event - The game event that may trigger abilities
 * @returns Array of triggered abilities ready to execute
 */
export function findMatchingTriggers(
  game: Game,
  event: GameEvent,
): TriggeredAbilityInfo[] {
  const permanents = collectPermanentsFromBattlefield(game)
  const triggersToExecute: TriggeredAbilityInfo[] = []

  for (const { permanent, controllerId } of permanents) {
    const triggers = permanent.definition.triggers
    if (!triggers) continue

    for (const trigger of triggers) {
      // Skip if event type doesn't match (avoid evaluating condition unnecessarily)
      if (trigger.eventType !== event.type) continue

      // Evaluate condition only for matching event types
      const isConditionMet = trigger.condition(game, event, permanent)

      if (isConditionMet) {
        triggersToExecute.push({
          effect: trigger.effect,
          controllerId,
          source: permanent,
        })
      }
    }
  }

  return triggersToExecute
}

/**
 * TriggerEvaluation namespace for organized service methods.
 */
export const TriggerEvaluation = {
  collectPermanentsFromBattlefield,
  findMatchingTriggers,
} as const
