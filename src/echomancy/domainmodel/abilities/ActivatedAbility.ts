/**
 * ActivatedAbility - Player-activated abilities with costs.
 * @see docs/ability-system.md
 */

import type { Effect } from "../effects/Effect"

/** MVP: Only TAP cost supported */
export type ActivationCost = {
  type: "TAP"
}

export type ActivatedAbility = {
  cost: ActivationCost
  effect: Effect
}
