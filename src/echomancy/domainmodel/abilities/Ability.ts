/**
 * Ability - Type definitions for the ability system
 *
 * An Ability is a rule unit attached to a card that produces effects when
 * specific conditions are met. Abilities are DECLARATIVE - they don't execute
 * actively, but are evaluated by the Game at specific points.
 *
 * SUPPORTED TYPES:
 * - ActivatedAbility: Player-activated with costs (e.g., "{T}: Draw a card")
 * - Trigger: Event-based (e.g., "When ~ enters, draw a card")
 *
 * NOT SUPPORTED (out of MVP scope):
 * - StaticAbility, ReplacementEffect, PreventionEffect, ManaAbility
 *
 * CRITICAL MVP LIMITATION:
 * - Triggered abilities execute immediately (don't go on stack yet)
 * - TODO(stack): Implement TriggeredAbilityOnStack
 *
 * For complete ability contract, architecture, evaluation rules, and
 * implementation guidelines, see ABILITY_CONTRACT_MVP.md
 */

import type { Trigger } from "../triggers/Trigger"
import type { ActivatedAbility } from "./ActivatedAbility"

/**
 * Union type for all ability types in the system.
 *
 * Currently includes:
 * - ActivatedAbility (defined in ActivatedAbility.ts)
 * - Trigger (defined in triggers/Trigger.ts)
 *
 * Future additions:
 * - StaticAbility (TODO)
 * - ManaAbility (TODO)
 */
export type Ability = ActivatedAbility | Trigger

/**
 * Type guard to check if an ability is an ActivatedAbility
 */
export function isActivatedAbility(
  ability: Ability,
): ability is ActivatedAbility {
  return "cost" in ability && "effect" in ability
}

/**
 * Type guard to check if an ability is a Trigger
 */
export function isTrigger(ability: Ability): ability is Trigger {
  return "eventType" in ability && "condition" in ability
}
