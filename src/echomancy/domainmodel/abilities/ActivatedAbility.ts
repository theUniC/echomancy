import type { Effect } from "../effects/Effect"

/**
 * ACTIVATED ABILITY – MVP Implementation
 *
 * See abilities/Ability.ts for the full Ability contract.
 *
 * ============================================================================
 * DEFINITION
 * ============================================================================
 *
 * An ActivatedAbility is a rule that allows a player to explicitly activate
 * an effect by paying a cost.
 *
 * Example: "{T}: Draw a card"
 * - cost: Tap this permanent
 * - effect: Draw a card
 *
 * ============================================================================
 * ACTIVATION FLOW
 * ============================================================================
 *
 * 1. Player uses ACTIVATE_ABILITY action (requires priority)
 * 2. Game validates:
 *    - Player controls the permanent
 *    - Player has priority
 *    - Cost can be paid
 * 3. Game pays the cost (e.g., taps the permanent)
 * 4. Game creates AbilityOnStack item
 * 5. Ability resolves via normal stack resolution
 *
 * ============================================================================
 * STACK BEHAVIOR
 * ============================================================================
 *
 * ActivatedAbilities ALWAYS go on the stack:
 * - Creates an AbilityOnStack item
 * - Resolves LIFO (Last In, First Out)
 * - Uses Last Known Information (source can leave battlefield)
 * - Players can respond with other spells/abilities
 *
 * ============================================================================
 * MVP LIMITATIONS
 * ============================================================================
 *
 * The following are NOT supported yet:
 * - Mana costs (only {T} is supported)
 * - Targeting (targets array always empty)
 * - Timing restrictions (e.g., "Activate only during combat")
 * - Loyalty abilities (planeswalkers)
 * - Multiple costs combined (e.g., "{T}, Pay 2 life")
 * - X costs
 * - Alternative costs
 * - Cost reduction
 *
 * See abilities/Ability.ts section 7 for complete non-goals list.
 *
 * ============================================================================
 */

/**
 * Represents the cost to activate an ability.
 *
 * MVP LIMITATION - Only {T} (tap) cost is currently supported.
 *
 * TODO: Add support for:
 * - Mana costs (e.g., {2}{R})
 * - Other tap costs (e.g., "Tap another creature you control")
 * - Sacrifice costs (e.g., "Sacrifice a creature")
 * - Discard costs (e.g., "Discard a card")
 * - Life payment costs (e.g., "Pay 2 life")
 * - Multiple costs combined (e.g., "{2}{R}, {T}, Sacrifice a creature")
 * - X costs (e.g., "{X}: Draw X cards")
 */
export type ActivationCost = {
  type: "TAP"
  // TODO: Add other cost types as the engine evolves
}

/**
 * Represents an activated ability that can be activated by a player.
 *
 * An ActivatedAbility consists of:
 * - cost: What the player must pay to activate
 * - effect: What happens when the ability resolves
 *
 * MVP LIMITATIONS:
 * - No targeting support (abilities resolve without targets)
 * - No timing restrictions beyond priority (e.g., "Activate only during combat")
 * - No mana costs (only tap cost supported)
 * - All abilities can be activated at instant speed (when you have priority)
 *
 * TODO: Future enhancements:
 * - Add targeting requirements (define valid targets, number of targets)
 * - Add timing restrictions (sorcery speed, combat only, main phase only, etc.)
 * - Add mana costs to ActivationCost union
 * - Add loyalty costs (for planeswalkers)
 * - Add additional cost types (sacrifice, discard, life payment, etc.)
 * - Add cost reduction effects
 */
export type ActivatedAbility = {
  cost: ActivationCost
  effect: Effect
  // TODO: Add targets requirement when targeting is implemented
  // targets?: TargetRequirement (number, validTarget predicate, etc.)
  // TODO: Add timing restrictions (e.g., sorcerySpeed: boolean)
}
