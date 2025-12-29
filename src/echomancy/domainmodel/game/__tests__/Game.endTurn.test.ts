import { expect, test } from "vitest"
import { InvalidEndTurnError, InvalidPlayerActionError } from "../GameErrors"
import { Step } from "../Steps"
import {
  addSpellToHand,
  advanceToStep,
  createSpell,
  createStartedGame,
} from "./helpers"

// =============================================================================
// END_TURN INTENT RECORDING
// =============================================================================

test("END_TURN records auto-pass intent (observable when stack is non-empty)", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  expect(game.isPlayerInAutoPass(player1.id)).toBe(false)

  // Cast a spell so there's something on the stack
  // This prevents immediate auto-advance so we can observe the intent
  const spell = createSpell("test-spell", "Test Spell", player1.id)
  addSpellToHand(game, player1.id, spell)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: [],
  })

  game.apply({ type: "END_TURN", playerId: player1.id })

  // Auto-pass intent is recorded
  expect(game.isPlayerInAutoPass(player1.id)).toBe(true)
})

test("END_TURN is only available to active player", () => {
  const { game, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // player2 is not the active player
  expect(() => {
    game.apply({ type: "END_TURN", playerId: player2.id })
  }).toThrow(InvalidPlayerActionError)
})

test("END_TURN throws error during CLEANUP step", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.CLEANUP)

  expect(() => {
    game.apply({ type: "END_TURN", playerId: player1.id })
  }).toThrow(InvalidEndTurnError)
})

// =============================================================================
// AUTO-PASS CLEARS AT TURN START
// =============================================================================

test("auto-pass intent clears at start of new turn", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Player 1 ends their turn
  game.apply({ type: "END_TURN", playerId: player1.id })

  // After END_TURN, we should be on player2's turn
  expect(game.currentPlayerId).toBe(player2.id)

  // Player 1's auto-pass intent should be cleared
  expect(game.isPlayerInAutoPass(player1.id)).toBe(false)
})

// =============================================================================
// AUTO-PASS WITH EMPTY STACK
// =============================================================================

test("END_TURN with empty stack advances through all steps to next turn", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  game.apply({ type: "END_TURN", playerId: player1.id })

  // Should have advanced to player2's turn
  expect(game.currentPlayerId).toBe(player2.id)
  expect(game.currentStep).toBe(Step.UNTAP)
})

test("END_TURN advances through combat phases correctly", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.BEGINNING_OF_COMBAT)

  game.apply({ type: "END_TURN", playerId: player1.id })

  // Should have advanced to player2's turn
  expect(game.currentPlayerId).toBe(player2.id)
  expect(game.currentStep).toBe(Step.UNTAP)
})

// =============================================================================
// AUTO-PASS WITH SPELLS ON STACK
// =============================================================================

test("END_TURN with spell on stack gives opponent priority to respond", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Player 1 casts a spell
  const spell = createSpell("test-spell", "Test Spell", player1.id)
  addSpellToHand(game, player1.id, spell)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: [],
  })

  // Spell is on stack, priority goes to player2
  expect(game.getStack()).toHaveLength(1)

  // Now player1 wants to end turn
  game.apply({ type: "END_TURN", playerId: player1.id })

  // Player 1 is in auto-pass, but opponent should still have priority
  expect(game.isPlayerInAutoPass(player1.id)).toBe(true)

  // Player 2 should be able to respond
  const player2Actions = game.getAllowedActionsFor(player2.id)
  expect(player2Actions).toContain("PASS_PRIORITY")

  // Stack should still exist (opponent hasn't passed yet)
  expect(game.getStack()).toHaveLength(1)
})

test("auto-pass player automatically passes when receiving priority", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Player 1 casts a spell
  const spell = createSpell("test-spell", "Test Spell", player1.id)
  addSpellToHand(game, player1.id, spell)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: [],
  })

  // Player 1 ends turn (sets auto-pass)
  game.apply({ type: "END_TURN", playerId: player1.id })

  // Player 2 passes priority
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })

  // Now player 1 would normally get priority, but they're in auto-pass
  // So they automatically pass, and since both have passed, stack resolves
  expect(game.getStack()).toHaveLength(0)

  // Spell should be in graveyard (it resolved)
  const graveyard = game.getGraveyard(player1.id)
  expect(graveyard).toHaveLength(1)
})

test("auto-pass continues after stack resolution", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Player 1 casts a spell
  const spell = createSpell("test-spell", "Test Spell", player1.id)
  addSpellToHand(game, player1.id, spell)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: [],
  })

  // Player 1 ends turn (sets auto-pass)
  game.apply({ type: "END_TURN", playerId: player1.id })

  // Player 2 passes priority - spell resolves
  game.apply({ type: "PASS_PRIORITY", playerId: player2.id })

  // After resolution, player 1 is still in auto-pass mode
  // So they continue auto-advancing through steps
  // Should end up on player 2's turn
  expect(game.currentPlayerId).toBe(player2.id)
  expect(game.currentStep).toBe(Step.UNTAP)
})

// =============================================================================
// OPPONENT CAN STILL RESPOND
// =============================================================================

test("opponent can cast spell in response even when active player in auto-pass", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Player 1 casts a spell
  const spell1 = createSpell("spell-1", "Spell 1", player1.id)
  addSpellToHand(game, player1.id, spell1)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell1.instanceId,
    targets: [],
  })

  // Player 1 ends turn
  game.apply({ type: "END_TURN", playerId: player1.id })

  // Player 2 has a spell to cast in response
  const spell2 = createSpell("spell-2", "Spell 2", player2.id)
  addSpellToHand(game, player2.id, spell2)

  // Player 2 should be able to cast
  const player2Actions = game.getAllowedActionsFor(player2.id)
  expect(player2Actions).toContain("CAST_SPELL")

  // Player 2 casts in response
  game.apply({
    type: "CAST_SPELL",
    playerId: player2.id,
    cardId: spell2.instanceId,
    targets: [],
  })

  // Stack should have 2 spells
  expect(game.getStack()).toHaveLength(2)

  // Player 1 is in auto-pass, so they automatically pass
  // Player 2 should have priority now (after player 1 auto-passed)
  // Actually, after casting, priority goes to opponent (player 1)
  // Player 1 auto-passes, so priority returns to player 2
  const actionsAfterCast = game.getAllowedActionsFor(player2.id)
  expect(actionsAfterCast).toContain("PASS_PRIORITY")
})

// =============================================================================
// PASS_PRIORITY VALIDATION UNCHANGED
// =============================================================================

test("PASS_PRIORITY still requires having priority", () => {
  const { game, player1 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  // Cast spell to get something on stack
  const spell = createSpell("test-spell", "Test Spell", player1.id)
  addSpellToHand(game, player1.id, spell)
  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell.instanceId,
    targets: [],
  })

  // Priority is with player2 now
  // Player1 trying to pass should fail
  expect(() => {
    game.apply({ type: "PASS_PRIORITY", playerId: player1.id })
  }).toThrow(InvalidPlayerActionError)
})
