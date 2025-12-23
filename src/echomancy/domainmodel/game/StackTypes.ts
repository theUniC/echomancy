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
 * Represents an activated ability on the stack.
 *
 * See abilities/Ability.ts for the full Ability contract.
 *
 * IMPORTANT: This is NOT a spell. Abilities:
 * - Do not move cards between zones
 * - Do not trigger ETB/LTB effects
 * - Are not affected by "counter target spell" effects (TODO: some spells can counter abilities)
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
 */
export type AbilityOnStack = {
  kind: "ABILITY"
  sourceId: string // permanentId of the card with the ability
  effect: Effect // Stored when activated for Last Known Information
  controllerId: string
  targets: Target[] // TODO: Implement targeting for abilities
}

/**
 * Represents a triggered ability on the stack.
 *
 * See abilities/Ability.ts for the full Ability contract.
 *
 * MVP CRITICAL LIMITATION:
 * This type is defined but NOT YET USED.
 * Triggered abilities currently execute immediately instead of going on the stack.
 *
 * TODO: Implement triggered abilities on stack:
 * 1. When a trigger fires, create TriggeredAbilityOnStack instead of executing immediately
 * 2. Add to stack before priority round
 * 3. Resolve via normal stack resolution
 * 4. Implement APNAP ordering for simultaneous triggers
 * 5. Allow players to respond to triggered abilities
 *
 * Once implemented, this will work like AbilityOnStack:
 * - sourceId: The permanent that has the trigger
 * - effect: The trigger's effect (captured for Last Known Information)
 * - controllerId: The player who controls the permanent
 * - targets: Target selection (future - requires targeting infrastructure)
 *
 * IMPORTANT: Unlike activated abilities, triggered abilities:
 * - Fire automatically when conditions are met
 * - Do not have a cost
 * - Use APNAP ordering when multiple trigger simultaneously
 * - May have intervening-if clauses (future)
 */
export type TriggeredAbilityOnStack = {
  kind: "TRIGGERED_ABILITY"
  sourceId: string // permanentId of the card with the trigger
  effect: (game: Game, context: EffectContext) => void // The trigger's effect
  controllerId: string
  targets: Target[] // TODO: Implement targeting for triggered abilities
}

/**
 * StackItem - Items that can be on the stack
 *
 * Currently supported:
 * - SpellOnStack: Cast spells (instant, sorcery, creature, etc.)
 * - AbilityOnStack: Activated abilities
 *
 * TODO: Add TriggeredAbilityOnStack to this union when implemented
 *
 * Resolution order: Last In, First Out (LIFO)
 */
export type StackItem = SpellOnStack | AbilityOnStack // | TriggeredAbilityOnStack (TODO)
