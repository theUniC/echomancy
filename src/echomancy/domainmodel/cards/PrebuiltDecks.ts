/**
 * Prebuilt Decks
 *
 * Factory functions for creating complete 60-card deck configurations.
 * Each function generates a new set of CardInstance objects with unique IDs.
 *
 * MVP Decks:
 * - Green Deck: 24 Forest, 20 Bear, 16 Giant Growth
 * - Red Deck: 24 Mountain, 20 Elite Vanguard, 16 Lightning Strike
 *
 * These decks are used for game setup and testing.
 * Each card becomes a unique game object (CardInstance) with its own instanceId.
 *
 * @see CardCatalog for card definitions
 */

import { v4 as uuidv4 } from "uuid"
import { CardCatalog } from "./CardCatalog"
import type { CardInstance } from "./CardInstance"

/**
 * Creates a green deck (60 cards) for the specified player.
 *
 * Composition:
 * - 24x Forest
 * - 20x Bear (2/2)
 * - 16x Giant Growth (instant)
 *
 * @param ownerId - The player who owns this deck
 * @returns Array of 60 CardInstance objects with unique IDs
 */
function greenDeck(ownerId: string): CardInstance[] {
  const deck: CardInstance[] = []

  // Add 24 Forests
  for (let i = 0; i < 24; i++) {
    deck.push({
      instanceId: uuidv4(),
      definition: CardCatalog.Forest,
      ownerId,
    })
  }

  // Add 20 Bears
  for (let i = 0; i < 20; i++) {
    deck.push({
      instanceId: uuidv4(),
      definition: CardCatalog.Bear,
      ownerId,
    })
  }

  // Add 16 Giant Growths
  for (let i = 0; i < 16; i++) {
    deck.push({
      instanceId: uuidv4(),
      definition: CardCatalog.GiantGrowth,
      ownerId,
    })
  }

  return deck
}

/**
 * Creates a red deck (60 cards) for the specified player.
 *
 * Composition:
 * - 24x Mountain
 * - 20x Elite Vanguard (2/1)
 * - 16x Lightning Strike (instant)
 *
 * @param ownerId - The player who owns this deck
 * @returns Array of 60 CardInstance objects with unique IDs
 */
function redDeck(ownerId: string): CardInstance[] {
  const deck: CardInstance[] = []

  // Add 24 Mountains
  for (let i = 0; i < 24; i++) {
    deck.push({
      instanceId: uuidv4(),
      definition: CardCatalog.Mountain,
      ownerId,
    })
  }

  // Add 20 Elite Vanguards
  for (let i = 0; i < 20; i++) {
    deck.push({
      instanceId: uuidv4(),
      definition: CardCatalog.EliteVanguard,
      ownerId,
    })
  }

  // Add 16 Lightning Strikes
  for (let i = 0; i < 16; i++) {
    deck.push({
      instanceId: uuidv4(),
      definition: CardCatalog.LightningStrike,
      ownerId,
    })
  }

  return deck
}

/**
 * PrebuiltDecks provides factory functions for creating complete deck configurations.
 */
export const PrebuiltDecks = {
  greenDeck,
  redDeck,
} as const
