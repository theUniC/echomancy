import { describe, expect, test } from "vitest"
import { CombatState } from "../CombatState"

describe("CombatState Value Object", () => {
  describe("initial", () => {
    test("creates empty combat state", () => {
      const state = CombatState.initial()

      expect(state.hasAttackers()).toBe(false)
      expect(state.attackerCount).toBe(0)
    })
  })

  describe("fromSnapshot", () => {
    test("creates state from snapshot", () => {
      const snapshot = {
        attackerIds: ["attacker-1", "attacker-2"],
        blockerAssignments: { "attacker-1": "blocker-1" },
      }

      const state = CombatState.fromSnapshot(snapshot)

      expect(state.isAttacking("attacker-1")).toBe(true)
      expect(state.isAttacking("attacker-2")).toBe(true)
      expect(state.getBlockerFor("attacker-1")).toBe("blocker-1")
      expect(state.getBlockerFor("attacker-2")).toBeNull()
    })
  })

  describe("withAttacker", () => {
    test("returns new state with creature as attacker", () => {
      const state = CombatState.initial()

      const newState = state.withAttacker("creature-1")

      expect(newState.isAttacking("creature-1")).toBe(true)
      expect(state.isAttacking("creature-1")).toBe(false) // Original unchanged
    })

    test("can add multiple attackers", () => {
      const state = CombatState.initial()
        .withAttacker("creature-1")
        .withAttacker("creature-2")

      expect(state.attackerCount).toBe(2)
      expect(state.isAttacking("creature-1")).toBe(true)
      expect(state.isAttacking("creature-2")).toBe(true)
    })

    test("adding same attacker twice is idempotent", () => {
      const state = CombatState.initial()
        .withAttacker("creature-1")
        .withAttacker("creature-1")

      expect(state.attackerCount).toBe(1)
    })
  })

  describe("withoutAttacker", () => {
    test("returns new state without the attacker", () => {
      const state = CombatState.initial().withAttacker("creature-1")

      const newState = state.withoutAttacker("creature-1")

      expect(newState.isAttacking("creature-1")).toBe(false)
      expect(state.isAttacking("creature-1")).toBe(true) // Original unchanged
    })

    test("also removes blocker assignment for that attacker", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")

      const newState = state.withoutAttacker("attacker-1")

      expect(newState.getBlockerFor("attacker-1")).toBeNull()
    })
  })

  describe("withBlocker", () => {
    test("returns new state with blocker assigned to attacker", () => {
      const state = CombatState.initial().withAttacker("attacker-1")

      const newState = state.withBlocker("attacker-1", "blocker-1")

      expect(newState.getBlockerFor("attacker-1")).toBe("blocker-1")
      expect(state.getBlockerFor("attacker-1")).toBeNull() // Original unchanged
    })

    test("can reassign blocker", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")
        .withBlocker("attacker-1", "blocker-2")

      expect(state.getBlockerFor("attacker-1")).toBe("blocker-2")
    })
  })

  describe("withoutBlocker", () => {
    test("returns new state without the blocker assignment", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")

      const newState = state.withoutBlocker("attacker-1")

      expect(newState.getBlockerFor("attacker-1")).toBeNull()
      expect(newState.isAttacking("attacker-1")).toBe(true) // Attacker still attacking
    })
  })

  describe("isAttacking", () => {
    test("returns true for declared attacker", () => {
      const state = CombatState.initial().withAttacker("creature-1")

      expect(state.isAttacking("creature-1")).toBe(true)
    })

    test("returns false for non-attacker", () => {
      const state = CombatState.initial()

      expect(state.isAttacking("creature-1")).toBe(false)
    })
  })

  describe("isBlocked", () => {
    test("returns true when attacker has blocker", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")

      expect(state.isBlocked("attacker-1")).toBe(true)
    })

    test("returns false when attacker has no blocker", () => {
      const state = CombatState.initial().withAttacker("attacker-1")

      expect(state.isBlocked("attacker-1")).toBe(false)
    })
  })

  describe("isBlocking", () => {
    test("returns true when creature is blocking an attacker", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")

      expect(state.isBlocking("blocker-1")).toBe(true)
    })

    test("returns false when creature is not blocking", () => {
      const state = CombatState.initial().withAttacker("attacker-1")

      expect(state.isBlocking("blocker-1")).toBe(false)
    })
  })

  describe("getBlockedAttacker", () => {
    test("returns attacker ID when creature is blocking", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")

      expect(state.getBlockedAttacker("blocker-1")).toBe("attacker-1")
    })

    test("returns null when creature is not blocking", () => {
      const state = CombatState.initial()

      expect(state.getBlockedAttacker("blocker-1")).toBeNull()
    })
  })

  describe("getAttackerIds", () => {
    test("returns set of all attacker IDs", () => {
      const state = CombatState.initial()
        .withAttacker("creature-1")
        .withAttacker("creature-2")

      const ids = state.getAttackerIds()

      expect(ids.has("creature-1")).toBe(true)
      expect(ids.has("creature-2")).toBe(true)
      expect(ids.size).toBe(2)
    })
  })

  describe("hasAttackers", () => {
    test("returns true when there are attackers", () => {
      const state = CombatState.initial().withAttacker("creature-1")

      expect(state.hasAttackers()).toBe(true)
    })

    test("returns false when no attackers", () => {
      const state = CombatState.initial()

      expect(state.hasAttackers()).toBe(false)
    })
  })

  describe("getBlockerAssignments", () => {
    test("returns map of attacker to blocker", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withAttacker("attacker-2")
        .withBlocker("attacker-1", "blocker-1")

      const assignments = state.getBlockerAssignments()

      expect(assignments.get("attacker-1")).toBe("blocker-1")
      expect(assignments.has("attacker-2")).toBe(false)
    })
  })

  describe("clear", () => {
    test("returns empty combat state", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withBlocker("attacker-1", "blocker-1")

      const cleared = state.clear()

      expect(cleared.hasAttackers()).toBe(false)
      expect(cleared.getBlockerFor("attacker-1")).toBeNull()
    })

    test("does not modify original state", () => {
      const state = CombatState.initial().withAttacker("attacker-1")

      state.clear()

      expect(state.isAttacking("attacker-1")).toBe(true)
    })
  })

  describe("toSnapshot", () => {
    test("returns snapshot with all properties", () => {
      const state = CombatState.initial()
        .withAttacker("attacker-1")
        .withAttacker("attacker-2")
        .withBlocker("attacker-1", "blocker-1")

      const snapshot = state.toSnapshot()

      expect(snapshot.attackerIds).toContain("attacker-1")
      expect(snapshot.attackerIds).toContain("attacker-2")
      expect(snapshot.blockerAssignments["attacker-1"]).toBe("blocker-1")
    })
  })

  describe("immutability", () => {
    test("all with* methods return new instances", () => {
      const state = CombatState.initial()

      const state2 = state.withAttacker("creature-1")
      const state3 = state2.withBlocker("creature-1", "blocker-1")
      const state4 = state3.withoutBlocker("creature-1")
      const state5 = state4.withoutAttacker("creature-1")
      const state6 = state.clear()

      expect(state2).not.toBe(state)
      expect(state3).not.toBe(state2)
      expect(state4).not.toBe(state3)
      expect(state5).not.toBe(state4)
      expect(state6).not.toBe(state)
    })
  })
})
