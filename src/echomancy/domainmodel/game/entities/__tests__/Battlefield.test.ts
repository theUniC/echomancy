import { describe, expect, test } from "vitest"
import { createTestCreature, createTestPlayer } from "../../__tests__/helpers"
import { Battlefield } from "../Battlefield"

describe("Battlefield", () => {
  describe("Creation", () => {
    test("creates empty battlefield", () => {
      const battlefield = Battlefield.empty()

      expect(battlefield.isEmpty()).toBe(true)
      expect(battlefield.count()).toBe(0)
      expect(battlefield.getAll()).toEqual([])
    })

    test("creates battlefield from existing cards", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)

      const battlefield = Battlefield.fromCards([creature1, creature2])

      expect(battlefield.isEmpty()).toBe(false)
      expect(battlefield.count()).toBe(2)
      expect(battlefield.getAll()).toEqual([creature1, creature2])
    })
  })

  describe("addPermanent", () => {
    test("adds permanent to empty battlefield", () => {
      const battlefield = Battlefield.empty()
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)

      const newBattlefield = battlefield.addPermanent(creature)

      expect(newBattlefield.count()).toBe(1)
      expect(newBattlefield.findPermanent(creature.instanceId)).toBe(creature)
    })

    test("returns new instance when adding permanent", () => {
      const battlefield = Battlefield.empty()
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)

      const newBattlefield = battlefield.addPermanent(creature)

      expect(newBattlefield).not.toBe(battlefield)
      expect(battlefield.count()).toBe(0) // Original unchanged
      expect(newBattlefield.count()).toBe(1)
    })

    test("adds multiple permanents", () => {
      const battlefield = Battlefield.empty()
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)

      const bf1 = battlefield.addPermanent(creature1)
      const bf2 = bf1.addPermanent(creature2)

      expect(bf2.count()).toBe(2)
      expect(bf2.findPermanent(creature1.instanceId)).toBe(creature1)
      expect(bf2.findPermanent(creature2.instanceId)).toBe(creature2)
    })
  })

  describe("removePermanent", () => {
    test("removes permanent by instanceId", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([creature])

      const newBattlefield = battlefield.removePermanent(creature.instanceId)

      expect(newBattlefield.isEmpty()).toBe(true)
      expect(newBattlefield.findPermanent(creature.instanceId)).toBeUndefined()
    })

    test("returns new instance when removing permanent", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([creature])

      const newBattlefield = battlefield.removePermanent(creature.instanceId)

      expect(newBattlefield).not.toBe(battlefield)
      expect(battlefield.count()).toBe(1) // Original unchanged
      expect(newBattlefield.count()).toBe(0)
    })

    test("removes correct permanent when multiple exist", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const creature3 = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([
        creature1,
        creature2,
        creature3,
      ])

      const newBattlefield = battlefield.removePermanent(creature2.instanceId)

      expect(newBattlefield.count()).toBe(2)
      expect(newBattlefield.findPermanent(creature1.instanceId)).toBe(creature1)
      expect(newBattlefield.findPermanent(creature2.instanceId)).toBeUndefined()
      expect(newBattlefield.findPermanent(creature3.instanceId)).toBe(creature3)
    })

    test("returns same battlefield when removing non-existent permanent", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([creature])

      const newBattlefield = battlefield.removePermanent("non-existent-id")

      expect(newBattlefield).not.toBe(battlefield) // Still new instance
      expect(newBattlefield.count()).toBe(1)
      expect(newBattlefield.findPermanent(creature.instanceId)).toBe(creature)
    })
  })

  describe("findPermanent", () => {
    test("finds permanent by instanceId", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([creature])

      const found = battlefield.findPermanent(creature.instanceId)

      expect(found).toBe(creature)
    })

    test("returns undefined for non-existent permanent", () => {
      const battlefield = Battlefield.empty()

      const found = battlefield.findPermanent("non-existent-id")

      expect(found).toBeUndefined()
    })

    test("finds correct permanent among multiple", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const creature3 = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([
        creature1,
        creature2,
        creature3,
      ])

      const found = battlefield.findPermanent(creature2.instanceId)

      expect(found).toBe(creature2)
    })
  })

  describe("findPermanentsByOwner", () => {
    test("finds all permanents owned by a player", () => {
      const player1 = createTestPlayer()
      const player2 = createTestPlayer()
      const creature1 = createTestCreature(player1.id)
      const creature2 = createTestCreature(player2.id)
      const creature3 = createTestCreature(player1.id)
      const battlefield = Battlefield.fromCards([
        creature1,
        creature2,
        creature3,
      ])

      const player1Permanents = battlefield.findPermanentsByOwner(player1.id)

      expect(player1Permanents).toHaveLength(2)
      expect(player1Permanents).toContain(creature1)
      expect(player1Permanents).toContain(creature3)
    })

    test("returns empty array for player with no permanents", () => {
      const player1 = createTestPlayer()
      const player2 = createTestPlayer()
      const creature = createTestCreature(player1.id)
      const battlefield = Battlefield.fromCards([creature])

      const player2Permanents = battlefield.findPermanentsByOwner(player2.id)

      expect(player2Permanents).toEqual([])
    })

    test("returns empty array on empty battlefield", () => {
      const player = createTestPlayer()
      const battlefield = Battlefield.empty()

      const permanents = battlefield.findPermanentsByOwner(player.id)

      expect(permanents).toEqual([])
    })
  })

  describe("getAll", () => {
    test("returns all permanents in order", () => {
      const player = createTestPlayer()
      const creature1 = createTestCreature(player.id)
      const creature2 = createTestCreature(player.id)
      const creature3 = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([
        creature1,
        creature2,
        creature3,
      ])

      const all = battlefield.getAll()

      expect(all).toEqual([creature1, creature2, creature3])
    })

    test("returns empty array for empty battlefield", () => {
      const battlefield = Battlefield.empty()

      const all = battlefield.getAll()

      expect(all).toEqual([])
    })

    test("returned array is not mutable reference", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([creature])

      const all = battlefield.getAll()
      all.push(createTestCreature(player.id)) // Try to mutate

      // Original battlefield should be unchanged
      expect(battlefield.count()).toBe(1)
      expect(battlefield.getAll()).toHaveLength(1)
    })
  })

  describe("isEmpty", () => {
    test("returns true for empty battlefield", () => {
      const battlefield = Battlefield.empty()

      expect(battlefield.isEmpty()).toBe(true)
    })

    test("returns false for battlefield with permanents", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const battlefield = Battlefield.fromCards([creature])

      expect(battlefield.isEmpty()).toBe(false)
    })
  })

  describe("count", () => {
    test("returns 0 for empty battlefield", () => {
      const battlefield = Battlefield.empty()

      expect(battlefield.count()).toBe(0)
    })

    test("returns correct count for battlefield with permanents", () => {
      const player = createTestPlayer()
      const creatures = [
        createTestCreature(player.id),
        createTestCreature(player.id),
        createTestCreature(player.id),
      ]
      const battlefield = Battlefield.fromCards(creatures)

      expect(battlefield.count()).toBe(3)
    })
  })

  describe("Migration from Zone", () => {
    test("creates battlefield from Zone-like object", () => {
      const player = createTestPlayer()
      const creature = createTestCreature(player.id)
      const zone = { cards: [creature] }

      const battlefield = Battlefield.fromZone(zone)

      expect(battlefield.count()).toBe(1)
      expect(battlefield.findPermanent(creature.instanceId)).toBe(creature)
    })

    test("creates empty battlefield from empty Zone", () => {
      const zone = { cards: [] }

      const battlefield = Battlefield.fromZone(zone)

      expect(battlefield.isEmpty()).toBe(true)
    })
  })
})
