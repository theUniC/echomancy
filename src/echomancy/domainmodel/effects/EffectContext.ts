import type { CardInstance } from "../cards/CardInstance"

export type EffectContext = {
  source: CardInstance
  controllerId: string
}
