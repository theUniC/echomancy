/**
 * Game Events - Data structures for trigger evaluation.
 * @see docs/game-events.md
 */

import type { CardInstance } from "@/echomancy/domainmodel/cards/CardInstance"
import type { GameSteps } from "@/echomancy/domainmodel/game/Steps"
import type { ZoneName } from "@/echomancy/domainmodel/zones/Zone"

export type ZoneChangedEvent = {
  type: "ZONE_CHANGED"
  card: CardInstance
  fromZone: ZoneName
  toZone: ZoneName
  controllerId: string
}

export type StepStartedEvent = {
  type: "STEP_STARTED"
  step: GameSteps
  activePlayerId: string
}

export type CreatureDeclaredAttackerEvent = {
  type: "CREATURE_DECLARED_ATTACKER"
  creature: CardInstance
  controllerId: string
}

export type CombatEndedEvent = {
  type: "COMBAT_ENDED"
  activePlayerId: string
}

export type SpellResolvedEvent = {
  type: "SPELL_RESOLVED"
  card: CardInstance
  controllerId: string
}

export type GameEvent =
  | ZoneChangedEvent
  | StepStartedEvent
  | CreatureDeclaredAttackerEvent
  | CombatEndedEvent
  | SpellResolvedEvent

export const GameEventTypes = {
  ZONE_CHANGED: "ZONE_CHANGED" as const,
  STEP_STARTED: "STEP_STARTED" as const,
  CREATURE_DECLARED_ATTACKER: "CREATURE_DECLARED_ATTACKER" as const,
  COMBAT_ENDED: "COMBAT_ENDED" as const,
  SPELL_RESOLVED: "SPELL_RESOLVED" as const,
} as const
