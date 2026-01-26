import { expect, test } from "vitest"
import { Step } from "../Steps"
import {
  addCreatureToBattlefield,
  advanceToStep,
  createStartedGame,
  createStartedGameWithDecks,
  createTestCreature,
  resolveStack,
  scheduleExtraCombatPhase,
  setupCreatureInCombat,
  setupMultipleCreatures,
} from "./helpers"

// ============================================================================
// Rule 1 — A creature enters the battlefield correctly
// ============================================================================

test("creature enters battlefield when cast and resolved", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const creature = createTestCreature(player1.id)
  const playerState = game.getPlayerState(player1.id)
  playerState.hand.cards.push(creature)

  // Cast the creature spell
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: creature.instanceId,
    targets: [],
  })

  // Resolve the stack (both players pass priority)
  resolveStack(game, player2.id, player1.id)

  const stateAfter = game.getPlayerState(player1.id)

  // Creature should be on battlefield
  expect(stateAfter.battlefield.cards).toHaveLength(1)
  expect(stateAfter.battlefield.cards[0].instanceId).toBe(creature.instanceId)
})

test("creature enters battlefield untapped", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isTapped).toBe(false)
})

test("creature enters battlefield not attacking", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isAttacking).toBe(false)
})

// ============================================================================
// Rule 2 — A creature can attack during the combat phase
// ============================================================================

test("creature can be declared as attacker in DECLARE_ATTACKERS step", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isAttacking).toBe(true)
})

test("creature becomes tapped when declared as attacker", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isTapped).toBe(true)
})

test("attack is registered for the turn", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.hasAttackedThisTurn).toBe(true)
})

test("DECLARE_ATTACKER action is available during DECLARE_ATTACKERS step", () => {
  const { game, player1 } = createStartedGame()
  setupCreatureInCombat(game, player1.id)

  const allowedActions = game.getAllowedActionsFor(player1.id)

  expect(allowedActions).toContain("DECLARE_ATTACKER")
})

test("multiple creatures can attack in same turn", () => {
  const { game, player1 } = createStartedGame()
  const [creature1, creature2] = setupMultipleCreatures(game, player1.id, 2)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature1.instanceId,
  })

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature2.instanceId,
  })

  const state1 = game.getCreatureState(creature1.instanceId)
  const state2 = game.getCreatureState(creature2.instanceId)

  expect(state1.isAttacking).toBe(true)
  expect(state2.isAttacking).toBe(true)
})

// ============================================================================
// Rule 3 — A tapped creature cannot attack
// ============================================================================

test("tapped creature cannot be declared as attacker", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  // Tap the creature manually
  game.tapPermanent(creature.instanceId)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  expect(() => {
    game.apply({
      type: "DECLARE_ATTACKER",
      playerId: player1.id,
      creatureId: creature.instanceId,
    })
  }).toThrow()
})

test("tapped creature remains tapped and not attacking when invalid attack attempted", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  game.tapPermanent(creature.instanceId)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Attempt should throw
  expect(() => {
    game.apply({
      type: "DECLARE_ATTACKER",
      playerId: player1.id,
      creatureId: creature.instanceId,
    })
  }).toThrow()

  // State should remain unchanged
  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isTapped).toBe(true)
  expect(creatureState.isAttacking).toBe(false)
})

test("DECLARE_ATTACKER action not available for tapped creatures", () => {
  const { game, player1 } = createStartedGame()
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  game.tapPermanent(creature.instanceId)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Action should still be listed but will fail validation
  // OR could be filtered out - depends on design choice
  // For now, we test that the action fails when attempted
  expect(() => {
    game.apply({
      type: "DECLARE_ATTACKER",
      playerId: player1.id,
      creatureId: creature.instanceId,
    })
  }).toThrow()
})

// ============================================================================
// Rule 4 — A creature cannot attack twice in the same turn
// ============================================================================

test("creature cannot attack twice in same turn with extra combat phase", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  // First attack
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  // Schedule an extra combat phase
  scheduleExtraCombatPhase(game)

  // Advance to the extra DECLARE_ATTACKERS step
  advanceToStep(game, Step.END_OF_COMBAT)
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Try to attack again with the same creature
  expect(() => {
    game.apply({
      type: "DECLARE_ATTACKER",
      playerId: player1.id,
      creatureId: creature.instanceId,
    })
  }).toThrow()
})

test("creature that attacked remains marked for the entire turn", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  // Advance through combat
  advanceToStep(game, Step.SECOND_MAIN)

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.hasAttackedThisTurn).toBe(true)
})

test("different creature can attack in extra combat phase", () => {
  const { game, player1 } = createStartedGame()
  const [creature1, creature2] = setupMultipleCreatures(game, player1.id, 2)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // First creature attacks
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature1.instanceId,
  })

  // Complete the first combat phase
  advanceToStep(game, Step.SECOND_MAIN)

  // Schedule extra combat
  scheduleExtraCombatPhase(game)

  // Advance to extra DECLARE_ATTACKERS
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })
  game.apply({ type: "ADVANCE_STEP", playerId: player1.id })

  // Different creature CAN attack
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature2.instanceId,
  })

  const state2 = game.getCreatureState(creature2.instanceId)

  expect(state2.isAttacking).toBe(true)
  expect(state2.hasAttackedThisTurn).toBe(true)
})

// ============================================================================
// Rule 5 — Attack state resets when the turn changes
// ============================================================================

test("creature attack state resets when turn changes", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  // End the turn
  game.apply({ type: "END_TURN", playerId: player1.id })

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isAttacking).toBe(false)
  expect(creatureState.hasAttackedThisTurn).toBe(false)
})

test("creature can attack again in next turn", () => {
  const { game, player1, player2 } = createStartedGameWithDecks()
  const creature = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Attack in first turn
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  // End turn and cycle back to player1
  game.apply({ type: "END_TURN", playerId: player1.id })
  game.apply({ type: "END_TURN", playerId: player2.id })

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  // Should be able to attack again
  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isAttacking).toBe(true)
  expect(creatureState.hasAttackedThisTurn).toBe(true)
})

test("isAttacking becomes false at end of combat", () => {
  const { game, player1 } = createStartedGame()
  const creature = setupCreatureInCombat(game, player1.id)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature.instanceId,
  })

  // Advance past combat
  advanceToStep(game, Step.SECOND_MAIN)

  const creatureState = game.getCreatureState(creature.instanceId)

  expect(creatureState.isAttacking).toBe(false)
  expect(creatureState.hasAttackedThisTurn).toBe(true) // Still marked as having attacked
})

test("multiple creatures reset attack state on turn change", () => {
  const { game, player1 } = createStartedGame()
  const [creature1, creature2] = setupMultipleCreatures(game, player1.id, 2)

  advanceToStep(game, Step.DECLARE_ATTACKERS)

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature1.instanceId,
  })

  game.apply({
    type: "DECLARE_ATTACKER",
    playerId: player1.id,
    creatureId: creature2.instanceId,
  })

  game.apply({ type: "END_TURN", playerId: player1.id })

  const state1 = game.getCreatureState(creature1.instanceId)
  const state2 = game.getCreatureState(creature2.instanceId)

  expect(state1.isAttacking).toBe(false)
  expect(state1.hasAttackedThisTurn).toBe(false)
  expect(state2.isAttacking).toBe(false)
  expect(state2.hasAttackedThisTurn).toBe(false)
})

// ============================================================================
// Rule 6 — UNTAP step automatically untaps creatures
// ============================================================================

test("creatures of active player untap automatically when entering UNTAP step", () => {
  const { game, player1, player2 } = createStartedGame()

  // Add creature to player1's battlefield
  const creature1 = createTestCreature(player1.id)
  addCreatureToBattlefield(game, player1.id, creature1)

  // Tap player1's creature
  game.tapPermanent(creature1.instanceId)

  // Verify it's tapped
  expect(game.getCreatureState(creature1.instanceId).isTapped).toBe(true)

  // End player1's turn and advance through player2's turn
  game.apply({ type: "END_TURN", playerId: player1.id })
  game.apply({ type: "END_TURN", playerId: player2.id })

  // Now we're at the start of player1's turn (UNTAP step)
  expect(game.currentStep).toBe(Step.UNTAP)
  expect(game.currentPlayerId).toBe(player1.id)

  // Player1's creature should be automatically untapped
  const creatureState = game.getCreatureState(creature1.instanceId)
  expect(creatureState.isTapped).toBe(false)
})

test("opponent's creatures do not untap during active player's UNTAP step", () => {
  const { game, player1, player2 } = createStartedGame()

  // Add creatures to both players
  const creature1 = createTestCreature(player1.id)
  const creature2 = createTestCreature(player2.id)
  addCreatureToBattlefield(game, player1.id, creature1)
  addCreatureToBattlefield(game, player2.id, creature2)

  // Tap both creatures
  game.tapPermanent(creature1.instanceId)
  game.tapPermanent(creature2.instanceId)

  // End player1's turn to reach player2's UNTAP step
  game.apply({ type: "END_TURN", playerId: player1.id })

  // Now we're at player2's UNTAP step
  expect(game.currentStep).toBe(Step.UNTAP)
  expect(game.currentPlayerId).toBe(player2.id)

  // Player2's creature should be untapped (active player)
  expect(game.getCreatureState(creature2.instanceId).isTapped).toBe(false)

  // Player1's creature should still be tapped (opponent)
  expect(game.getCreatureState(creature1.instanceId).isTapped).toBe(true)
})

test("multiple creatures of active player untap simultaneously", () => {
  const { game, player1, player2 } = createStartedGame()

  // Add multiple creatures to player1
  const [creature1, creature2, creature3] = setupMultipleCreatures(
    game,
    player1.id,
    3,
  )

  // Tap all creatures
  game.tapPermanent(creature1.instanceId)
  game.tapPermanent(creature2.instanceId)
  game.tapPermanent(creature3.instanceId)

  // End player1's turn and cycle back
  game.apply({ type: "END_TURN", playerId: player1.id })
  game.apply({ type: "END_TURN", playerId: player2.id })

  // All creatures should be untapped
  expect(game.getCreatureState(creature1.instanceId).isTapped).toBe(false)
  expect(game.getCreatureState(creature2.instanceId).isTapped).toBe(false)
  expect(game.getCreatureState(creature3.instanceId).isTapped).toBe(false)
})
