import type { Effect } from "../effects/Effect"

export type SpellType =
  | "CREATURE"
  | "INSTANT"
  | "SORCERY"
  | "ARTIFACT"
  | "ENCHANTMENT"

export type LandDefinition = {
  id: string
  name: string
  category: "LAND"
}

export type SpellDefinition = {
  id: string
  name: string
  category: "SPELL"
  spellType: SpellType
  effect?: Effect
}

export type CardDefinition = LandDefinition | SpellDefinition
