import { expect, test } from "vitest"
import {
  AttackerAlreadyBlockedError,
  CreatureAlreadyBlockingError,
  InvalidPlayerActionError,
  TappedCreatureCannotBlockError,
} from "../GameErrors"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  advanceToStep,
  createStartedGame,
  createTestCreature,
} from "./helpers"

/**
 * Tests for Combat Resolution MVP
 *
 * This test suite validates the combat resolution system as defined in the
 * Combat Resolution MVP Contract.
 *
 * Scope:
 * - Declare blockers
 * - Damage assignment
 * - Damage resolution
 * - Creature destruction
 * - Damage to players
 * - Damage cleanup
 *
 * Explicitly excluded from MVP:
 * - First strike / Double strike
 * - Trample
 * - Deathtouch
 * - Indestructible
 * - Damage prevention
 * - Regeneration
 * - Planeswalker combat
 */

// ============================================================================
// Creature vs Creature Combat
// ============================================================================

test("2/2 blocks 2/2 - both creatures die", () => {
  const { game, player1, player2 } = createStartedGame()

  // Player 1 has a 2/2 attacker
  const attacker = createTestCreature(player1.id, "attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  // Player 2 has a 2/2 blocker
  const blocker = createTestCreature(player2.id, "blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, blocker)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Player 1 declares attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  // Advance to DECLARE_BLOCKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Player 2 declares blocker
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  // Advance to COMBAT_DAMAGE (this resolves damage automatically)
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.COMBAT_DAMAGE)

  // Both creatures should be in graveyards
  const player1Graveyard = game.getGraveyard(player1.id)
  const player2Graveyard = game.getGraveyard(player2.id)

  expect(player1Graveyard).toHaveLength(1)
  expect(player1Graveyard[0].instanceId).toBe(attacker.instanceId)

  expect(player2Graveyard).toHaveLength(1)
  expect(player2Graveyard[0].instanceId).toBe(blocker.instanceId)

  // Both battlefields should be empty
  expect(game.getPlayerState(player1.id).battlefield.cards).toHaveLength(0)
  expect(game.getPlayerState(player2.id).battlefield.cards).toHaveLength(0)
})

test("3/3 blocks 2/2 - 2/2 dies, 3/3 survives", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 3, 3)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Attacker should be dead
  const player1Graveyard = game.getGraveyard(player1.id)
  expect(player1Graveyard).toHaveLength(1)
  expect(player1Graveyard[0].instanceId).toBe(attacker.instanceId)

  // Blocker should still be alive
  const player2Battlefield = game.getPlayerState(player2.id).battlefield.cards
  expect(player2Battlefield).toHaveLength(1)
  expect(player2Battlefield[0].instanceId).toBe(blocker.instanceId)

  // Blocker should have 2 damage marked
  const blockerState = game.getCreatureState(blocker.instanceId)
  expect(blockerState.damageMarkedThisTurn).toBe(2)
})

test("+1/+1 counter affects combat outcome", () => {
  const { game, player1, player2 } = createStartedGame()

  // 1/1 attacker with a +1/+1 counter (becomes 2/2)
  const attacker = createTestCreature(player1.id, "attacker", 1, 1)
  addCreatureToBattlefield(game, player1.id, attacker)
  game.addCounters(attacker.instanceId, "PLUS_ONE_PLUS_ONE", 1)

  // 2/2 blocker
  const blocker = createTestCreature(player2.id, "blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Both should die (2 damage to each other)
  expect(game.getGraveyard(player1.id)).toHaveLength(1)
  expect(game.getGraveyard(player2.id)).toHaveLength(1)
})

// ============================================================================
// Creature vs Player Damage
// ============================================================================

test("unblocked attacker deals damage to defending player", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 3, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  const initialLife = player2.lifeTotal

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  // Advance through declare blockers (no blocks)
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Advance to combat damage
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Player 2 should have taken 3 damage
  expect(player2.lifeTotal).toBe(initialLife - 3)
})

test("multiple unblocked attackers stack damage", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker1 = createTestCreature(player1.id, "attacker1", 2, 2)
  const attacker2 = createTestCreature(player1.id, "attacker2", 3, 3)
  addCreatureToBattlefield(game, player1.id, attacker1)
  addCreatureToBattlefield(game, player1.id, attacker2)

  const initialLife = player2.lifeTotal

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

  // Advance through declare blockers (no blocks)
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Advance to combat damage
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Player 2 should have taken 5 damage (2 + 3)
  expect(player2.lifeTotal).toBe(initialLife - 5)
})

test("blocked attacker does not deal damage to player", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 3, 3)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 1, 1)
  addCreatureToBattlefield(game, player2.id, blocker)

  const initialLife = player2.lifeTotal

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Player 2 should not have taken any damage (blocker absorbed it)
  expect(player2.lifeTotal).toBe(initialLife)
})

// ============================================================================
// Damage Timing
// ============================================================================

test("damage is simultaneous - both creatures die even if one has more power", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 5, 1)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 1, 5)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Both should die (damage is simultaneous)
  expect(game.getGraveyard(player1.id)).toHaveLength(1)
  expect(game.getGraveyard(player2.id)).toHaveLength(1)
})

test("damage is cleared at CLEANUP step", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 2, 3)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 1, 5)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Both creatures should survive with damage marked
  expect(game.getCreatureState(attacker.instanceId).damageMarkedThisTurn).toBe(
    1,
  )
  expect(game.getCreatureState(blocker.instanceId).damageMarkedThisTurn).toBe(2)

  // Advance to CLEANUP
  advanceToStep(game, Step.CLEANUP)

  // Damage should be cleared
  expect(game.getCreatureState(attacker.instanceId).damageMarkedThisTurn).toBe(
    0,
  )
  expect(game.getCreatureState(blocker.instanceId).damageMarkedThisTurn).toBe(0)
})

test("damage does not persist across turns", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 1, 3)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 1, 3)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Both creatures have 1 damage
  expect(game.getCreatureState(attacker.instanceId).damageMarkedThisTurn).toBe(
    1,
  )
  expect(game.getCreatureState(blocker.instanceId).damageMarkedThisTurn).toBe(1)

  // End turn
  game.apply({ type: "END_TURN", playerId: player1.id })

  // Damage should be cleared at next player's turn
  expect(game.getCreatureState(attacker.instanceId).damageMarkedThisTurn).toBe(
    0,
  )
  expect(game.getCreatureState(blocker.instanceId).damageMarkedThisTurn).toBe(0)
})

// ============================================================================
// Edge Cases
// ============================================================================

test("only defending player can declare blockers", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Active player (player1) attempts to declare blocker - should fail
  expect(() => {
    game.apply({
      type: "DECLARE_BLOCKER",
      playerId: player1.id, // Active player trying to block
      blockerId: blocker.instanceId,
      attackerId: attacker.instanceId,
    })
  }).toThrow(InvalidPlayerActionError)

  // Defending player (player2) can declare blocker - should succeed
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id, // Defending player
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  // Verify blocking was successful
  const blockerState = game.getCreatureState(blocker.instanceId)
  expect(blockerState.blockingCreatureId).toBe(attacker.instanceId)
})

test("tapped creature cannot block", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, blocker)
  game.tapPermanent(blocker.instanceId)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Attempt to block with tapped creature should fail
  expect(() => {
    game.apply({
      type: "DECLARE_BLOCKER",
      playerId: player2.id,
      blockerId: blocker.instanceId,
      attackerId: attacker.instanceId,
    })
  }).toThrow(TappedCreatureCannotBlockError)
})

test("creature cannot block twice", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker1 = createTestCreature(player1.id, "attacker1", 2, 2)
  const attacker2 = createTestCreature(player1.id, "attacker2", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker1)
  addCreatureToBattlefield(game, player1.id, attacker2)

  const blocker = createTestCreature(player2.id, "blocker", 2, 2)
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

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Block first attacker
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker1.instanceId,
  })

  // Attempt to block second attacker should fail
  expect(() => {
    game.apply({
      type: "DECLARE_BLOCKER",
      playerId: player2.id,
      blockerId: blocker.instanceId,
      attackerId: attacker2.instanceId,
    })
  }).toThrow(CreatureAlreadyBlockingError)
})

test("attacker cannot be blocked twice (MVP: one blocker per attacker)", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker1 = createTestCreature(player2.id, "blocker1", 2, 2)
  const blocker2 = createTestCreature(player2.id, "blocker2", 2, 2)
  addCreatureToBattlefield(game, player2.id, blocker1)
  addCreatureToBattlefield(game, player2.id, blocker2)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // First blocker blocks successfully
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker1.instanceId,
    attackerId: attacker.instanceId,
  })

  // Attempt to block same attacker with second blocker should fail
  expect(() => {
    game.apply({
      type: "DECLARE_BLOCKER",
      playerId: player2.id,
      blockerId: blocker2.instanceId,
      attackerId: attacker.instanceId,
    })
  }).toThrow(AttackerAlreadyBlockedError)
})

test("dead creatures are removed before next step", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Both creatures should be in graveyards immediately
  expect(game.getPlayerState(player1.id).battlefield.cards).toHaveLength(0)
  expect(game.getPlayerState(player2.id).battlefield.cards).toHaveLength(0)
  expect(game.getGraveyard(player1.id)).toHaveLength(1)
  expect(game.getGraveyard(player2.id)).toHaveLength(1)
})

test("0-power creature deals no damage", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 0, 3)
  addCreatureToBattlefield(game, player1.id, attacker)

  const initialLife = player2.lifeTotal

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Player 2 should take no damage
  expect(player2.lifeTotal).toBe(initialLife)
})

test("combat state resets at end of combat", () => {
  const { game, player1, player2 } = createStartedGame()

  const attacker = createTestCreature(player1.id, "attacker", 3, 3)
  addCreatureToBattlefield(game, player1.id, attacker)

  const blocker = createTestCreature(player2.id, "blocker", 1, 5)
  addCreatureToBattlefield(game, player2.id, blocker)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: attacker.instanceId,
  })

  const attackerState = game.getCreatureState(attacker.instanceId)
  expect(attackerState.isAttacking).toBe(true)

  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: blocker.instanceId,
    attackerId: attacker.instanceId,
  })

  const blockerState = game.getCreatureState(blocker.instanceId)
  expect(blockerState.blockingCreatureId).toBe(attacker.instanceId)
  // Re-fetch attacker state after blocker declared (snapshot is immutable)
  const attackerStateAfterBlocking = game.getCreatureState(attacker.instanceId)
  expect(attackerStateAfterBlocking.blockedBy).toBe(blocker.instanceId)

  // Advance past END_OF_COMBAT
  advanceToStep(game, Step.SECOND_MAIN)

  // Combat state should be cleared
  const attackerStateAfter = game.getCreatureState(attacker.instanceId)
  const blockerStateAfter = game.getCreatureState(blocker.instanceId)

  expect(attackerStateAfter.isAttacking).toBe(false)
  expect(attackerStateAfter.blockedBy).toBe(null)
  expect(blockerStateAfter.blockingCreatureId).toBe(null)
})
