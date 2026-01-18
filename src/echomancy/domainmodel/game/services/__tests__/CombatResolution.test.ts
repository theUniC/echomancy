import { describe, expect, test } from "vitest"
import {
  addCreatureToBattlefield,
  advanceToStep,
  createStartedGame,
  createTestCreature,
} from "../../__tests__/helpers"
import { Step } from "../../Steps"
import { CombatResolution } from "../CombatResolution"

/**
 * CombatResolution Service Tests
 *
 * These tests verify the pure damage calculation logic.
 * IMPORTANT: Tests stop at DECLARE_BLOCKERS step to avoid triggering
 * Game's automatic combat resolution (which happens on entering COMBAT_DAMAGE).
 */
describe("CombatResolution Service", () => {
  describe("calculateDamageAssignments", () => {
    test("returns empty array when no creatures are attacking", () => {
      const { game, player1 } = createStartedGame()
      const creature = createTestCreature(player1.id, "creature-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, creature)
      // Stay at a step before combat resolution triggers
      advanceToStep(game, Step.DECLARE_ATTACKERS)

      const assignments = CombatResolution.calculateDamageAssignments(game)

      expect(assignments).toEqual([])
    })

    test("unblocked attacker deals damage to defending player", () => {
      const { game, player1, player2 } = createStartedGame()
      const attacker = createTestCreature(player1.id, "attacker-1", 3, 3)
      addCreatureToBattlefield(game, player1.id, attacker)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker.instanceId,
      })

      // Stay at DECLARE_BLOCKERS - don't advance to COMBAT_DAMAGE
      // which would trigger automatic resolution
      advanceToStep(game, Step.DECLARE_BLOCKERS)

      const assignments = CombatResolution.calculateDamageAssignments(game)

      expect(assignments).toHaveLength(1)
      expect(assignments[0]).toEqual({
        targetId: player2.id,
        amount: 3,
        isPlayer: true,
      })
    })

    test("blocked attacker and blocker deal damage to each other", () => {
      const { game, player1, player2 } = createStartedGame()
      const attacker = createTestCreature(player1.id, "attacker-1", 4, 4)
      const blocker = createTestCreature(player2.id, "blocker-1", 2, 2)
      addCreatureToBattlefield(game, player1.id, attacker)
      addCreatureToBattlefield(game, player2.id, blocker)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker.instanceId,
      })

      advanceToStep(game, Step.DECLARE_BLOCKERS)
      game.apply({
        type: "DECLARE_BLOCKER",
        playerId: player2.id,
        blockerId: blocker.instanceId,
        attackerId: attacker.instanceId,
      })

      // Stay at DECLARE_BLOCKERS - don't advance to COMBAT_DAMAGE
      const assignments = CombatResolution.calculateDamageAssignments(game)

      expect(assignments).toHaveLength(2)
      // Attacker damages blocker
      expect(assignments).toContainEqual({
        targetId: blocker.instanceId,
        amount: 4,
        isPlayer: false,
      })
      // Blocker damages attacker
      expect(assignments).toContainEqual({
        targetId: attacker.instanceId,
        amount: 2,
        isPlayer: false,
      })
    })

    test("multiple attackers deal damage correctly", () => {
      const { game, player1, player2 } = createStartedGame()
      const attacker1 = createTestCreature(player1.id, "attacker-1", 2, 2)
      const attacker2 = createTestCreature(player1.id, "attacker-2", 3, 3)
      addCreatureToBattlefield(game, player1.id, attacker1)
      addCreatureToBattlefield(game, player1.id, attacker2)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker1.instanceId,
      })
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker2.instanceId,
      })

      // Stay at DECLARE_BLOCKERS
      advanceToStep(game, Step.DECLARE_BLOCKERS)

      const assignments = CombatResolution.calculateDamageAssignments(game)

      expect(assignments).toHaveLength(2)
      // Both deal damage to player2
      expect(assignments).toContainEqual({
        targetId: player2.id,
        amount: 2,
        isPlayer: true,
      })
      expect(assignments).toContainEqual({
        targetId: player2.id,
        amount: 3,
        isPlayer: true,
      })
    })

    test("mixed blocked and unblocked attackers", () => {
      const { game, player1, player2 } = createStartedGame()
      const attacker1 = createTestCreature(player1.id, "attacker-1", 2, 2)
      const attacker2 = createTestCreature(player1.id, "attacker-2", 3, 3)
      const blocker = createTestCreature(player2.id, "blocker-1", 1, 1)
      addCreatureToBattlefield(game, player1.id, attacker1)
      addCreatureToBattlefield(game, player1.id, attacker2)
      addCreatureToBattlefield(game, player2.id, blocker)

      advanceToStep(game, Step.DECLARE_ATTACKERS)
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker1.instanceId,
      })
      game.apply({
        type: "DECLARE_ATTACKER",
        playerId: player1.id,
        creatureId: attacker2.instanceId,
      })

      advanceToStep(game, Step.DECLARE_BLOCKERS)
      // Block attacker1, leave attacker2 unblocked
      game.apply({
        type: "DECLARE_BLOCKER",
        playerId: player2.id,
        blockerId: blocker.instanceId,
        attackerId: attacker1.instanceId,
      })

      // Stay at DECLARE_BLOCKERS - don't advance to COMBAT_DAMAGE
      const assignments = CombatResolution.calculateDamageAssignments(game)

      // attacker1 <-> blocker combat (2 assignments)
      // attacker2 -> player2 (1 assignment)
      expect(assignments).toHaveLength(3)

      // Unblocked attacker2 hits player
      expect(assignments).toContainEqual({
        targetId: player2.id,
        amount: 3,
        isPlayer: true,
      })

      // Blocked combat
      expect(assignments).toContainEqual({
        targetId: blocker.instanceId,
        amount: 2,
        isPlayer: false,
      })
      expect(assignments).toContainEqual({
        targetId: attacker1.instanceId,
        amount: 1,
        isPlayer: false,
      })
    })
  })
})
