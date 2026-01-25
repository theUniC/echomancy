import { describe, expect, test } from "vitest"
import { PrebuiltDecks } from "../PrebuiltDecks"

describe("PrebuiltDecks", () => {
  describe("Green Deck", () => {
    test("creates 60-card green deck for a player", () => {
      const ownerId = "player-1"
      const deck = PrebuiltDecks.greenDeck(ownerId)

      expect(deck).toHaveLength(60)
    })

    test("contains 24 Forests", () => {
      const deck = PrebuiltDecks.greenDeck("player-1")
      const forests = deck.filter((card) => card.definition.id === "forest")

      expect(forests).toHaveLength(24)
    })

    test("contains 20 Bears", () => {
      const deck = PrebuiltDecks.greenDeck("player-1")
      const bears = deck.filter((card) => card.definition.id === "bear")

      expect(bears).toHaveLength(20)
    })

    test("contains 16 Giant Growths", () => {
      const deck = PrebuiltDecks.greenDeck("player-1")
      const growths = deck.filter(
        (card) => card.definition.id === "giant-growth",
      )

      expect(growths).toHaveLength(16)
    })

    test("each card has a unique instance ID", () => {
      const deck = PrebuiltDecks.greenDeck("player-1")
      const instanceIds = deck.map((card) => card.instanceId)
      const uniqueIds = new Set(instanceIds)

      expect(uniqueIds.size).toBe(60)
    })

    test("all cards have the correct owner ID", () => {
      const ownerId = "player-1"
      const deck = PrebuiltDecks.greenDeck(ownerId)

      for (const card of deck) {
        expect(card.ownerId).toBe(ownerId)
      }
    })
  })

  describe("Red Deck", () => {
    test("creates 60-card red deck for a player", () => {
      const ownerId = "player-1"
      const deck = PrebuiltDecks.redDeck(ownerId)

      expect(deck).toHaveLength(60)
    })

    test("contains 24 Mountains", () => {
      const deck = PrebuiltDecks.redDeck("player-1")
      const mountains = deck.filter((card) => card.definition.id === "mountain")

      expect(mountains).toHaveLength(24)
    })

    test("contains 20 Elite Vanguards", () => {
      const deck = PrebuiltDecks.redDeck("player-1")
      const vanguards = deck.filter(
        (card) => card.definition.id === "elite-vanguard",
      )

      expect(vanguards).toHaveLength(20)
    })

    test("contains 16 Lightning Strikes", () => {
      const deck = PrebuiltDecks.redDeck("player-1")
      const strikes = deck.filter(
        (card) => card.definition.id === "lightning-strike",
      )

      expect(strikes).toHaveLength(16)
    })

    test("each card has a unique instance ID", () => {
      const deck = PrebuiltDecks.redDeck("player-1")
      const instanceIds = deck.map((card) => card.instanceId)
      const uniqueIds = new Set(instanceIds)

      expect(uniqueIds.size).toBe(60)
    })

    test("all cards have the correct owner ID", () => {
      const ownerId = "player-2"
      const deck = PrebuiltDecks.redDeck(ownerId)

      for (const card of deck) {
        expect(card.ownerId).toBe(ownerId)
      }
    })
  })

  describe("Multiple deck generation", () => {
    test("generating same deck twice produces different instance IDs", () => {
      const deck1 = PrebuiltDecks.greenDeck("player-1")
      const deck2 = PrebuiltDecks.greenDeck("player-1")

      const ids1 = new Set(deck1.map((c) => c.instanceId))
      const ids2 = new Set(deck2.map((c) => c.instanceId))

      // No overlap in instance IDs
      const intersection = [...ids1].filter((id) => ids2.has(id))
      expect(intersection).toHaveLength(0)
    })
  })
})
