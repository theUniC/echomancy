import type { Effect } from "../effects/Effect"

export type CardType = "LAND" | "SPELL"

export type CardDefinition = {
  id: string
  name: string
  type: CardType
  effect?: Effect
}
