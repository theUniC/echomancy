import type { CardInstance } from "../cards/CardInstance"
import type { Target } from "../targets/Target"

/**
 * EffectContext - Execution context for ability effects
 *
 * Contains all information needed to execute an ability's effect:
 * - controllerId (required): The player who controls this ability
 * - source (optional): The card with this ability (may be undefined if it left battlefield)
 * - targets (optional): Selected targets (MVP: always empty, no targeting yet)
 *
 * IMPORTANT:
 * - Uses Last Known Information (source captures card state at activation)
 * - ETB abilities DO NOT reuse spell targets
 * - Context is captured when ability is activated/triggered
 *
 * TODO(targeting): Add target selection and validation
 * TODO(targeting): Implement illegal target handling
 *
 * See ABILITY_CONTRACT_MVP.md for complete details.
 */
export type EffectContext = {
  /** Card with this ability (undefined if left battlefield - Last Known Information) */
  source?: CardInstance

  /** Player who controls this ability (always present) */
  controllerId: string

  /** Selected targets (MVP: always empty - TODO(targeting): implement) */
  targets: Target[]
}
