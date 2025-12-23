import type { Effect } from "../effects/Effect"

/**
 * ActivatedAbility - Player-activated abilities with costs
 *
 * Player explicitly activates by paying a cost. Always goes on stack.
 * Example: "{T}: Draw a card"
 *
 * MVP Limitations:
 * - Only {T} cost supported (no mana, life, sacrifice, etc.)
 * - No targeting
 * - No timing restrictions (can activate any time with priority)
 *
 * See ABILITY_CONTRACT_MVP.md for complete ability contract.
 */

/**
 * Represents the cost to activate an ability.
 *
 * MVP: Only {T} (tap) cost supported.
 *
 * TODO(costs): Add mana costs, sacrifice, discard, life payment
 * TODO(costs): Add multiple costs combined
 * TODO(costs): Add X costs
 */
export type ActivationCost = {
  type: "TAP"
}

/**
 * An activated ability with a cost and effect.
 *
 * TODO(targeting): Add targeting requirements
 * TODO(timing): Add timing restrictions (sorcery speed, combat only, etc.)
 * TODO(costs): Add loyalty costs for planeswalkers
 */
export type ActivatedAbility = {
  cost: ActivationCost
  effect: Effect
}
