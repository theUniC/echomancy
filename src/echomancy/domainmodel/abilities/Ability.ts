/**
 * Ability - Union type for all ability types.
 * @see docs/ability-system.md
 */

import type { Trigger } from "../triggers/Trigger"
import type { ActivatedAbility } from "./ActivatedAbility"

export type Ability = ActivatedAbility | Trigger

export function isActivatedAbility(
  ability: Ability,
): ability is ActivatedAbility {
  return "cost" in ability && "effect" in ability
}

export function isTrigger(ability: Ability): ability is Trigger {
  return "eventType" in ability && "condition" in ability
}
