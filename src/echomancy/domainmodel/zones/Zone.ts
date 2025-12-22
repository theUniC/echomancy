import type { CardInstance } from "../cards/CardInstance"

export type Zone = {
  cards: CardInstance[]
}

/**
 * Zone names in the game
 */
export type ZoneName =
  | "HAND"
  | "BATTLEFIELD"
  | "GRAVEYARD"
  | "STACK"
  | "LIBRARY"
  | "EXILE"

/**
 * Zone constants to avoid magic strings
 *
 * Usage: Zone.BATTLEFIELD instead of "BATTLEFIELD"
 */
export const ZoneNames = {
  HAND: "HAND" as const,
  BATTLEFIELD: "BATTLEFIELD" as const,
  GRAVEYARD: "GRAVEYARD" as const,
  STACK: "STACK" as const,
  LIBRARY: "LIBRARY" as const,
  EXILE: "EXILE" as const,
} as const
