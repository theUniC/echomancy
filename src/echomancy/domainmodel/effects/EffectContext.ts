/**
 * EffectContext - Execution context for ability effects.
 * @see docs/effect-system.md
 */

import type { CardInstance } from "../cards/CardInstance"
import type { Target } from "../targets/Target"

export type EffectContext = {
  /** Card with this ability (uses Last Known Information) */
  source?: CardInstance
  controllerId: string
  /** MVP: always empty */
  targets: Target[]
}
