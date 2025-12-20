import type { ActivatedAbility } from "../abilities/ActivatedAbility"
import type { Effect } from "../effects/Effect"

export type CardType =
  | "CREATURE"
  | "INSTANT"
  | "SORCERY"
  | "ARTIFACT"
  | "ENCHANTMENT"
  | "PLANESWALKER"
  | "LAND"

export type CardDefinition = {
  id: string
  name: string
  types: CardType[]
  effect?: Effect
  onEnterBattlefield?: Effect
  activatedAbility?: ActivatedAbility
}
