import type { CardInstance } from "../cards/CardInstance"

export type Zone = {
  cards: CardInstance[]
}

/**
 * Zone names in the game
 */
export type ZoneName = "HAND" | "BATTLEFIELD" | "GRAVEYARD" | "STACK" | "LIBRARY" | "EXILE"
