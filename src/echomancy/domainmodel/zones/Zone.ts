/**
 * Zone types and constants.
 * @see docs/zones-and-cards.md
 */

import type { CardInstance } from "../cards/CardInstance"

export type Zone = {
  cards: CardInstance[]
}

export type ZoneName =
  | "HAND"
  | "BATTLEFIELD"
  | "GRAVEYARD"
  | "STACK"
  | "LIBRARY"
  | "EXILE"

export const ZoneNames = {
  HAND: "HAND" as const,
  BATTLEFIELD: "BATTLEFIELD" as const,
  GRAVEYARD: "GRAVEYARD" as const,
  STACK: "STACK" as const,
  LIBRARY: "LIBRARY" as const,
  EXILE: "EXILE" as const,
} as const
