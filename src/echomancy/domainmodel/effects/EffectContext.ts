import type { CardInstance } from "../cards/CardInstance"
import type { Target } from "../targets/Target"

/**
 * EFFECT CONTEXT – Ability Execution Context
 *
 * See abilities/Ability.ts for the full Ability contract.
 *
 * ============================================================================
 * DEFINITION
 * ============================================================================
 *
 * EffectContext contains all the information needed to execute an ability's
 * effect. This context is captured when the ability is activated or triggered,
 * and includes Last Known Information if the source leaves the battlefield.
 *
 * ============================================================================
 * FIELDS
 * ============================================================================
 *
 * REQUIRED:
 * - controllerId: The player who controls this ability
 *                 Always present, determines who makes choices and pays costs
 *
 * OPTIONAL:
 * - source: The card/permanent that has this ability
 *           May be undefined if:
 *           - The source has left the battlefield (Last Known Information)
 *           - The effect is not tied to a specific card (rare)
 *           - The effect outlived its source (delayed triggers, future)
 *
 * - targets: Array of targets chosen for this ability
 *            MVP LIMITATION: Usually empty (targeting not fully implemented)
 *            When targeting is implemented, this will contain:
 *            - Selected cards/players/zones
 *            - Validated at time of activation/trigger
 *            - Uses Last Known Information at resolution
 *
 * ============================================================================
 * IMPORTANT NOTES
 * ============================================================================
 *
 * ETB Abilities and Spell Targets:
 * - ETB abilities DO NOT reuse spell targets
 * - Each ability must define its own targeting requirements (future)
 * - When a spell resolves and becomes a permanent, its targets are NOT
 *   passed to the ETB trigger
 *
 * Last Known Information:
 * - If source permanent leaves the battlefield, the ability still resolves
 * - The context.source captures the card state when ability was activated/triggered
 * - Effects can use this information even if the card is gone
 *
 * Targeting (Future):
 * - TODO: When targeting is implemented, targets will be selected when:
 *   - Activated ability is activated (player chooses targets)
 *   - Triggered ability goes on stack (player chooses targets if required)
 * - TODO: Targets will be validated on resolution (illegal target rules)
 *
 * ============================================================================
 * MVP LIMITATIONS
 * ============================================================================
 *
 * The following are NOT supported yet:
 * - Targeting in abilities (targets array always empty)
 * - Target validation on resolution
 * - "Illegal target" handling
 * - Multiple target selection
 * - Distributing effects among targets (e.g., "distribute 3 damage")
 * - Modal abilities (choosing which effect to execute)
 * - X values or variable parameters
 *
 * See abilities/Ability.ts section 7 for complete non-goals list.
 *
 * ============================================================================
 */
export type EffectContext = {
  /**
   * The card/permanent that has this ability.
   *
   * May be undefined if the source has left the battlefield or
   * the effect is not tied to a specific card.
   *
   * Uses Last Known Information if the source is no longer on battlefield.
   */
  source?: CardInstance

  /**
   * The player who controls this ability.
   *
   * Always present. This determines who makes choices, who pays costs,
   * and who benefits from effects like "you draw a card".
   */
  controllerId: string

  /**
   * Array of targets chosen for this ability.
   *
   * MVP LIMITATION: Usually empty (targeting not fully implemented).
   *
   * TODO: When targeting is implemented, this will contain the selected
   * targets (cards, players, zones, etc.) validated at activation time
   * and re-validated at resolution.
   */
  targets: Target[]
}
