import { describe, expect, test } from "vitest"
import type { CardInstance } from "../../../cards/CardInstance"
import { PermanentState } from "../PermanentState"

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

const _createTestArtifactCard = (): CardInstance => ({
  instanceId: "test-artifact-1",
  definition: {
    id: "test-artifact",
    name: "Test Artifact",
    types: ["ARTIFACT"],
  },
  ownerId: "player-1",
})

const _createTestLandCard = (): CardInstance => ({
  instanceId: "test-land-1",
  definition: {
    id: "test-land",
    name: "Test Land",
    types: ["LAND"],
  },
  ownerId: "player-1",
})

describe("PermanentState Value Object", () => {
  describe("Factory Methods", () => {
    describe("forCreature()", () => {
      test("creates state with creature sub-state", () => {
        const card = createTestCreatureCard(3, 4)
        const state = PermanentState.forCreature(card)

        expect(state.creatureState).toBeDefined()
        expect(state.creatureState?.basePower).toBe(3)
        expect(state.creatureState?.baseToughness).toBe(4)
        expect(state.creatureState?.hasSummoningSickness).toBe(true)
      })

      test("creates state with common permanent state", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card)

        expect(state.isTapped).toBe(false)
        expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(0)
      })
    })

    describe("forNonCreature()", () => {
      test("creates state without creature sub-state", () => {
        const state = PermanentState.forNonCreature()

        expect(state.creatureState).toBeUndefined()
      })

      test("creates state with common permanent state", () => {
        const state = PermanentState.forNonCreature()

        expect(state.isTapped).toBe(false)
        expect(state.getCounters("CHARGE")).toBe(0)
      })
    })
  })

  describe("Common Operations (All Permanents)", () => {
    describe("withTapped()", () => {
      test("taps a creature", () => {
        const card = createTestCreatureCard(2, 2)
        const state1 = PermanentState.forCreature(card)
        const state2 = state1.withTapped(true)

        expect(state1.isTapped).toBe(false)
        expect(state2.isTapped).toBe(true)
      })

      test("taps an artifact", () => {
        const state1 = PermanentState.forNonCreature()
        const state2 = state1.withTapped(true)

        expect(state1.isTapped).toBe(false)
        expect(state2.isTapped).toBe(true)
      })

      test("untaps a permanent", () => {
        const state1 = PermanentState.forNonCreature().withTapped(true)
        const state2 = state1.withTapped(false)

        expect(state1.isTapped).toBe(true)
        expect(state2.isTapped).toBe(false)
      })
    })

    describe("addCounters()", () => {
      test("adds +1/+1 counters to a creature", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card).addCounters(
          "PLUS_ONE_PLUS_ONE",
          2,
        )

        expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(2)
      })

      test("adds charge counters to an artifact", () => {
        const state = PermanentState.forNonCreature().addCounters("CHARGE", 3)

        expect(state.getCounters("CHARGE")).toBe(3)
      })

      test("adds loyalty counters", () => {
        const state = PermanentState.forNonCreature().addCounters("LOYALTY", 4)

        expect(state.getCounters("LOYALTY")).toBe(4)
      })

      test("supports any string counter type", () => {
        const state = PermanentState.forNonCreature().addCounters(
          "CUSTOM_COUNTER",
          5,
        )

        expect(state.getCounters("CUSTOM_COUNTER")).toBe(5)
      })

      test("multiple additions accumulate", () => {
        const state = PermanentState.forNonCreature()
          .addCounters("CHARGE", 2)
          .addCounters("CHARGE", 3)

        expect(state.getCounters("CHARGE")).toBe(5)
      })
    })

    describe("removeCounters()", () => {
      test("removes counters", () => {
        const state = PermanentState.forNonCreature()
          .addCounters("CHARGE", 5)
          .removeCounters("CHARGE", 2)

        expect(state.getCounters("CHARGE")).toBe(3)
      })

      test("doesn't go below zero", () => {
        const state = PermanentState.forNonCreature()
          .addCounters("CHARGE", 2)
          .removeCounters("CHARGE", 10)

        expect(state.getCounters("CHARGE")).toBe(0)
      })

      test("removes counter type when reaching zero", () => {
        const state = PermanentState.forNonCreature()
          .addCounters("CHARGE", 2)
          .removeCounters("CHARGE", 2)

        expect(state.getCounters("CHARGE")).toBe(0)
      })
    })

    describe("getCounters()", () => {
      test("returns zero for non-existent counter type", () => {
        const state = PermanentState.forNonCreature()

        expect(state.getCounters("NONEXISTENT")).toBe(0)
      })

      test("returns correct count for existing counters", () => {
        const state = PermanentState.forNonCreature().addCounters("CHARGE", 7)

        expect(state.getCounters("CHARGE")).toBe(7)
      })
    })
  })

  describe("Creature-Specific Operations", () => {
    describe("withAttacking()", () => {
      test("sets attacking state on creatures", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card).withAttacking(true)

        expect(state.creatureState?.isAttacking).toBe(true)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.withAttacking(true)).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("withHasAttackedThisTurn()", () => {
      test("sets attack history on creatures", () => {
        const card = createTestCreatureCard(2, 2)
        const state =
          PermanentState.forCreature(card).withHasAttackedThisTurn(true)

        expect(state.creatureState?.hasAttackedThisTurn).toBe(true)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.withHasAttackedThisTurn(true)).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("withSummoningSickness()", () => {
      test("sets summoning sickness on creatures", () => {
        const card = createTestCreatureCard(2, 2)
        const state =
          PermanentState.forCreature(card).withSummoningSickness(false)

        expect(state.creatureState?.hasSummoningSickness).toBe(false)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.withSummoningSickness(false)).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("withDamage()", () => {
      test("sets damage on creatures", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card).withDamage(3)

        expect(state.creatureState?.damageMarkedThisTurn).toBe(3)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.withDamage(3)).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("withBlockingCreatureId()", () => {
      test("sets blocking creature id", () => {
        const card = createTestCreatureCard(2, 2)
        const state =
          PermanentState.forCreature(card).withBlockingCreatureId("attacker-1")

        expect(state.creatureState?.blockingCreatureId).toBe("attacker-1")
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.withBlockingCreatureId("id")).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("withBlockedBy()", () => {
      test("sets blocked by id", () => {
        const card = createTestCreatureCard(2, 2)
        const state =
          PermanentState.forCreature(card).withBlockedBy("blocker-1")

        expect(state.creatureState?.blockedBy).toBe("blocker-1")
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.withBlockedBy("id")).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("getCurrentPower()", () => {
      test("returns power with +1/+1 counters", () => {
        const card = createTestCreatureCard(3, 4)
        const state = PermanentState.forCreature(card).addCounters(
          "PLUS_ONE_PLUS_ONE",
          2,
        )

        expect(state.getCurrentPower()).toBe(5)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.getCurrentPower()).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("getCurrentToughness()", () => {
      test("returns toughness with +1/+1 counters", () => {
        const card = createTestCreatureCard(3, 4)
        const state = PermanentState.forCreature(card).addCounters(
          "PLUS_ONE_PLUS_ONE",
          2,
        )

        expect(state.getCurrentToughness()).toBe(6)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.getCurrentToughness()).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("hasLethalDamage()", () => {
      test("returns true when damage >= toughness", () => {
        const card = createTestCreatureCard(2, 3)
        const state = PermanentState.forCreature(card).withDamage(3)

        expect(state.hasLethalDamage()).toBe(true)
      })

      test("returns false when damage < toughness", () => {
        const card = createTestCreatureCard(2, 3)
        const state = PermanentState.forCreature(card).withDamage(2)

        expect(state.hasLethalDamage()).toBe(false)
      })

      test("considers +1/+1 counters", () => {
        const card = createTestCreatureCard(2, 3)
        const state = PermanentState.forCreature(card)
          .addCounters("PLUS_ONE_PLUS_ONE", 2)
          .withDamage(4)

        expect(state.hasLethalDamage()).toBe(false) // 4 damage < 5 toughness
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.hasLethalDamage()).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("resetForNewTurn()", () => {
      test("clears combat state for creatures", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card)
          .withAttacking(true)
          .withHasAttackedThisTurn(true)
          .withDamage(2)
          .withBlockingCreatureId("attacker-1")
          .withBlockedBy("blocker-1")
          .resetForNewTurn()

        expect(state.creatureState?.isAttacking).toBe(false)
        expect(state.creatureState?.hasAttackedThisTurn).toBe(false)
        expect(state.creatureState?.damageMarkedThisTurn).toBe(0)
        expect(state.creatureState?.blockingCreatureId).toBeNull()
        expect(state.creatureState?.blockedBy).toBeNull()
        expect(state.creatureState?.hasSummoningSickness).toBe(false)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.resetForNewTurn()).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("clearDamage()", () => {
      test("clears damage marked this turn", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card)
          .withDamage(5)
          .clearDamage()

        expect(state.creatureState?.damageMarkedThisTurn).toBe(0)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.clearDamage()).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })

    describe("clearCombatState()", () => {
      test("clears combat state but preserves damage", () => {
        const card = createTestCreatureCard(2, 2)
        const state = PermanentState.forCreature(card)
          .withAttacking(true)
          .withBlockingCreatureId("attacker-1")
          .withBlockedBy("blocker-1")
          .withDamage(3)
          .clearCombatState()

        expect(state.creatureState?.isAttacking).toBe(false)
        expect(state.creatureState?.blockingCreatureId).toBeNull()
        expect(state.creatureState?.blockedBy).toBeNull()
        expect(state.creatureState?.damageMarkedThisTurn).toBe(3)
      })

      test("throws when called on non-creature", () => {
        const state = PermanentState.forNonCreature()

        expect(() => state.clearCombatState()).toThrow(
          "Cannot use creature-specific operation on non-creature permanent",
        )
      })
    })
  })

  describe("Immutability", () => {
    test("withTapped returns new instance", () => {
      const state1 = PermanentState.forNonCreature()
      const state2 = state1.withTapped(true)

      expect(state1).not.toBe(state2)
      expect(state1.isTapped).toBe(false)
      expect(state2.isTapped).toBe(true)
    })

    test("addCounters returns new instance", () => {
      const state1 = PermanentState.forNonCreature()
      const state2 = state1.addCounters("CHARGE", 1)

      expect(state1).not.toBe(state2)
      expect(state1.getCounters("CHARGE")).toBe(0)
      expect(state2.getCounters("CHARGE")).toBe(1)
    })

    test("creature operations return new instance", () => {
      const card = createTestCreatureCard(2, 2)
      const state1 = PermanentState.forCreature(card)
      const state2 = state1.withAttacking(true)

      expect(state1).not.toBe(state2)
      expect(state1.creatureState?.isAttacking).toBe(false)
      expect(state2.creatureState?.isAttacking).toBe(true)
    })
  })

  describe("Mixed Operations", () => {
    test("creature can be tapped and have counters", () => {
      const card = createTestCreatureCard(3, 4)
      const state = PermanentState.forCreature(card)
        .withTapped(true)
        .addCounters("PLUS_ONE_PLUS_ONE", 2)
        .addCounters("CHARGE", 3)

      expect(state.isTapped).toBe(true)
      expect(state.getCounters("PLUS_ONE_PLUS_ONE")).toBe(2)
      expect(state.getCounters("CHARGE")).toBe(3)
      expect(state.getCurrentPower()).toBe(5)
    })

    test("non-creature can be tapped and have counters", () => {
      const state = PermanentState.forNonCreature()
        .withTapped(true)
        .addCounters("CHARGE", 5)
        .addCounters("LOYALTY", 3)

      expect(state.isTapped).toBe(true)
      expect(state.getCounters("CHARGE")).toBe(5)
      expect(state.getCounters("LOYALTY")).toBe(3)
    })
  })
})
