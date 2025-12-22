import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import type { GameSteps } from "@/echomancy/domainmodel/game/Steps"
import type { ZoneName } from "@/echomancy/domainmodel/zones/Zone"

/**
 * Game Events - Conceptual events emitted by the Game
 *
 * These are NOT event bus events or observables.
 * They are data structures representing "something that happened"
 * that the Game can use to evaluate which triggers should fire.
 *
 * The Game:
 * - detects that something happened
 * - constructs an event object
 * - inspects the current game state
 * - evaluates which triggers apply
 *
 * Cards declare triggers (conditions + effects), they do NOT
 * subscribe to events or maintain internal state.
 */

/**
 * ZoneChanged - A card moved from one zone to another
 *
 * Used for:
 * - ETB (hand → battlefield, stack → battlefield)
 * - Dies (battlefield → graveyard)
 * - Leaves battlefield (battlefield → any other zone)
 */
export type ZoneChangedEvent = {
  type: "ZONE_CHANGED"
  card: CardInstance
  fromZone: ZoneName
  toZone: ZoneName
  controllerId: string
}

/**
 * StepStarted - A new step/phase has begun
 *
 * Used for:
 * - Untap triggers
 * - "At the beginning of combat..." triggers
 * - "At the beginning of your upkeep..." triggers (future)
 */
export type StepStartedEvent = {
  type: "STEP_STARTED"
  step: GameSteps
  activePlayerId: string
}

/**
 * CreatureDeclaredAttacker - A creature was declared as an attacker
 *
 * Used for:
 * - "Whenever this creature attacks..." triggers
 * - "Whenever a creature attacks..." triggers
 */
export type CreatureDeclaredAttackerEvent = {
  type: "CREATURE_DECLARED_ATTACKER"
  creature: CardInstance
  controllerId: string
}

/**
 * CombatEnded - The combat phase has ended
 *
 * Used for:
 * - Reset of combat-related states
 * - "At end of combat..." triggers
 */
export type CombatEndedEvent = {
  type: "COMBAT_ENDED"
  activePlayerId: string
}

/**
 * SpellResolved - A spell finished resolving from the stack
 *
 * Used as a generic hook for post-resolution triggers.
 *
 * NOTE: This fires AFTER the spell's effect has been applied
 * and the card has been moved to its final zone.
 */
export type SpellResolvedEvent = {
  type: "SPELL_RESOLVED"
  card: CardInstance
  controllerId: string
}

/**
 * All game events
 */
export type GameEvent =
  | ZoneChangedEvent
  | StepStartedEvent
  | CreatureDeclaredAttackerEvent
  | CombatEndedEvent
  | SpellResolvedEvent
