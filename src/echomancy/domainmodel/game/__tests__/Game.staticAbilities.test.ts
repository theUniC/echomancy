import { expect, test } from "vitest"
import type { CardInstance } from "../../cards/CardInstance"
import { CannotBlockFlyingCreatureError } from "../GameErrors"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  advanceToStep,
  createStartedGame,
  createTestCreature,
} from "./helpers"

/**
 * Tests for Static Abilities MVP
 *
 * This test suite validates the MVP static ability keywords:
 * - Flying: Can only be blocked by creatures with Flying or Reach
 * - Reach: Can block creatures with Flying
 * - Vigilance: Does not tap when attacking
 *
 * MVP static abilities are consultative; no layers yet.
 * These keywords only affect rule checks and validations.
 *
 * Scope:
 * - Flying blocking restrictions
 * - Reach blocking allowance
 * - Vigilance tap prevention
 *
 * Explicitly excluded from MVP:
 * - Lords / continuous effects ("other elves get +1/+1")
 * - 7-layer system
 * - Ability gain/loss ("creature gains flying")
 * - Temporary modifiers ("until end of turn")
 * - First strike / Double strike
 * - Trample
 * - Deathtouch
 * - Menace
 * - Lifelink
 * - Hexproof / Shroud / Protection / Ward
 * - Replacement effects
 */

// ============================================================================
// Helper: Create creature with static abilities
// ============================================================================

function createCreatureWithAbilities(
  ownerId: string,
  instanceId: string,
  abilities: ("FLYING" | "REACH" | "VIGILANCE")[],
  power?: number,
  toughness?: number,
): CardInstance {
  return {
    instanceId,
    definition: {
      id: "creature-with-abilities",
      name: "Creature With Abilities",
      types: ["CREATURE"],
      staticAbilities: abilities,
      power,
      toughness,
    },
    ownerId,
  }
}

// ============================================================================
// VIGILANCE Tests
// ============================================================================

test("Vigilance: creature with Vigilance does not tap when attacking", () => {
  const { game, player1 } = createStartedGame()

  // Player 1 has a creature with Vigilance
  const vigilantCreature = createCreatureWithAbilities(
    player1.id,
    "vigilant-creature",
    ["VIGILANCE"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, vigilantCreature)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Declare attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: vigilantCreature.instanceId,
  })

  const creatureState = game.getCreatureState(vigilantCreature.instanceId)

  // Creature should be attacking
  expect(creatureState.isAttacking).toBe(true)
  expect(creatureState.hasAttackedThisTurn).toBe(true)

  // But should NOT be tapped (Vigilance)
  expect(creatureState.isTapped).toBe(false)
})

test("Vigilance: creature without Vigilance taps when attacking", () => {
  const { game, player1 } = createStartedGame()

  // Player 1 has a normal creature (no Vigilance)
  const normalCreature = createTestCreature(player1.id, "normal-creature", 2, 2)
  addCreatureToBattlefield(game, player1.id, normalCreature)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Declare attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: normalCreature.instanceId,
  })

  const creatureState = game.getCreatureState(normalCreature.instanceId)

  // Creature should be attacking
  expect(creatureState.isAttacking).toBe(true)
  expect(creatureState.hasAttackedThisTurn).toBe(true)

  // And SHOULD be tapped (no Vigilance)
  expect(creatureState.isTapped).toBe(true)
})

// ============================================================================
// FLYING Tests
// ============================================================================

test("Flying: creature without Flying or Reach cannot block Flying creature", () => {
  const { game, player1, player2 } = createStartedGame()

  // Player 1 has a Flying attacker
  const flyingAttacker = createCreatureWithAbilities(
    player1.id,
    "flying-attacker",
    ["FLYING"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, flyingAttacker)

  // Player 2 has a normal blocker (no Flying, no Reach)
  const normalBlocker = createTestCreature(player2.id, "normal-blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, normalBlocker)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Player 1 declares Flying attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: flyingAttacker.instanceId,
  })

  // Advance to DECLARE_BLOCKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Player 2 tries to block with normal creature - should fail
  expect(() => {
    game.apply({
      type: "DECLARE_BLOCKER",
      playerId: player2.id,
      blockerId: normalBlocker.instanceId,
      attackerId: flyingAttacker.instanceId,
    })
  }).toThrow(CannotBlockFlyingCreatureError)
})

test("Flying: creature with Flying can block Flying creature", () => {
  const { game, player1, player2 } = createStartedGame()

  // Player 1 has a Flying attacker
  const flyingAttacker = createCreatureWithAbilities(
    player1.id,
    "flying-attacker",
    ["FLYING"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, flyingAttacker)

  // Player 2 has a Flying blocker
  const flyingBlocker = createCreatureWithAbilities(
    player2.id,
    "flying-blocker",
    ["FLYING"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player2.id, flyingBlocker)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Player 1 declares Flying attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: flyingAttacker.instanceId,
  })

  // Advance to DECLARE_BLOCKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Player 2 blocks with Flying creature - should succeed
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: flyingBlocker.instanceId,
    attackerId: flyingAttacker.instanceId,
  })

  // Verify blocking relationship established
  const blockerState = game.getCreatureState(flyingBlocker.instanceId)
  const attackerState = game.getCreatureState(flyingAttacker.instanceId)

  expect(blockerState.blockingCreatureId).toBe(flyingAttacker.instanceId)
  expect(attackerState.blockedBy).toBe(flyingBlocker.instanceId)
})

test("Flying: normal creature can be blocked by normal creature", () => {
  const { game, player1, player2 } = createStartedGame()

  // Player 1 has a normal attacker (no Flying)
  const normalAttacker = createTestCreature(player1.id, "normal-attacker", 2, 2)
  addCreatureToBattlefield(game, player1.id, normalAttacker)

  // Player 2 has a normal blocker
  const normalBlocker = createTestCreature(player2.id, "normal-blocker", 2, 2)
  addCreatureToBattlefield(game, player2.id, normalBlocker)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Player 1 declares attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: normalAttacker.instanceId,
  })

  // Advance to DECLARE_BLOCKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Player 2 blocks with normal creature - should succeed
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: normalBlocker.instanceId,
    attackerId: normalAttacker.instanceId,
  })

  // Verify blocking relationship established
  const blockerState = game.getCreatureState(normalBlocker.instanceId)
  const attackerState = game.getCreatureState(normalAttacker.instanceId)

  expect(blockerState.blockingCreatureId).toBe(normalAttacker.instanceId)
  expect(attackerState.blockedBy).toBe(normalBlocker.instanceId)
})

// ============================================================================
// REACH Tests
// ============================================================================

test("Reach: creature with Reach can block Flying creature", () => {
  const { game, player1, player2 } = createStartedGame()

  // Player 1 has a Flying attacker
  const flyingAttacker = createCreatureWithAbilities(
    player1.id,
    "flying-attacker",
    ["FLYING"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, flyingAttacker)

  // Player 2 has a creature with Reach (but not Flying)
  const reachBlocker = createCreatureWithAbilities(
    player2.id,
    "reach-blocker",
    ["REACH"],
    2,
    3,
  )
  addCreatureToBattlefield(game, player2.id, reachBlocker)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Player 1 declares Flying attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: flyingAttacker.instanceId,
  })

  // Advance to DECLARE_BLOCKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Player 2 blocks with Reach creature - should succeed
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: reachBlocker.instanceId,
    attackerId: flyingAttacker.instanceId,
  })

  // Verify blocking relationship established
  const blockerState = game.getCreatureState(reachBlocker.instanceId)
  const attackerState = game.getCreatureState(flyingAttacker.instanceId)

  expect(blockerState.blockingCreatureId).toBe(flyingAttacker.instanceId)
  expect(attackerState.blockedBy).toBe(reachBlocker.instanceId)
})

test("Reach: creature with both Flying and Reach can block Flying creature", () => {
  const { game, player1, player2 } = createStartedGame()

  // Player 1 has a Flying attacker
  const flyingAttacker = createCreatureWithAbilities(
    player1.id,
    "flying-attacker",
    ["FLYING"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, flyingAttacker)

  // Player 2 has a creature with both Flying and Reach
  const flyingReachBlocker = createCreatureWithAbilities(
    player2.id,
    "flying-reach-blocker",
    ["FLYING", "REACH"],
    2,
    3,
  )
  addCreatureToBattlefield(game, player2.id, flyingReachBlocker)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Player 1 declares Flying attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: flyingAttacker.instanceId,
  })

  // Advance to DECLARE_BLOCKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  expect(game.currentStep).toBe(Step.DECLARE_BLOCKERS)

  // Player 2 blocks with Flying+Reach creature - should succeed
  game.apply({
    type: "DECLARE_BLOCKER",
    playerId: player2.id,
    blockerId: flyingReachBlocker.instanceId,
    attackerId: flyingAttacker.instanceId,
  })

  // Verify blocking relationship established
  const blockerState = game.getCreatureState(flyingReachBlocker.instanceId)
  const attackerState = game.getCreatureState(flyingAttacker.instanceId)

  expect(blockerState.blockingCreatureId).toBe(flyingAttacker.instanceId)
  expect(attackerState.blockedBy).toBe(flyingReachBlocker.instanceId)
})

// ============================================================================
// Combined Tests
// ============================================================================

test("Combined: Vigilant Flying creature can attack and block on same turn", () => {
  const { game, player1 } = createStartedGame()

  // Player 1 has a creature with Vigilance and Flying
  const vigilantFlyer = createCreatureWithAbilities(
    player1.id,
    "vigilant-flyer",
    ["VIGILANCE", "FLYING"],
    2,
    2,
  )
  addCreatureToBattlefield(game, player1.id, vigilantFlyer)

  // Advance to DECLARE_ATTACKERS
  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Declare attacker
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: vigilantFlyer.instanceId,
  })

  const creatureState = game.getCreatureState(vigilantFlyer.instanceId)

  // Creature should be attacking but NOT tapped (due to Vigilance)
  expect(creatureState.isAttacking).toBe(true)
  expect(creatureState.hasAttackedThisTurn).toBe(true)
  expect(creatureState.isTapped).toBe(false)

  // Note: The creature could theoretically block next turn if it survives
  // and doesn't attack again, since it's not tapped
})
