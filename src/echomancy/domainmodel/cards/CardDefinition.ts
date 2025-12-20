import type { Effect } from "../effects/Effect"

export type CardType =
  | "CREATURE"
  | "INSTANT"
  | "SORCERY"
  | "ARTIFACT"
  | "ENCHANTMENT"
  | "LAND"

export type CardDefinition = {
  id: string
  name: string
  types: CardType[]
  effect?: Effect
}
