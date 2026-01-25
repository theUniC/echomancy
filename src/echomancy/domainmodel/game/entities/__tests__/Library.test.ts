import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../../../cards/CardInstance"
import { Library } from "../Library"

describe("Library", () => {
  describe("empty()", () => {
    test("creates an empty library", () => {
      const library = Library.empty()

      expect(library.isEmpty()).toBe(true)
      expect(library.count()).toBe(0)
    })
  })

  describe("fromCards()", () => {
    test("creates a library from card array", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)

      expect(library.count()).toBe(2)
      expect(library.isEmpty()).toBe(false)
    })

    test("preserves card order (top to bottom)", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-top",
          definition: { id: "def-1", name: "Top", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-middle",
          definition: { id: "def-2", name: "Middle", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-bottom",
          definition: { id: "def-3", name: "Bottom", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)
      const topCard = library.peekTop(1)

      expect(topCard).toHaveLength(1)
      expect(topCard[0]?.instanceId).toBe("card-top")
    })
  })

  describe("drawFromTop()", () => {
    test("removes top card and returns it with new library", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)
      const result = library.drawFromTop()

      expect(result.card?.instanceId).toBe("card-1")
      expect(result.newLibrary.count()).toBe(1)

      // Original library unchanged (immutable)
      expect(library.count()).toBe(2)
    })

    test("returns undefined card when drawing from empty library", () => {
      const library = Library.empty()
      const result = library.drawFromTop()

      expect(result.card).toBeUndefined()
      expect(result.newLibrary.isEmpty()).toBe(true)
    })

    test("preserves order after multiple draws", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "First", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Second", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-3",
          definition: { id: "def-3", name: "Third", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)

      const firstDraw = library.drawFromTop()
      expect(firstDraw.card?.instanceId).toBe("card-1")

      const secondDraw = firstDraw.newLibrary.drawFromTop()
      expect(secondDraw.card?.instanceId).toBe("card-2")

      const thirdDraw = secondDraw.newLibrary.drawFromTop()
      expect(thirdDraw.card?.instanceId).toBe("card-3")

      expect(thirdDraw.newLibrary.isEmpty()).toBe(true)
    })
  })

  describe("peekTop()", () => {
    test("returns top N cards without modifying library", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-3",
          definition: { id: "def-3", name: "Card 3", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)
      const peeked = library.peekTop(2)

      expect(peeked).toHaveLength(2)
      expect(peeked[0]?.instanceId).toBe("card-1")
      expect(peeked[1]?.instanceId).toBe("card-2")

      // Library unchanged
      expect(library.count()).toBe(3)
    })

    test("returns all cards if N exceeds library size", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)
      const peeked = library.peekTop(10)

      expect(peeked).toHaveLength(1)
      expect(peeked[0]?.instanceId).toBe("card-1")
    })

    test("returns empty array when peeking empty library", () => {
      const library = Library.empty()
      const peeked = library.peekTop(3)

      expect(peeked).toHaveLength(0)
    })
  })

  describe("count()", () => {
    test("returns correct card count", () => {
      const library = Library.fromCards([
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
      ])

      expect(library.count()).toBe(2)
    })
  })

  describe("isEmpty()", () => {
    test("returns true for empty library", () => {
      const library = Library.empty()
      expect(library.isEmpty()).toBe(true)
    })

    test("returns false for library with cards", () => {
      const library = Library.fromCards([
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
      ])

      expect(library.isEmpty()).toBe(false)
    })
  })

  describe("immutability", () => {
    test("original library unchanged after drawFromTop", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
      ]

      const original = Library.fromCards(cards)
      const afterDraw = original.drawFromTop()

      expect(original.count()).toBe(1)
      expect(afterDraw.newLibrary.count()).toBe(0)
    })

    test("cards array returned by peekTop is defensive copy", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
      ]

      const library = Library.fromCards(cards)
      const peeked = library.peekTop(1)

      // Mutate the returned array
      peeked.push({
        instanceId: "hacked-card",
        definition: { id: "hack", name: "Hack", types: ["INSTANT"] },
        ownerId: "player-1",
      })

      // Library should be unchanged
      expect(library.count()).toBe(1)
      expect(library.peekTop(1)).toHaveLength(1)
    })
  })

  describe("shuffle()", () => {
    test("returns a new Library instance", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-3",
          definition: { id: "def-3", name: "Card 3", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]
      const library = Library.fromCards(cards)

      const shuffled = library.shuffle()

      expect(shuffled).toBeInstanceOf(Library)
      expect(shuffled).not.toBe(library)
    })

    test("shuffled library has same number of cards", () => {
      const cards: CardInstance[] = Array.from({ length: 5 }, (_, i) => ({
        instanceId: `card-${i + 1}`,
        definition: { id: `def-${i}`, name: `Card ${i}`, types: ["INSTANT"] },
        ownerId: "player-1",
      }))
      const library = Library.fromCards(cards)

      const shuffled = library.shuffle()

      expect(shuffled.count()).toBe(5)
    })

    test("shuffled library contains same cards", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-3",
          definition: { id: "def-3", name: "Card 3", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]
      const library = Library.fromCards(cards)

      const shuffled = library.shuffle()
      const originalIds = new Set(cards.map((c) => c.instanceId))
      const shuffledIds = new Set(shuffled.getAll().map((c) => c.instanceId))

      expect(shuffledIds).toEqual(originalIds)
    })

    test("shuffle with seed produces deterministic results", () => {
      const cards: CardInstance[] = Array.from({ length: 20 }, (_, i) => ({
        instanceId: `card-${i + 1}`,
        definition: { id: `def-${i}`, name: `Card ${i}`, types: ["INSTANT"] },
        ownerId: "player-1",
      }))
      const library = Library.fromCards(cards)

      const seed = 12345
      const shuffled1 = library.shuffle(seed)
      const shuffled2 = library.shuffle(seed)

      const order1 = shuffled1.getAll().map((c) => c.instanceId)
      const order2 = shuffled2.getAll().map((c) => c.instanceId)

      expect(order1).toEqual(order2)
    })

    test("shuffle without seed produces random results", () => {
      const cards: CardInstance[] = Array.from({ length: 60 }, (_, i) => ({
        instanceId: `card-${i + 1}`,
        definition: { id: `def-${i}`, name: `Card ${i}`, types: ["INSTANT"] },
        ownerId: "player-1",
      }))
      const library = Library.fromCards(cards)

      const shuffled1 = library.shuffle()
      const shuffled2 = library.shuffle()

      const order1 = shuffled1.getAll().map((c) => c.instanceId)
      const order2 = shuffled2.getAll().map((c) => c.instanceId)

      // Very unlikely to be the same order
      // For a 60-card deck, probability is 1/60! which is effectively 0
      expect(order1).not.toEqual(order2)
    })

    test("shuffle with different seeds produces different results", () => {
      const cards: CardInstance[] = Array.from({ length: 20 }, (_, i) => ({
        instanceId: `card-${i + 1}`,
        definition: { id: `def-${i}`, name: `Card ${i}`, types: ["INSTANT"] },
        ownerId: "player-1",
      }))
      const library = Library.fromCards(cards)

      const shuffled1 = library.shuffle(12345)
      const shuffled2 = library.shuffle(54321)

      const order1 = shuffled1.getAll().map((c) => c.instanceId)
      const order2 = shuffled2.getAll().map((c) => c.instanceId)

      expect(order1).not.toEqual(order2)
    })

    test("shuffling empty library returns empty library", () => {
      const library = Library.empty()

      const shuffled = library.shuffle()

      expect(shuffled.isEmpty()).toBe(true)
      expect(shuffled.count()).toBe(0)
    })

    test("shuffling single-card library returns same card", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "only-card",
          definition: { id: "def-1", name: "Only Card", types: ["INSTANT"] },
          ownerId: "player-1",
        },
      ]
      const library = Library.fromCards(cards)

      const shuffled = library.shuffle()

      expect(shuffled.count()).toBe(1)
      expect(shuffled.peekTop(1)[0]?.instanceId).toBe("only-card")
    })

    test("drawFromTop works after shuffle", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-3",
          definition: { id: "def-3", name: "Card 3", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]
      const library = Library.fromCards(cards)
      const shuffled = library.shuffle(12345)

      const { card, newLibrary } = shuffled.drawFromTop()

      expect(card).toBeDefined()
      expect(newLibrary.count()).toBe(2)
    })

    test("peekTop works after shuffle", () => {
      const cards: CardInstance[] = [
        {
          instanceId: "card-1",
          definition: { id: "def-1", name: "Card 1", types: ["INSTANT"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-2",
          definition: { id: "def-2", name: "Card 2", types: ["CREATURE"] },
          ownerId: "player-1",
        },
        {
          instanceId: "card-3",
          definition: { id: "def-3", name: "Card 3", types: ["LAND"] },
          ownerId: "player-1",
        },
      ]
      const library = Library.fromCards(cards)
      const shuffled = library.shuffle(12345)

      const topCards = shuffled.peekTop(2)

      expect(topCards).toHaveLength(2)
    })
  })
})
