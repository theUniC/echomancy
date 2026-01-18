import { describe, expect, test } from "bun:test"
import type { CardInstance } from "../../cards/CardInstance"
import { CreatureState } from "../valueobjects/CreatureState"

const createTestCreatureCard = (
  power: number,
  toughness: number,
): CardInstance => ({
  instanceId: "test-creature-1",
  definition: {
    id: "test-creature",
    name: "Test Creature",
    types: ["CREATURE"],
    power,
    toughness,
  },
  ownerId: "player-1",
})

describe("CreatureState Value Object", () => {
  describe("forCreature()", () => {
    test("creates state with correct base stats", () => {
      const card = createTestCreatureCard(3, 4)
      const state = CreatureState.forCreature(card)

      expect(state.basePower).toBe(3)
      expect(state.baseToughness).toBe(4)
    })

    test("creates state with summoning sickness", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card)

      expect(state.hasSummoningSickness).toBe(true)
    })

    test("creates untapped state", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card)

      expect(state.isTapped).toBe(false)
    })

    test("creates state with no counters", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card)

      expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(0)
    })
  })

  describe("immutability", () => {
    test("withTapped returns new instance", () => {
      const card = createTestCreatureCard(2, 2)
      const state1 = CreatureState.forCreature(card)
      const state2 = state1.withTapped(true)

      expect(state1.isTapped).toBe(false)
      expect(state2.isTapped).toBe(true)
    })

    test("withAttacking returns new instance", () => {
      const card = createTestCreatureCard(2, 2)
      const state1 = CreatureState.forCreature(card)
      const state2 = state1.withAttacking(true)

      expect(state1.isAttacking).toBe(false)
      expect(state2.isAttacking).toBe(true)
    })

    test("withDamage returns new instance", () => {
      const card = createTestCreatureCard(2, 2)
      const state1 = CreatureState.forCreature(card)
      const state2 = state1.withDamage(3)

      expect(state1.damageMarkedThisTurn).toBe(0)
      expect(state2.damageMarkedThisTurn).toBe(3)
    })
  })

  describe("counters", () => {
    test("addCounters increases counter value", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card).addCounters(
        "PLUS_ONE_PLUS_ONE",
        2,
      )

      expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(2)
    })

    test("removeCounters decreases counter value", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card)
        .addCounters("PLUS_ONE_PLUS_ONE", 3)
        .removeCounters("PLUS_ONE_PLUS_ONE", 1)

      expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(2)
    })

    test("removeCounters doesn't go below zero", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card)
        .addCounters("PLUS_ONE_PLUS_ONE", 1)
        .removeCounters("PLUS_ONE_PLUS_ONE", 5)

      expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(0)
    })
  })

  describe("power/toughness calculations", () => {
    test("getCurrentPower includes +1/+1 counters", () => {
      const card = createTestCreatureCard(2, 3)
      const state = CreatureState.forCreature(card).addCounters(
        "PLUS_ONE_PLUS_ONE",
        2,
      )

      expect(state.getCurrentPower()).toBe(4)
    })

    test("getCurrentToughness includes +1/+1 counters", () => {
      const card = createTestCreatureCard(2, 3)
      const state = CreatureState.forCreature(card).addCounters(
        "PLUS_ONE_PLUS_ONE",
        2,
      )

      expect(state.getCurrentToughness()).toBe(5)
    })
  })

  describe("hasLethalDamage()", () => {
    test("returns false when damage < toughness", () => {
      const card = createTestCreatureCard(2, 3)
      const state = CreatureState.forCreature(card).withDamage(2)

      expect(state.hasLethalDamage()).toBe(false)
    })

    test("returns true when damage >= toughness", () => {
      const card = createTestCreatureCard(2, 3)
      const state = CreatureState.forCreature(card).withDamage(3)

      expect(state.hasLethalDamage()).toBe(true)
    })

    test("considers +1/+1 counters for toughness", () => {
      const card = createTestCreatureCard(2, 3)
      const state = CreatureState.forCreature(card)
        .addCounters("PLUS_ONE_PLUS_ONE", 2)
        .withDamage(4)

      expect(state.hasLethalDamage()).toBe(false) // 4 damage < 5 toughness
    })
  })

  describe("resetForNewTurn()", () => {
    test("clears combat state", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card)
        .withAttacking(true)
        .withHasAttackedThisTurn(true)
        .withDamage(1)
        .withBlockingCreatureId("blocker-1")
        .resetForNewTurn()

      expect(state.isAttacking).toBe(false)
      expect(state.hasAttackedThisTurn).toBe(false)
      expect(state.damageMarkedThisTurn).toBe(0)
      expect(state.blockingCreatureId).toBeNull()
    })

    test("removes summoning sickness", () => {
      const card = createTestCreatureCard(2, 2)
      const state = CreatureState.forCreature(card).resetForNewTurn()

      expect(state.hasSummoningSickness).toBe(false)
    })
  })

  describe("toExport()", () => {
    test("exports all fields correctly", () => {
      const card = createTestCreatureCard(3, 4)
      const state = CreatureState.forCreature(card)
        .withTapped(true)
        .addCounters("PLUS_ONE_PLUS_ONE", 1)

      const exported = state.toExport()

      expect(exported.basePower).toBe(3)
      expect(exported.baseToughness).toBe(4)
      expect(exported.currentPower).toBe(4)
      expect(exported.currentToughness).toBe(5)
      expect(exported.isTapped).toBe(true)
      expect(exported.counters).toEqual({ PLUS_ONE_PLUS_ONE: 1 })
    })
  })
})
