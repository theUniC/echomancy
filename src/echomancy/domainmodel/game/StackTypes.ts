import type { CardInstance } from "../cards/CardInstance"
import type { Effect } from "../effects/Effect"
import type { Target } from "../targets/Target"

export type SpellOnStack = {
  kind: "SPELL"
  card: CardInstance
  controllerId: string
  targets: Target[]
}

/**
 * Represents an activated ability on the stack.
 *
 * IMPORTANT: This is NOT a spell. Abilities:
 * - Do not move cards between zones
 * - Do not trigger ETB/LTB effects
 * - Are not affected by "counter target spell" effects
 * - Come from permanents on the battlefield
 * - Resolve independently once on stack (Last Known Information)
 *
 * The effect is stored when activated so the ability can resolve
 * even if the source permanent leaves the battlefield.
 *
 * MVP LIMITATIONS:
 * - No targeting support (targets array always empty)
 * - Only supports permanents as sources (not emblems, etc.)
 *
 * TODO: Add support for:
 * - Targeting in abilities
 * - Mana abilities (special rules, don't use stack)
 * - Loyalty abilities (planeswalkers)
 * - Triggered abilities (separate from activated)
 */
export type AbilityOnStack = {
  kind: "ABILITY"
  sourceId: string // permanentId of the card with the ability
  effect: Effect // Stored when activated for Last Known Information
  controllerId: string
  targets: Target[] // TODO: Implement targeting for abilities
}

export type StackItem = SpellOnStack | AbilityOnStack
