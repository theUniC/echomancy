import type { CardInstance } from "../cards/CardInstance"
import type { Target } from "../targets/Target"

export type EffectContext = {
  source: CardInstance
  controllerId: string
  targets: Target[]
}
