import type { Effect } from "../effects/Effect"

export type CardType = "LAND" | "SPELL" | "CREATURE"

export type CardDefinition = {
  id: string
  name: string
  type: CardType
  effect?: Effect
}
