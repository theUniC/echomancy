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

/**
 * Static ability keywords supported by the game engine.
 *
 * MVP includes only Flying, Reach, and Vigilance.
 * These are consultative keywords that affect rule checks only.
 *
 * Static abilities are:
 * - Always-on (do not go on the stack)
 * - Consultative (only affect validations and rule checks)
 * - Local (no global expert system)
 *
 * TODO: Future expansion:
 * - First strike / Double strike
 * - Trample
 * - Deathtouch
 * - Lifelink
 * - Menace
 * - Hexproof / Shroud
 * - Protection
 * - Ward
 * - Continuous effects / lords ("other elves get +1/+1")
 * - 7-layer system for complex dependencies
 * - Ability gain/loss ("creature gains flying until end of turn")
 * - Replacement effects
 */
export type StaticAbility = "FLYING" | "REACH" | "VIGILANCE" | "HASTE" | "FLASH"

/**
 * Static ability keyword constants.
 * Use these constants instead of string literals to avoid magic strings.
 */
export const StaticAbilities = {
  FLYING: "FLYING" as const,
  REACH: "REACH" as const,
  VIGILANCE: "VIGILANCE" as const,
  HASTE: "HASTE" as const,
  FLASH: "FLASH" as const,
} satisfies Record<string, StaticAbility>

export type CardDefinition = {
  readonly id: string
  readonly name: string
  readonly types: readonly CardType[]
  readonly effect?: Effect
  readonly activatedAbility?: ActivatedAbility
  readonly triggers?: readonly Trigger[]
  /**
   * Static ability keywords.
   * Only applicable to permanents (primarily creatures).
   * Default: empty (no static abilities)
   *
   * @see StaticAbility for supported keywords
   */
  readonly staticAbilities?: readonly StaticAbility[]
  /**
   * Base power for creatures.
   * Only applicable when types includes "CREATURE".
   * Default: 0
   */
  readonly power?: number
  /**
   * Base toughness for creatures.
   * Only applicable when types includes "CREATURE".
   * Default: 1 (minimum viable creature)
   */
  readonly toughness?: number
}
