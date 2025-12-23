/**
 * Stack item types for spells and abilities.
 * @see docs/stack-and-priority.md
 */

import type { CardInstance } from "../cards/CardInstance"
import type { Effect } from "../effects/Effect"
import type { Target } from "../targets/Target"

export type SpellOnStack = {
  kind: "SPELL"
  card: CardInstance
  controllerId: string
  targets: Target[]
}

export type AbilityOnStack = {
  kind: "ABILITY"
  sourceId: string
  effect: Effect
  controllerId: string
  targets: Target[]
}

/** Defined but not yet used - triggers execute immediately in MVP */
export type TriggeredAbilityOnStack = {
  kind: "TRIGGERED_ABILITY"
  sourceId: string
  effect: Effect
  controllerId: string
  targets: Target[]
}

export type StackItem = SpellOnStack | AbilityOnStack
