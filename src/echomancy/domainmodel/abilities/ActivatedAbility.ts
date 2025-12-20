import type { Effect } from "../effects/Effect"

/**
 * Represents the cost to activate an ability.
 *
 * MVP LIMITATION - Only {T} (tap) cost is currently supported.
 * TODO: Add support for:
 * - Mana costs (e.g., {2}{R})
 * - Other tap costs (e.g., "Tap another creature you control")
 * - Sacrifice costs
 * - Discard costs
 * - Life payment costs
 * - Multiple costs combined
 * - X costs
 */
export type ActivationCost = {
  type: "TAP"
  // TODO: Add other cost types as the engine evolves
}

/**
 * Represents an activated ability that can be activated by a player.
 *
 * MVP LIMITATIONS:
 * - No targeting support (abilities resolve without targets)
 * - No timing restrictions beyond priority (e.g., "Activate only during combat")
 * - No mana costs
 * - All abilities can be activated at instant speed (when you have priority)
 *
 * TODO: Future enhancements:
 * - Add targeting requirements
 * - Add timing restrictions (sorcery speed, combat only, etc.)
 * - Add mana costs
 * - Add loyalty costs (for planeswalkers)
 * - Add additional costs beyond tap
 */
export type ActivatedAbility = {
  cost: ActivationCost
  effect: Effect
  // TODO: Add targets requirement when targeting is implemented
  // TODO: Add timing restrictions (e.g., sorcerySpeed: boolean)
}
