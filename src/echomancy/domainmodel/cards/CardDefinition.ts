/**
 * Card definition types.
 * @see docs/zones-and-cards.md
 */

import type { ActivatedAbility } from "../abilities/ActivatedAbility"
import type { Effect } from "../effects/Effect"
import type { Trigger } from "../triggers/Trigger"

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
  activatedAbility?: ActivatedAbility
  triggers?: Trigger[]
}
