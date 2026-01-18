import { describe, expect, test } from "vitest"
import { createTestCreature, createTestPlayer } from "../../__tests__/helpers"
import { Hand } from "../Hand"

describe("Hand", () => {
  describe("Creation", () => {
    test("creates empty hand", () => {
      const hand = Hand.empty()

      expect(hand.isEmpty()).toBe(true)
      expect(hand.count()).toBe(0)
      expect(hand.getAll()).toEqual([])
    })

    test("creates hand from existing cards", () => {
      const player = createTestPlayer()
      const card1 = createTestCreature(player.id)
      const card2 = createTestCreature(player.id)

      const hand = Hand.fromCards([card1, card2])

      expect(hand.isEmpty()).toBe(false)
      expect(hand.count()).toBe(2)
      expect(hand.getAll()).toEqual([card1, card2])
    })
  })

  describe("addCard", () => {
    test("adds card to empty hand", () => {
      const hand = Hand.empty()
      const player = createTestPlayer()
      const card = createTestCreature(player.id)

      const newHand = hand.addCard(card)

      expect(newHand.count()).toBe(1)
      expect(newHand.findCard(card.instanceId)).toBe(card)
    })

    test("returns new instance when adding card", () => {
      const hand = Hand.empty()
      const player = createTestPlayer()
      const card = createTestCreature(player.id)

      const newHand = hand.addCard(card)

      expect(newHand).not.toBe(hand)
      expect(hand.count()).toBe(0) // Original unchanged
      expect(newHand.count()).toBe(1)
    })

    test("adds multiple cards", () => {
      const hand = Hand.empty()
      const player = createTestPlayer()
      const card1 = createTestCreature(player.id)
      const card2 = createTestCreature(player.id)

      const hand1 = hand.addCard(card1)
      const hand2 = hand1.addCard(card2)

      expect(hand2.count()).toBe(2)
      expect(hand2.findCard(card1.instanceId)).toBe(card1)
      expect(hand2.findCard(card2.instanceId)).toBe(card2)
    })
  })

  describe("removeCard", () => {
    test("removes card by instanceId", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const hand = Hand.fromCards([card])

      const newHand = hand.removeCard(card.instanceId)

      expect(newHand.isEmpty()).toBe(true)
      expect(newHand.findCard(card.instanceId)).toBeUndefined()
    })

    test("returns new instance when removing card", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const hand = Hand.fromCards([card])

      const newHand = hand.removeCard(card.instanceId)

      expect(newHand).not.toBe(hand)
      expect(hand.count()).toBe(1) // Original unchanged
      expect(newHand.count()).toBe(0)
    })

    test("removes correct card when multiple exist", () => {
      const player = createTestPlayer()
      const card1 = createTestCreature(player.id)
      const card2 = createTestCreature(player.id)
      const card3 = createTestCreature(player.id)
      const hand = Hand.fromCards([card1, card2, card3])

      const newHand = hand.removeCard(card2.instanceId)

      expect(newHand.count()).toBe(2)
      expect(newHand.findCard(card1.instanceId)).toBe(card1)
      expect(newHand.findCard(card2.instanceId)).toBeUndefined()
      expect(newHand.findCard(card3.instanceId)).toBe(card3)
    })

    test("returns same hand when removing non-existent card", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const hand = Hand.fromCards([card])

      const newHand = hand.removeCard("non-existent-id")

      expect(newHand).not.toBe(hand) // Still new instance
      expect(newHand.count()).toBe(1)
      expect(newHand.findCard(card.instanceId)).toBe(card)
    })
  })

  describe("findCard", () => {
    test("finds card by instanceId", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const hand = Hand.fromCards([card])

      const found = hand.findCard(card.instanceId)

      expect(found).toBe(card)
    })

    test("returns undefined for non-existent card", () => {
      const hand = Hand.empty()

      const found = hand.findCard("non-existent-id")

      expect(found).toBeUndefined()
    })

    test("finds correct card among multiple", () => {
      const player = createTestPlayer()
      const card1 = createTestCreature(player.id)
      const card2 = createTestCreature(player.id)
      const card3 = createTestCreature(player.id)
      const hand = Hand.fromCards([card1, card2, card3])

      const found = hand.findCard(card2.instanceId)

      expect(found).toBe(card2)
    })
  })

  describe("getAll", () => {
    test("returns all cards in order", () => {
      const player = createTestPlayer()
      const card1 = createTestCreature(player.id)
      const card2 = createTestCreature(player.id)
      const card3 = createTestCreature(player.id)
      const hand = Hand.fromCards([card1, card2, card3])

      const all = hand.getAll()

      expect(all).toEqual([card1, card2, card3])
    })

    test("returns empty array for empty hand", () => {
      const hand = Hand.empty()

      const all = hand.getAll()

      expect(all).toEqual([])
    })

    test("returned array is not mutable reference", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const hand = Hand.fromCards([card])

      const all = hand.getAll()
      all.push(createTestCreature(player.id)) // Try to mutate

      // Original hand should be unchanged
      expect(hand.count()).toBe(1)
      expect(hand.getAll()).toHaveLength(1)
    })
  })

  describe("isEmpty", () => {
    test("returns true for empty hand", () => {
      const hand = Hand.empty()

      expect(hand.isEmpty()).toBe(true)
    })

    test("returns false for hand with cards", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const hand = Hand.fromCards([card])

      expect(hand.isEmpty()).toBe(false)
    })
  })

  describe("count", () => {
    test("returns 0 for empty hand", () => {
      const hand = Hand.empty()

      expect(hand.count()).toBe(0)
    })

    test("returns correct count for hand with cards", () => {
      const player = createTestPlayer()
      const cards = [
        createTestCreature(player.id),
        createTestCreature(player.id),
        createTestCreature(player.id),
      ]
      const hand = Hand.fromCards(cards)

      expect(hand.count()).toBe(3)
    })
  })

  describe("Migration from Zone", () => {
    test("creates hand from Zone-like object", () => {
      const player = createTestPlayer()
      const card = createTestCreature(player.id)
      const zone = { cards: [card] }

      const hand = Hand.fromZone(zone)

      expect(hand.count()).toBe(1)
      expect(hand.findCard(card.instanceId)).toBe(card)
    })

    test("creates empty hand from empty Zone", () => {
      const zone = { cards: [] }

      const hand = Hand.fromZone(zone)

      expect(hand.isEmpty()).toBe(true)
    })
  })
})
