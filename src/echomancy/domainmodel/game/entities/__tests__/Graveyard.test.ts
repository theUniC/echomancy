import { describe, expect, test } from "vitest"
import { createTestCreature, createTestPlayer } from "../../__tests__/helpers"
import { Graveyard } from "../Graveyard"

describe("Graveyard", () => {
  describe("Creation", () => {
    test("creates empty graveyard", () => {
      const graveyard = Graveyard.empty()

      expect(graveyard.isEmpty()).toBe(true)
      expect(graveyard.count()).toBe(0)
      expect(graveyard.getAll()).toEqual([])
    })

    test("creates graveyard from existing cards", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)

      const graveyard = Graveyard.fromCards([creature1, creature2])

      expect(graveyard.isEmpty()).toBe(false)
      expect(graveyard.count()).toBe(2)
      expect(graveyard.getAll()).toEqual([creature1, creature2])
    })
  })

  describe("addCard", () => {
    test("adds card to empty graveyard", () => {
      const graveyard = Graveyard.empty()
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)

      const newGraveyard = graveyard.addCard(creature)

      expect(newGraveyard.count()).toBe(1)
      expect(newGraveyard.getTopCard()).toBe(creature)
    })

    test("returns new instance when adding card", () => {
      const graveyard = Graveyard.empty()
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)

      const newGraveyard = graveyard.addCard(creature)

      expect(newGraveyard).not.toBe(graveyard)
      expect(graveyard.count()).toBe(0) // Original unchanged
      expect(newGraveyard.count()).toBe(1)
    })

    test("adds multiple cards maintaining order (last in is on top)", () => {
      const graveyard = Graveyard.empty()
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const creature3 = createTestCreature(player.id)

      const gy1 = graveyard.addCard(creature1)
      const gy2 = gy1.addCard(creature2)
      const gy3 = gy2.addCard(creature3)

      expect(gy3.count()).toBe(3)
      expect(gy3.getTopCard()).toBe(creature3) // Last in is on top
      expect(gy3.getAll()).toEqual([creature1, creature2, creature3])
    })
  })

  describe("getTopCard", () => {
    test("returns most recently added card", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const graveyard = Graveyard.fromCards([creature1, creature2])

      const topCard = graveyard.getTopCard()

      expect(topCard).toBe(creature2)
    })

    test("returns undefined for empty graveyard", () => {
      const graveyard = Graveyard.empty()

      const topCard = graveyard.getTopCard()

      expect(topCard).toBeUndefined()
    })

    test("returns only card in single-card graveyard", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const graveyard = Graveyard.fromCards([creature])

      const topCard = graveyard.getTopCard()

      expect(topCard).toBe(creature)
    })
  })

  describe("getAll", () => {
    test("returns all cards in order (bottom to top)", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const creature3 = createTestCreature(player.id)
      const graveyard = Graveyard.fromCards([creature1, creature2, creature3])

      const all = graveyard.getAll()

      expect(all).toEqual([creature1, creature2, creature3])
    })

    test("returns empty array for empty graveyard", () => {
      const graveyard = Graveyard.empty()

      const all = graveyard.getAll()

      expect(all).toEqual([])
    })

    test("returned array is not mutable reference", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const graveyard = Graveyard.fromCards([creature])

      const all = graveyard.getAll()
      all.push(createTestCreature(player.id)) // Try to mutate

      // Original graveyard should be unchanged
      expect(graveyard.count()).toBe(1)
      expect(graveyard.getAll()).toHaveLength(1)
    })
  })

  describe("isEmpty", () => {
    test("returns true for empty graveyard", () => {
      const graveyard = Graveyard.empty()

      expect(graveyard.isEmpty()).toBe(true)
    })

    test("returns false for graveyard with cards", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const graveyard = Graveyard.fromCards([creature])

      expect(graveyard.isEmpty()).toBe(false)
    })
  })

  describe("count", () => {
    test("returns 0 for empty graveyard", () => {
      const graveyard = Graveyard.empty()

      expect(graveyard.count()).toBe(0)
    })

    test("returns correct count for graveyard with cards", () => {
      const player = createTestPlayer()
      const creatures = [
        createTestCreature(player.id),
        createTestCreature(player.id),
        createTestCreature(player.id),
      ]
      const graveyard = Graveyard.fromCards(creatures)

      expect(graveyard.count()).toBe(3)
    })
  })

  describe("Migration from Zone", () => {
    test("creates graveyard from Zone-like object", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const zone = { cards: [creature] }

      const graveyard = Graveyard.fromZone(zone)

      expect(graveyard.count()).toBe(1)
      expect(graveyard.getTopCard()).toBe(creature)
    })

    test("creates empty graveyard from empty Zone", () => {
      const zone = { cards: [] }

      const graveyard = Graveyard.fromZone(zone)

      expect(graveyard.isEmpty()).toBe(true)
    })

    test("preserves card order from Zone", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const zone = { cards: [creature1, creature2] }

      const graveyard = Graveyard.fromZone(zone)

      expect(graveyard.getAll()).toEqual([creature1, creature2])
      expect(graveyard.getTopCard()).toBe(creature2)
    })
  })
})
