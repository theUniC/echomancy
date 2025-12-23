import type { CardInstance } from "../cards/CardInstance"
import type { Effect } from "../effects/Effect"
import type { EffectContext } from "../effects/EffectContext"
import type { Target } from "../targets/Target"
import type { Game } from "./Game"

export type SpellOnStack = {
  kind: "SPELL"
  card: CardInstance
  controllerId: string
  targets: Target[]
}

/**
 * Activated ability on the stack
 *
 * NOT a spell - doesn't move cards, doesn't trigger ETB/LTB.
 * Uses Last Known Information (resolves even if source leaves battlefield).
 *
 * TODO(targeting): Add targeting support
 * TODO(costs): Add mana abilities (don't use stack)
 * TODO(planeswalkers): Add loyalty abilities
 */
export type AbilityOnStack = {
  kind: "ABILITY"
  sourceId: string
  effect: Effect
  controllerId: string
  targets: Target[]
}

/**
 * Triggered ability on the stack
 *
 * CRITICAL: This type is defined but NOT YET USED.
 * MVP: Triggered abilities execute immediately instead of going on stack.
 *
 * TODO(stack): Create TriggeredAbilityOnStack when trigger fires (not execute immediately)
 * TODO(stack): Add to stack before priority round
 * TODO(apnap): Implement APNAP ordering for simultaneous triggers
 * TODO(targeting): Add target selection for triggered abilities
 *
 * Unlike activated abilities: fire automatically, no cost, use APNAP ordering
 */
export type TriggeredAbilityOnStack = {
  kind: "TRIGGERED_ABILITY"
  sourceId: string
  effect: Effect
  controllerId: string
  targets: Target[]
}

/**
 * Items that can be on the stack (resolution order: LIFO)
 *
 * Currently: SpellOnStack, AbilityOnStack
 * TODO(stack): Add TriggeredAbilityOnStack when implemented
 */
export type StackItem = SpellOnStack | AbilityOnStack
