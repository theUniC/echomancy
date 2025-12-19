import { expect, test } from "vitest"
import { Step } from "../Steps"
import {
  addSpellToHand,
  advanceToStep,
  createSpell,
  createStartedGame,
} from "./helpers"

test("opponent can respond to a spell with CAST_SPELL", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spell1 = createSpell("spell-1", "Test Spell 1", player1.id)
  const spell2 = createSpell("spell-2", "Test Spell 2", player2.id)
  addSpellToHand(game, player1.id, spell1)
  addSpellToHand(game, player2.id, spell2)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell1.instanceId,
  })

  let stack = game.getStack()
  expect(stack).toHaveLength(1)

  let actionsPlayer1 = game.getAllowedActionsFor(player1.id)
  let actionsPlayer2 = game.getAllowedActionsFor(player2.id)

  expect(actionsPlayer1).toEqual([])
  expect(actionsPlayer2).toContain("CAST_SPELL")
  expect(actionsPlayer2).toContain("PASS_PRIORITY")

  game.apply({
    type: "CAST_SPELL",
    playerId: player2.id,
    cardId: spell2.instanceId,
  })

  stack = game.getStack()
  expect(stack).toHaveLength(2)
  expect(stack[0].card.instanceId).toBe(spell1.instanceId)
  expect(stack[1].card.instanceId).toBe(spell2.instanceId)

  actionsPlayer1 = game.getAllowedActionsFor(player1.id)
  actionsPlayer2 = game.getAllowedActionsFor(player2.id)

  expect(actionsPlayer1).toContain("PASS_PRIORITY")
  expect(actionsPlayer2).toEqual([])
})

test("stack resolves in LIFO order after both players pass", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spell1 = createSpell("spell-1", "Test Spell 1", player1.id)
  const spell2 = createSpell("spell-2", "Test Spell 2", player2.id)
  addSpellToHand(game, player1.id, spell1)
  addSpellToHand(game, player2.id, spell2)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell1.instanceId,
  })

  game.apply({
    type: "CAST_SPELL",
    playerId: player2.id,
    cardId: spell2.instanceId,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player1.id,
  })

  game.apply({
    type: "PASS_PRIORITY",
    playerId: player2.id,
  })

  const stack = game.getStack()
  expect(stack).toHaveLength(1)
  expect(stack[0].card.instanceId).toBe(spell1.instanceId)

  const graveyard2 = game.getGraveyard(player2.id)
  expect(graveyard2).toHaveLength(1)
  expect(graveyard2[0].instanceId).toBe(spell2.instanceId)
})

test("priority alternates correctly through multiple responses", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spell1 = createSpell("spell-1", "Spell 1", player1.id)
  const spell2 = createSpell("spell-2", "Spell 2", player2.id)
  const spell3 = createSpell("spell-3", "Spell 3", player1.id)
  addSpellToHand(game, player1.id, spell1)
  addSpellToHand(game, player2.id, spell2)
  addSpellToHand(game, player1.id, spell3)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell1.instanceId,
  })

  expect(game.getAllowedActionsFor(player2.id)).toContain("CAST_SPELL")
  expect(game.getAllowedActionsFor(player1.id)).toEqual([])

  game.apply({
    type: "CAST_SPELL",
    playerId: player2.id,
    cardId: spell2.instanceId,
  })

  expect(game.getAllowedActionsFor(player1.id)).toContain("CAST_SPELL")
  expect(game.getAllowedActionsFor(player2.id)).toEqual([])

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell3.instanceId,
  })

  expect(game.getAllowedActionsFor(player2.id)).toContain("PASS_PRIORITY")
  expect(game.getAllowedActionsFor(player1.id)).toEqual([])

  const stack = game.getStack()
  expect(stack).toHaveLength(3)
})

test("PLAY_LAND is only available to current player even when opponent has priority", () => {
  const { game, player1, player2 } = createStartedGame()
  advanceToStep(game, Step.FIRST_MAIN)

  const spell1 = createSpell("spell-1", "Spell 1", player1.id)
  addSpellToHand(game, player1.id, spell1)

  game.apply({
    type: "CAST_SPELL",
    playerId: player1.id,
    cardId: spell1.instanceId,
  })

  const actionsPlayer2 = game.getAllowedActionsFor(player2.id)
  expect(actionsPlayer2).not.toContain("PLAY_LAND")
  expect(actionsPlayer2).toContain("PASS_PRIORITY")

  const actionsPlayer1 = game.getAllowedActionsFor(player1.id)
  expect(actionsPlayer1).not.toContain("PLAY_LAND")
  expect(actionsPlayer1).toEqual([])
})
